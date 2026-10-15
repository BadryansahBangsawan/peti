use std::io::Read;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

use crate::links::is_name;

pub const DEVICE_PAGE: &str = "https://github.com/login/device";
const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_URL: &str = "https://api.github.com/user";
const GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";
const SERVICE: &str = "app.peti";
const TOKEN_ACCOUNT: &str = "github-token";
const LOGIN_ACCOUNT: &str = "github-login";
const MAX_BODY: usize = 1_000_000;
const USER_AGENT: &str = "Peti/0.1.0";
const SCOPE: &str = "public_repo";
const API_VERSION: &str = "2026-03-10";
const FORK_TRIES: usize = 6;

const UNPREPARED: &str = "Masuk GitHub belum disiapkan.";
const DENIED: &str = "Masuk dibatalkan.";
const EXPIRED: &str = "Kode masuk kedaluwarsa. Coba lagi.";
const NOT_RUN: &str = "Masuk tidak bisa dijalankan.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginError {
    Unprepared,
    Denied,
    Expired,
    NotRun,
}

impl std::fmt::Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Unprepared => UNPREPARED,
            Self::Denied => DENIED,
            Self::Expired => EXPIRED,
            Self::NotRun => NOT_RUN,
        })
    }
}

pub(crate) struct Account {
    token: String,
    login: String,
}

pub(crate) trait Secrets: Send + Sync {
    fn save(&self, account: &Account) -> Result<(), ()>;
    fn load(&self) -> Result<Option<Account>, ()>;
    fn clear(&self) -> Result<(), ()>;
}

pub struct OsSecrets;

impl Secrets for OsSecrets {
    fn save(&self, account: &Account) -> Result<(), ()> {
        if !secret_ok(&account.token, 2048) {
            return Err(());
        }
        let login = if is_name(&account.login) {
            account.login.as_str()
        } else {
            ""
        };
        let token_entry = keyring_entry(TOKEN_ACCOUNT)?;
        token_entry.set_password(&account.token).map_err(|_| ())?;
        if login.is_empty() {
            if delete_entry(LOGIN_ACCOUNT).is_err() {
                let _ = token_entry.delete_credential();
                return Err(());
            }
            return Ok(());
        }
        let login_entry = match keyring_entry(LOGIN_ACCOUNT) {
            Ok(entry) => entry,
            Err(()) => {
                let _ = token_entry.delete_credential();
                return Err(());
            }
        };
        if login_entry.set_password(login).is_err() {
            let _ = token_entry.delete_credential();
            return Err(());
        }
        Ok(())
    }

    fn load(&self) -> Result<Option<Account>, ()> {
        let token = match keyring_entry(TOKEN_ACCOUNT)?.get_password() {
            Ok(token) if secret_ok(&token, 2048) => token,
            Ok(_) => return Err(()),
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(_) => return Err(()),
        };
        let login = match keyring_entry(LOGIN_ACCOUNT)?.get_password() {
            Ok(value) if is_name(&value) => value,
            Ok(_) | Err(keyring::Error::NoEntry) => String::new(),
            Err(_) => return Err(()),
        };
        Ok(Some(Account { token, login }))
    }

    fn clear(&self) -> Result<(), ()> {
        delete_entry(TOKEN_ACCOUNT)?;
        delete_entry(LOGIN_ACCOUNT)?;
        Ok(())
    }
}

fn keyring_entry(account: &str) -> Result<keyring::Entry, ()> {
    keyring::Entry::new(SERVICE, account).map_err(|_| ())
}

fn delete_entry(account: &str) -> Result<(), ()> {
    match keyring_entry(account)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err(()),
    }
}

#[derive(Clone)]
struct Pending {
    device_code: String,
    interval: u64,
    next_poll: u64,
    expires: u64,
    user_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginView {
    pub connected: bool,
    pub login: String,
    pub name: String,
    pub avatar_url: String,
    pub user_code: String,
    pub interval: u64,
    pub open_page: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Take {
    Download,
    Fork,
    ForkDownload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TakeOutcome {
    pub forked: String,
    pub queued: bool,
    pub need_folder: bool,
    pub own_repo: bool,
    pub failed: bool,
}

pub(crate) struct Session<S: Send + Sync> {
    secrets: S,
    client_id: Mutex<String>,
    pending: Mutex<Option<Pending>>,
}

impl<S: Secrets> Session<S> {
    pub fn new(secrets: S, client_id: impl Into<String>) -> Self {
        Self {
            secrets,
            client_id: Mutex::new(client_id.into()),
            pending: Mutex::new(None),
        }
    }

    fn current_client_id(&self) -> String {
        match self.client_id.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    pub fn set_client_id(&self, value: &str) {
        let next = value.trim().to_owned();
        match self.client_id.lock() {
            Ok(mut guard) => *guard = next,
            Err(poisoned) => *poisoned.into_inner() = next,
        }
    }

    pub fn status(&self) -> Result<LoginView, LoginError> {
        match self.secrets.load() {
            Err(()) => Err(LoginError::NotRun),
            Ok(Some(account)) => {
                self.clear_pending();
                Ok(connected_view(&account))
            }
            Ok(None) => Ok(self.pending_view(false)),
        }
    }

    pub fn profile<F>(&self, mut fetch: F) -> Result<LoginView, LoginError>
    where
        F: FnMut(&str, &str, &str, Option<&str>) -> Result<(u16, String), LoginError>,
    {
        match self.secrets.load() {
            Err(()) => Err(LoginError::NotRun),
            Ok(Some(account)) => {
                self.clear_pending();
                let mut view = connected_view(&account);
                if let Ok((200, body)) = fetch("GET", USER_URL, "", Some(account.token.as_str())) {
                    let shown = profile_from_user(&body);
                    if is_name(&shown.login) {
                        view.login = shown.login;
                    }
                    view.name = shown.name;
                    view.avatar_url = shown.avatar_url;
                }
                Ok(view)
            }
            Ok(None) => Ok(self.pending_view(false)),
        }
    }

    pub fn begin<F>(&self, now: u64, mut fetch: F) -> Result<LoginView, LoginError>
    where
        F: FnMut(&str, &str, &str, Option<&str>) -> Result<(u16, String), LoginError>,
    {
        let client_id = self.current_client_id();
        if !client_id_ok(&client_id) {
            return Err(LoginError::Unprepared);
        }
        if let Some(account) = self.connected()? {
            return Ok(account);
        }
        let body = form(&[("client_id", client_id.as_str()), ("scope", SCOPE)]);
        let (status, text) = fetch("POST", DEVICE_CODE_URL, &body, None)?;
        let pending = parse_device(now, status, &text)?;
        let view = waiting_view(&pending, true);
        self.store_pending(pending)?;
        Ok(view)
    }

    pub fn poll<F>(&self, now: u64, mut fetch: F) -> Result<LoginView, LoginError>
    where
        F: FnMut(&str, &str, &str, Option<&str>) -> Result<(u16, String), LoginError>,
    {
        if let Some(account) = self.connected()? {
            return Ok(account);
        }
        let Some(pending) = self.snapshot() else {
            return Ok(logged_out_view());
        };
        if now < pending.next_poll {
            return Ok(waiting_view(&pending, false));
        }
        if now >= pending.expires {
            self.clear_code(&pending.device_code);
            return Err(LoginError::Expired);
        }
        let client_id = self.current_client_id();
        let body = form(&[
            ("client_id", client_id.as_str()),
            ("device_code", pending.device_code.as_str()),
            ("grant_type", GRANT),
        ]);
        let response = fetch("POST", TOKEN_URL, &body, None);
        let (status, text) = match response {
            Ok(value) => value,
            Err(err) => {
                self.reschedule(&pending, pending.interval, now);
                return Err(err);
            }
        };
        match classify_token(status, &text, pending.interval) {
            TokenStep::Wait(interval) => {
                self.reschedule(&pending, interval, now);
                Ok(self.code_view(&pending.device_code))
            }
            TokenStep::Denied => {
                self.clear_code(&pending.device_code);
                Err(LoginError::Denied)
            }
            TokenStep::Expired => {
                self.clear_code(&pending.device_code);
                Err(LoginError::Expired)
            }
            TokenStep::Again => {
                self.reschedule(&pending, pending.interval, now);
                Err(LoginError::NotRun)
            }
            TokenStep::Token(token) => self.store_token(&pending.device_code, &token, &mut fetch),
        }
    }

    pub fn logout(&self) -> Result<LoginView, LoginError> {
        self.secrets.clear().map_err(|_| LoginError::NotRun)?;
        self.clear_pending();
        Ok(logged_out_view())
    }

    pub fn take<F, W>(
        &self,
        store: &crate::store::Store,
        owner: &str,
        repo: &str,
        action: Take,
        folder: Option<&str>,
        mut fetch: F,
        mut wait: W,
    ) -> Result<TakeOutcome, LoginError>
    where
        F: FnMut(&str, &str, &str, Option<&str>) -> Result<(u16, String), LoginError>,
        W: FnMut(),
    {
        if action == Take::Download {
            return queue_names(store, owner, repo, folder);
        }
        let account = match self.secrets.load() {
            Err(()) => return Err(LoginError::NotRun),
            Ok(None) => return Ok(failed_take()),
            Ok(Some(account)) => account,
        };
        if !is_name(&account.login) || !is_name(owner) || !is_name(repo) {
            return Ok(failed_take());
        }
        if owner.eq_ignore_ascii_case(&account.login) {
            return Ok(TakeOutcome {
                own_repo: true,
                ..blank_take()
            });
        }
        if action == Take::ForkDownload && !folder_ready(store, folder)? {
            return Ok(TakeOutcome {
                need_folder: true,
                ..blank_take()
            });
        }
        let post = format!("https://api.github.com/repos/{owner}/{repo}/forks");
        let (status, body) = match fetch("POST", &post, "{}", Some(&account.token)) {
            Ok(value) => value,
            Err(_) => return Ok(failed_take()),
        };
        if status != 200 && status != 202 {
            return Ok(failed_take());
        }
        let Some((fork_owner, fork_repo)) = parse_fork(&body, &account.login) else {
            return Ok(failed_take());
        };
        let forked = format!("{fork_owner}/{fork_repo}");
        if action == Take::Fork {
            return Ok(TakeOutcome {
                forked,
                ..blank_take()
            });
        }
        let get = format!("https://api.github.com/repos/{fork_owner}/{fork_repo}");
        let mut ready = false;
        for attempt in 0..FORK_TRIES {
            if attempt > 0 {
                wait();
            }
            if let Ok((200, text)) = fetch("GET", &get, "", Some(&account.token)) {
                if parse_fork(&text, &account.login).is_some_and(|(got_owner, got_repo)| {
                    got_owner.eq_ignore_ascii_case(&fork_owner) && got_repo == fork_repo
                }) {
                    ready = true;
                    break;
                }
            }
        }
        if !ready {
            return Ok(failed_take());
        }
        let mut outcome = queue_names(store, &fork_owner, &fork_repo, folder)?;
        outcome.forked = forked;
        Ok(outcome)
    }

    fn connected(&self) -> Result<Option<LoginView>, LoginError> {
        match self.secrets.load() {
            Err(()) => Err(LoginError::NotRun),
            Ok(Some(account)) => {
                self.clear_pending();
                Ok(Some(connected_view(&account)))
            }
            Ok(None) => Ok(None),
        }
    }

    fn store_token<F>(
        &self,
        device_code: &str,
        token: &str,
        fetch: &mut F,
    ) -> Result<LoginView, LoginError>
    where
        F: FnMut(&str, &str, &str, Option<&str>) -> Result<(u16, String), LoginError>,
    {
        let shown = match fetch("GET", USER_URL, "", Some(token)) {
            Ok((200, body)) => profile_from_user(&body),
            _ => ShownProfile::blank(),
        };
        let saved = self.secrets.save(&Account {
            token: token.to_owned(),
            login: shown.login.clone(),
        });
        self.clear_code(device_code);
        saved.map_err(|_| LoginError::NotRun)?;
        Ok(LoginView {
            connected: true,
            login: shown.login,
            name: shown.name,
            avatar_url: shown.avatar_url,
            user_code: String::new(),
            interval: 0,
            open_page: false,
        })
    }

    fn snapshot(&self) -> Option<Pending> {
        self.pending.lock().ok()?.clone()
    }

    fn store_pending(&self, pending: Pending) -> Result<(), LoginError> {
        let mut guard = self.pending.lock().map_err(|_| LoginError::NotRun)?;
        *guard = Some(pending);
        Ok(())
    }

    fn clear_pending(&self) {
        if let Ok(mut guard) = self.pending.lock() {
            *guard = None;
        }
    }

    fn clear_code(&self, device_code: &str) {
        if let Ok(mut guard) = self.pending.lock() {
            if guard
                .as_ref()
                .is_some_and(|pending| pending.device_code == device_code)
            {
                *guard = None;
            }
        }
    }

    fn reschedule(&self, pending: &Pending, interval: u64, now: u64) {
        if let Ok(mut guard) = self.pending.lock() {
            if let Some(current) = guard.as_mut() {
                if current.device_code == pending.device_code {
                    current.interval = interval;
                    current.next_poll = now.saturating_add(interval);
                }
            }
        }
    }

    fn pending_view(&self, open_page: bool) -> LoginView {
        match self.snapshot() {
            Some(pending) => waiting_view(&pending, open_page),
            None => logged_out_view(),
        }
    }

    fn code_view(&self, device_code: &str) -> LoginView {
        match self.snapshot() {
            Some(pending) if pending.device_code == device_code => waiting_view(&pending, false),
            _ => logged_out_view(),
        }
    }
}

fn connected_view(account: &Account) -> LoginView {
    LoginView {
        connected: true,
        login: if is_name(&account.login) {
            account.login.clone()
        } else {
            String::new()
        },
        name: String::new(),
        avatar_url: String::new(),
        user_code: String::new(),
        interval: 0,
        open_page: false,
    }
}

fn waiting_view(pending: &Pending, open_page: bool) -> LoginView {
    LoginView {
        connected: false,
        login: String::new(),
        name: String::new(),
        avatar_url: String::new(),
        user_code: pending.user_code.clone(),
        interval: pending.interval,
        open_page,
    }
}

fn logged_out_view() -> LoginView {
    LoginView {
        connected: false,
        login: String::new(),
        name: String::new(),
        avatar_url: String::new(),
        user_code: String::new(),
        interval: 0,
        open_page: false,
    }
}

fn blank_take() -> TakeOutcome {
    TakeOutcome {
        forked: String::new(),
        queued: false,
        need_folder: false,
        own_repo: false,
        failed: false,
    }
}

fn failed_take() -> TakeOutcome {
    TakeOutcome {
        failed: true,
        ..blank_take()
    }
}

fn folder_ready(store: &crate::store::Store, folder: Option<&str>) -> Result<bool, LoginError> {
    if folder.map(str::trim).is_some_and(|value| !value.is_empty()) {
        return Ok(true);
    }
    let settings = store.settings().map_err(|_| LoginError::NotRun)?;
    Ok(settings
        .default_folder
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty()))
}

fn queue_names(
    store: &crate::store::Store,
    owner: &str,
    repo: &str,
    folder: Option<&str>,
) -> Result<TakeOutcome, LoginError> {
    match crate::queue::enqueue_discovered(store, &[(owner, repo)], folder) {
        Ok(crate::queue::QueueAsk::NeedFolder) => Ok(TakeOutcome {
            need_folder: true,
            ..blank_take()
        }),
        Ok(crate::queue::QueueAsk::Queued(items)) => Ok(TakeOutcome {
            queued: !items.is_empty(),
            ..blank_take()
        }),
        Err(_) => Err(LoginError::NotRun),
    }
}

fn parse_fork(body: &str, login: &str) -> Option<(String, String)> {
    if body.len() > MAX_BODY {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(body.trim()).ok()?;
    let owner = value.get("owner")?.get("login")?.as_str()?.trim();
    let repo = value.get("name")?.as_str()?.trim();
    if !is_name(owner) || !is_name(repo) || !owner.eq_ignore_ascii_case(login) {
        return None;
    }
    // Paste parsing strips one ".git" suffix, so this name would queue a different repo.
    if repo.ends_with(".git") {
        return None;
    }
    if let Some(full) = value.get("full_name").and_then(|item| item.as_str()) {
        let (left, right) = full.trim().split_once('/')?;
        if !left.eq_ignore_ascii_case(owner) || right != repo {
            return None;
        }
    }
    Some((owner.to_owned(), repo.to_owned()))
}

pub(crate) fn client_id_ok(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn secret_ok(value: &str, max: usize) -> bool {
    (1..=max).contains(&value.len()) && value.bytes().all(|byte| (0x21..=0x7e).contains(&byte))
}

fn user_code_ok(value: &str) -> bool {
    (4..=20).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn clamp_interval(value: Option<u64>) -> u64 {
    value.unwrap_or(5).max(5).min(3600)
}

fn slowed(current: u64, body: Option<u64>) -> u64 {
    let bumped = current.saturating_add(5);
    body.map(|value| value.max(bumped))
        .unwrap_or(bumped)
        .max(5)
        .min(3600)
}

fn expires_at(now: u64, value: Option<u64>) -> u64 {
    now.saturating_add(value.filter(|secs| *secs > 0).unwrap_or(900))
}

enum TokenStep {
    Wait(u64),
    Denied,
    Expired,
    Again,
    Token(String),
}

fn classify_token(status: u16, body: &str, interval: u64) -> TokenStep {
    let Some(map) = fields(body) else {
        return TokenStep::Again;
    };
    if let Some(error) = text_field(&map, "error") {
        return match error.as_str() {
            "authorization_pending" => TokenStep::Wait(interval),
            "slow_down" => TokenStep::Wait(slowed(interval, number_field(&map, "interval"))),
            "access_denied" => TokenStep::Denied,
            "expired_token" | "token_expired" => TokenStep::Expired,
            _ => TokenStep::Again,
        };
    }
    if status == 200 {
        if let Some(token) = text_field(&map, "access_token") {
            if secret_ok(&token, 2048) {
                return TokenStep::Token(token);
            }
        }
    }
    TokenStep::Again
}

fn parse_device(now: u64, status: u16, body: &str) -> Result<Pending, LoginError> {
    if status != 200 {
        return Err(LoginError::NotRun);
    }
    let map = fields(body).ok_or(LoginError::NotRun)?;
    if text_field(&map, "verification_uri").as_deref() != Some(DEVICE_PAGE) {
        return Err(LoginError::NotRun);
    }
    let user_code = text_field(&map, "user_code").ok_or(LoginError::NotRun)?;
    if !user_code_ok(&user_code) {
        return Err(LoginError::NotRun);
    }
    let device_code = text_field(&map, "device_code").ok_or(LoginError::NotRun)?;
    if !secret_ok(&device_code, 256) {
        return Err(LoginError::NotRun);
    }
    let interval = clamp_interval(number_field(&map, "interval"));
    Ok(Pending {
        device_code,
        interval,
        next_poll: now.saturating_add(interval),
        expires: expires_at(now, number_field(&map, "expires_in")),
        user_code,
    })
}

struct ShownProfile {
    login: String,
    name: String,
    avatar_url: String,
}

impl ShownProfile {
    fn blank() -> Self {
        Self {
            login: String::new(),
            name: String::new(),
            avatar_url: String::new(),
        }
    }
}

fn profile_from_user(body: &str) -> ShownProfile {
    let Some(map) = fields(body) else {
        return ShownProfile::blank();
    };
    let login = match text_field(&map, "login") {
        Some(login) if is_name(&login) => login,
        _ => String::new(),
    };
    let raw_name = map.get("name").and_then(|value| value.as_str()).unwrap_or("");
    let raw_avatar = map
        .get("avatar_url")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    ShownProfile {
        login,
        name: shown_name(raw_name),
        avatar_url: shown_avatar(raw_avatar),
    }
}

fn shown_name(value: &str) -> String {
    let name = value.trim();
    if name.is_empty() || name.len() > 128 || hidden_char(name) {
        String::new()
    } else {
        name.to_owned()
    }
}

fn shown_avatar(value: &str) -> String {
    if value.is_empty()
        || value.len() > 512
        || value.chars().any(char::is_whitespace)
        || hidden_char(value)
    {
        return String::new();
    }
    let Ok(url) = reqwest::Url::parse(value) else {
        return String::new();
    };
    if url.scheme() != "https"
        || url.host_str() != Some("avatars.githubusercontent.com")
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path().len() < 2
    {
        return String::new();
    }
    value.to_owned()
}

fn hidden_char(value: &str) -> bool {
    value.chars().any(|ch| {
        let code = ch as u32;
        code <= 0x1f
            || code == 0x7f
            || (0x202a..=0x202e).contains(&code)
            || (0x2066..=0x2069).contains(&code)
    })
}

fn fields(body: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_BODY {
        return None;
    }
    if trimmed.starts_with('{') {
        let value: serde_json::Value = serde_json::from_str(trimmed).ok()?;
        return value.as_object().cloned();
    }
    if !trimmed.contains('=') {
        return None;
    }
    let mut map = serde_json::Map::new();
    for part in trimmed.split('&') {
        if part.is_empty() {
            continue;
        }
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        map.insert(decode(key)?, serde_json::Value::String(decode(value)?));
    }
    Some(map)
}

fn text_field(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<String> {
    let text = map.get(key)?.as_str()?.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_owned())
    }
}

fn number_field(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<u64> {
    match map.get(key)? {
        serde_json::Value::Number(value) => value.as_u64(),
        serde_json::Value::String(value) => value.trim().parse().ok(),
        _ => None,
    }
}

fn form(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(key, value)| format!("{}={}", encode(key), encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => out.push(b' '),
            b'%' => {
                if index + 2 >= bytes.len() {
                    return None;
                }
                let high = from_hex(bytes[index + 1])?;
                let low = from_hex(bytes[index + 2])?;
                out.push((high << 4) | low);
                index += 2;
            }
            byte => out.push(byte),
        }
        index += 1;
    }
    String::from_utf8(out).ok()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub fn login_request_allowed(method: &str, url: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(url) else {
        return false;
    };
    if url.scheme() != "https"
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return false;
    }
    let host = url.host_str();
    let path = url.path();
    match (method, host, path) {
        ("POST", Some("github.com"), "/login/device/code" | "/login/oauth/access_token") => true,
        ("GET", Some("api.github.com"), "/user") => true,
        _ => false,
    }
}

pub fn fetch_login(
    method: &str,
    url: &str,
    body: &str,
    token: Option<&str>,
) -> Result<(u16, String), LoginError> {
    if !login_request_allowed(method, url) {
        return Err(LoginError::NotRun);
    }
    if token.is_some() && !(method == "GET" && url == USER_URL) {
        return Err(LoginError::NotRun);
    }
    if let Some(token) = token {
        if !secret_ok(token, 2048) {
            return Err(LoginError::NotRun);
        }
    }
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(20))
        .https_only(true)
        .build()
        .map_err(|_| LoginError::NotRun)?;
    let mut request = match method {
        "GET" => client.get(url),
        "POST" => client
            .post(url)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body.to_owned()),
        _ => return Err(LoginError::NotRun),
    };
    request = request.header("accept", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let mut response = request.send().map_err(|_| LoginError::NotRun)?;
    let status = response.status().as_u16();
    let bytes = read_capped(&mut response).map_err(|_| LoginError::NotRun)?;
    let text = String::from_utf8(bytes).map_err(|_| LoginError::NotRun)?;
    Ok((status, text))
}

fn read_capped(response: &mut impl Read) -> Result<Vec<u8>, ()> {
    let mut buf = vec![0_u8; MAX_BODY + 1];
    let mut read = 0;
    while read <= MAX_BODY {
        match response.read(&mut buf[read..]) {
            Ok(0) => break,
            Ok(n) => read += n,
            Err(_) => return Err(()),
        }
    }
    if read > MAX_BODY {
        return Err(());
    }
    buf.truncate(read);
    Ok(buf)
}

pub fn fork_request_allowed(method: &str, url: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(url) else {
        return false;
    };
    if url.scheme() != "https"
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.host_str() != Some("api.github.com")
        || url.query().is_some()
    {
        return false;
    }
    let parts: Vec<&str> = url.path().split('/').collect();
    match (method, parts.as_slice()) {
        ("POST", ["", "repos", owner, repo, "forks"]) => is_name(owner) && is_name(repo),
        ("GET", ["", "repos", owner, repo]) => is_name(owner) && is_name(repo),
        _ => false,
    }
}

pub fn fetch_fork(
    method: &str,
    url: &str,
    body: &str,
    token: Option<&str>,
) -> Result<(u16, String), LoginError> {
    let Some(token) = token else {
        return Err(LoginError::NotRun);
    };
    if !fork_request_allowed(method, url) || !secret_ok(token, 2048) {
        return Err(LoginError::NotRun);
    }
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(20))
        .https_only(true)
        .build()
        .map_err(|_| LoginError::NotRun)?;
    let mut request = match method {
        "GET" => client.get(url),
        "POST" => client
            .post(url)
            .header("content-type", "application/json")
            .body(body.to_owned()),
        _ => return Err(LoginError::NotRun),
    };
    request = request
        .header("accept", "application/vnd.github+json")
        .header("x-github-api-version", API_VERSION)
        .header("authorization", format!("Bearer {token}"));
    let mut response = request.send().map_err(|_| LoginError::NotRun)?;
    let status = response.status().as_u16();
    let bytes = read_capped(&mut response).map_err(|_| LoginError::NotRun)?;
    let text = String::from_utf8(bytes).map_err(|_| LoginError::NotRun)?;
    Ok((status, text))
}

pub fn open_login_page() {
    let (program, args) = login_args();
    let _ = Command::new(program).args(args).spawn();
}

fn login_args() -> (&'static str, &'static [&'static str]) {
    #[cfg(target_os = "macos")]
    {
        return ("open", &[DEVICE_PAGE]);
    }
    #[cfg(target_os = "windows")]
    {
        return ("cmd", &["/C", "start", "", DEVICE_PAGE]);
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        ("xdg-open", &[DEVICE_PAGE])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    const CLIENT: &str = "Iv1.peti-test";
    const NOW: u64 = 1_700_000_000;
    const TOKEN: &str = "peti-secret-token-9f3a";
    const DEVICE: &str = "device-secret-9f3a";
    const CODE: &str = "ABCD-1234";

    struct MemorySecrets {
        inner: Mutex<Option<Account>>,
    }

    impl MemorySecrets {
        fn new() -> Self {
            Self {
                inner: Mutex::new(None),
            }
        }
    }

    impl Secrets for MemorySecrets {
        fn save(&self, account: &Account) -> Result<(), ()> {
            let mut guard = self.inner.lock().map_err(|_| ())?;
            *guard = Some(Account {
                token: account.token.clone(),
                login: account.login.clone(),
            });
            Ok(())
        }

        fn load(&self) -> Result<Option<Account>, ()> {
            let guard = self.inner.lock().map_err(|_| ())?;
            Ok(guard.as_ref().map(|account| Account {
                token: account.token.clone(),
                login: account.login.clone(),
            }))
        }

        fn clear(&self) -> Result<(), ()> {
            let mut guard = self.inner.lock().map_err(|_| ())?;
            *guard = None;
            Ok(())
        }
    }

    struct BrokenSecrets;

    impl Secrets for BrokenSecrets {
        fn save(&self, _: &Account) -> Result<(), ()> {
            Err(())
        }

        fn load(&self) -> Result<Option<Account>, ()> {
            Err(())
        }

        fn clear(&self) -> Result<(), ()> {
            Err(())
        }
    }

    struct SaveFails;

    impl Secrets for SaveFails {
        fn save(&self, _: &Account) -> Result<(), ()> {
            Err(())
        }

        fn load(&self) -> Result<Option<Account>, ()> {
            Ok(None)
        }

        fn clear(&self) -> Result<(), ()> {
            Ok(())
        }
    }

    struct Script {
        calls: Vec<(String, String, String, Option<String>)>,
        responses: Vec<Result<(u16, String), LoginError>>,
    }

    impl Script {
        fn next(
            &mut self,
            method: &str,
            url: &str,
            body: &str,
            token: Option<&str>,
        ) -> Result<(u16, String), LoginError> {
            self.calls.push((
                method.to_owned(),
                url.to_owned(),
                body.to_owned(),
                token.map(str::to_owned),
            ));
            if self.responses.is_empty() {
                return Err(LoginError::NotRun);
            }
            self.responses.remove(0)
        }
    }

    fn device_json(uri: &str, interval: &str, expires: &str) -> String {
        format!(
            r#"{{"device_code":"{DEVICE}","user_code":"{CODE}","verification_uri":"{uri}","expires_in":{expires},"interval":{interval}}}"#
        )
    }

    fn token_json(body: &str) -> Result<(u16, String), LoginError> {
        Ok((200, body.to_owned()))
    }

    fn assert_no_secret(body: &str) {
        assert!(!body.contains("client_secret"), "{body}");
        let map = fields(body).unwrap();
        assert!(map.get("client_secret").is_none());
        match map.get("scope").and_then(|value| value.as_str()) {
            None | Some("public_repo") => {}
            Some(other) => panic!("unexpected scope {other} in {body}"),
        }
    }

    #[test]
    fn missing_client_id_makes_no_request() {
        let session = Session::new(MemorySecrets::new(), "bad id");
        let mut calls = 0;
        let err = session
            .begin(NOW, |_, _, _, _| {
                calls += 1;
                Err(LoginError::NotRun)
            })
            .unwrap_err();
        assert_eq!(err, LoginError::Unprepared);
        assert_eq!(err.to_string(), UNPREPARED);
        assert_eq!(calls, 0);
        let empty = Session::new(MemorySecrets::new(), "");
        let err = empty
            .begin(NOW, |_, _, _, _| {
                calls += 1;
                Err(LoginError::NotRun)
            })
            .unwrap_err();
        assert_eq!(err, LoginError::Unprepared);
        assert_eq!(calls, 0);
        assert!(!client_id_ok(&"a".repeat(129)));
    }

    #[test]
    fn device_flow_sends_public_repo_and_no_secret() {
        let session = Session::new(MemorySecrets::new(), CLIENT);
        let mut script = Script {
            calls: Vec::new(),
            responses: vec![token_json(&device_json(DEVICE_PAGE, "5", "900"))],
        };
        let view = session
            .begin(NOW, |method, url, body, token| {
                script.next(method, url, body, token)
            })
            .unwrap();
        assert!(view.open_page);
        assert_eq!(view.user_code, CODE);
        assert!(!view.connected);
        assert_eq!(script.calls.len(), 1);
        let (method, url, body, token) = &script.calls[0];
        assert_eq!(method, "POST");
        assert_eq!(url, DEVICE_CODE_URL);
        assert!(token.is_none());
        assert_eq!(body, "client_id=Iv1.peti-test&scope=public_repo");
        assert_no_secret(body);
        let early = session
            .poll(NOW + 4, |_, _, _, _| -> Result<(u16, String), LoginError> {
                panic!("poll before the interval");
            })
            .unwrap();
        assert_eq!(early.user_code, CODE);
        assert!(!early.open_page);
    }

    #[test]
    fn a_hostile_verification_page_is_dropped() {
        for uri in [
            "https://github.com.evil.test/login/device",
            "https://github.com/login/device.evil",
            "https://evil.test/login/device",
            "http://github.com/login/device",
            "https://github.com/login/device/extra",
        ] {
            let session = Session::new(MemorySecrets::new(), CLIENT);
            let mut calls = 0;
            let err = session
                .begin(NOW, |method, url, body, _| {
                    calls += 1;
                    assert_eq!(method, "POST");
                    assert_eq!(url, DEVICE_CODE_URL);
                    assert_no_secret(body);
                    token_json(&device_json(uri, "5", "900"))
                })
                .unwrap_err();
            assert_eq!(err, LoginError::NotRun, "{uri}");
            assert_eq!(calls, 1, "{uri}");
            let view = session.status().unwrap();
            assert!(!view.connected);
            assert!(view.user_code.is_empty(), "{uri}");
            assert!(!view.open_page);
        }
    }

    #[test]
    fn poll_waits_then_slow_down_adds_five_seconds() {
        let session = Session::new(MemorySecrets::new(), CLIENT);
        let mut calls = 0;
        session
            .begin(NOW, |_, _, body, token| {
                assert!(token.is_none());
                assert_no_secret(body);
                token_json(&device_json(DEVICE_PAGE, "0", "30"))
            })
            .unwrap();
        let held = session
            .poll(NOW + 4, |_, _, _, _| -> Result<(u16, String), LoginError> {
                panic!("interval of 0 still waits 5 seconds");
            })
            .unwrap();
        assert_eq!(held.interval, 5);
        session
            .poll(NOW + 5, |_, url, body, token| {
                calls += 1;
                assert_eq!(url, TOKEN_URL);
                assert!(token.is_none());
                assert_no_secret(body);
                let map = fields(body).unwrap();
                assert_eq!(map.len(), 3);
                assert_eq!(
                    map.get("grant_type").and_then(|value| value.as_str()),
                    Some(GRANT)
                );
                token_json(r#"{"error":"slow_down"}"#)
            })
            .unwrap();
        assert_eq!(calls, 1);
        let waiting = session
            .poll(
                NOW + 14,
                |_, _, _, _| -> Result<(u16, String), LoginError> {
                    panic!("slow_down holds the extra five seconds");
                },
            )
            .unwrap();
        assert_eq!(waiting.interval, 10);
        session
            .poll(NOW + 15, |_, _, _, _| {
                calls += 1;
                token_json(r#"{"error":"slow_down","interval":15}"#)
            })
            .unwrap();
        assert_eq!(session.status().unwrap().interval, 15);
        let capped = Session::new(MemorySecrets::new(), CLIENT);
        capped
            .begin(NOW, |_, _, _, _| {
                token_json(&device_json(DEVICE_PAGE, "3598", "9000"))
            })
            .unwrap();
        capped
            .poll(NOW + 3598, |_, _, _, _| {
                token_json(r#"{"error":"slow_down"}"#)
            })
            .unwrap();
        assert_eq!(capped.status().unwrap().interval, 3600);
    }

    #[test]
    fn denial_and_expiry_save_nothing() {
        for (body, expected) in [
            (r#"{"error":"access_denied"}"#, LoginError::Denied),
            (r#"{"error":"expired_token"}"#, LoginError::Expired),
            (r#"{"error":"token_expired"}"#, LoginError::Expired),
        ] {
            let session = Session::new(MemorySecrets::new(), CLIENT);
            session
                .begin(NOW, |_, _, _, _| {
                    token_json(&device_json(DEVICE_PAGE, "5", "900"))
                })
                .unwrap();
            let err = session
                .poll(NOW + 5, |_, _, request, token| {
                    assert!(token.is_none());
                    assert_no_secret(request);
                    token_json(body)
                })
                .unwrap_err();
            assert_eq!(err, expected, "{body}");
            assert!(err.to_string() == DENIED || err.to_string() == EXPIRED);
            assert!(session.secrets.load().unwrap().is_none(), "{body}");
            assert!(session.status().unwrap().user_code.is_empty(), "{body}");
        }
        let session = Session::new(MemorySecrets::new(), CLIENT);
        session
            .begin(NOW, |_, _, _, _| {
                token_json(&device_json(DEVICE_PAGE, "5", "10"))
            })
            .unwrap();
        let mut calls = 0;
        let err = session
            .poll(NOW + 10, |_, _, _, _| {
                calls += 1;
                Err(LoginError::NotRun)
            })
            .unwrap_err();
        assert_eq!(err, LoginError::Expired);
        assert_eq!(calls, 0);
    }

    #[test]
    fn login_then_logout_can_repeat_and_the_database_has_no_token() {
        let path = temp_db();
        let store = crate::store::Store::open(&path).unwrap();
        store
            .insert_queue("octocat", "Hello-World", "", "/tmp/peti-kept")
            .unwrap();
        store
            .insert_history(
                "octocat",
                "Hello-World",
                "",
                "/tmp/peti-kept",
                "2026-10-14T00:00:00Z",
            )
            .unwrap();
        let session = Session::new(MemorySecrets::new(), CLIENT);
        let mut script = Script {
            calls: Vec::new(),
            responses: vec![
                token_json(&device_json(DEVICE_PAGE, "5", "900")),
                token_json(&format!(
                    r#"{{"access_token":"{TOKEN}","token_type":"bearer","scope":""}}"#
                )),
                token_json(r#"{"login":"octocat"}"#),
                token_json(&device_json(DEVICE_PAGE, "5", "900")),
                token_json(r#"{"access_token":"peti-secret-token-second","token_type":"bearer"}"#),
                Ok((500, "nope".to_owned())),
            ],
        };
        session
            .begin(NOW, |method, url, body, token| {
                script.next(method, url, body, token)
            })
            .unwrap();
        let view = session
            .poll(NOW + 5, |method, url, body, token| {
                script.next(method, url, body, token)
            })
            .unwrap();
        assert!(view.connected);
        assert_eq!(view.login, "octocat");
        assert!(view.user_code.is_empty());
        assert!(!view.open_page);
        let user_call = script.calls.iter().find(|call| call.1 == USER_URL).unwrap();
        assert_eq!(user_call.0, "GET");
        assert_eq!(user_call.3.as_deref(), Some(TOKEN));
        assert!(!user_call.2.contains(TOKEN));
        for call in &script.calls {
            if call.1 != USER_URL {
                assert!(call.3.is_none(), "{}", call.1);
                assert_no_secret(&call.2);
            }
        }
        let saved = session.secrets.load().unwrap().unwrap();
        assert_eq!(saved.token, TOKEN);
        assert_eq!(saved.login, "octocat");
        assert!(!file_has(&path, TOKEN));
        assert!(!file_has(&path, DEVICE));
        assert!(!file_has(&path, "client_secret"));
        let logged_out = session.logout().unwrap();
        assert!(!logged_out.connected);
        assert!(session.secrets.load().unwrap().is_none());
        assert_eq!(store.queue().unwrap().len(), 1);
        assert_eq!(store.history().unwrap().len(), 1);
        assert_eq!(store.queue().unwrap()[0].repo, "Hello-World");
        session
            .begin(NOW + 20, |method, url, body, token| {
                script.next(method, url, body, token)
            })
            .unwrap();
        let again = session
            .poll(NOW + 25, |method, url, body, token| {
                script.next(method, url, body, token)
            })
            .unwrap();
        assert!(again.connected);
        assert!(again.login.is_empty());
        assert_eq!(
            session.secrets.load().unwrap().unwrap().token,
            "peti-secret-token-second"
        );
        assert!(!file_has(&path, "peti-secret-token-second"));
        assert_eq!(store.queue().unwrap().len(), 1);
        assert_eq!(store.history().unwrap().len(), 1);
        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_bad_account_name_is_not_invented_and_a_save_failure_drops_the_code() {
        let session = Session::new(MemorySecrets::new(), CLIENT);
        session
            .begin(NOW, |_, _, _, _| {
                token_json(&device_json(DEVICE_PAGE, "5", "900"))
            })
            .unwrap();
        let view = session
            .poll(NOW + 5, |_, url, _, token| {
                if url == USER_URL {
                    assert_eq!(token, Some(TOKEN));
                    return token_json(r#"{"login":"not a name"}"#);
                }
                token_json(&format!(r#"{{"access_token":"{TOKEN}"}}"#))
            })
            .unwrap();
        assert!(view.connected);
        assert!(view.login.is_empty());
        assert_eq!(session.secrets.load().unwrap().unwrap().login, "");
        let broken = Session::new(SaveFails, CLIENT);
        broken
            .begin(NOW, |_, _, _, _| {
                token_json(&device_json(DEVICE_PAGE, "5", "900"))
            })
            .unwrap();
        let err = broken
            .poll(NOW + 5, |_, url, _, _| {
                if url == USER_URL {
                    return Err(LoginError::NotRun);
                }
                token_json(&format!(r#"{{"access_token":"{TOKEN}"}}"#))
            })
            .unwrap_err();
        assert_eq!(err, LoginError::NotRun);
        let after = broken.status().unwrap();
        assert!(!after.connected);
        assert!(after.user_code.is_empty());
        assert!(broken.snapshot().is_none());
        assert!(broken.secrets.load().unwrap().is_none());
    }

    #[test]
    fn a_saved_login_loads_the_public_profile_and_drops_a_foreign_avatar() {
        let secrets = MemorySecrets::new();
        secrets
            .save(&Account {
                token: TOKEN.to_owned(),
                login: "octocat".to_owned(),
            })
            .unwrap();
        let session = Session::new(secrets, CLIENT);
        let view = session
            .profile(|method, url, body, token| {
                assert_eq!(method, "GET");
                assert_eq!(url, USER_URL);
                assert!(body.is_empty());
                assert_eq!(token, Some(TOKEN));
                token_json(
                    r#"{"login":"octocat","name":" Mona Lisa ","avatar_url":"https://avatars.githubusercontent.com/u/1?v=4"}"#,
                )
            })
            .unwrap();
        assert!(view.connected);
        assert_eq!(view.login, "octocat");
        assert_eq!(view.name, "Mona Lisa");
        assert_eq!(
            view.avatar_url,
            "https://avatars.githubusercontent.com/u/1?v=4"
        );
        let saved = session.secrets.load().unwrap().unwrap();
        assert_eq!(saved.login, "octocat");
        assert_eq!(saved.token, TOKEN);

        for avatar in [
            "https://evil.test/a.png",
            "http://avatars.githubusercontent.com/u/1",
            "https://avatars.githubusercontent.com.evil.test/u/1",
            "https://user:pw@avatars.githubusercontent.com/u/1",
            "https://github.com/identicons/octocat.png",
        ] {
            let body = format!(r#"{{"login":"octocat","name":"ok","avatar_url":"{avatar}"}}"#);
            let hostile = session
                .profile(|_, _, _, _| token_json(&body))
                .unwrap();
            assert!(hostile.avatar_url.is_empty(), "{avatar}");
            assert_eq!(hostile.name, "ok");
            assert!(hostile.connected);
        }

        let broken_name = session
            .profile(|_, _, _, _| token_json(r#"{"login":"octocat","name":"a\nb","avatar_url":"https://avatars.githubusercontent.com/u/1"}"#))
            .unwrap();
        assert!(broken_name.name.is_empty());
        assert_eq!(
            broken_name.avatar_url,
            "https://avatars.githubusercontent.com/u/1"
        );

        let offline = session
            .profile(|_, _, _, _| Err(LoginError::NotRun))
            .unwrap();
        assert!(offline.connected);
        assert_eq!(offline.login, "octocat");
        assert!(offline.name.is_empty());
        assert!(offline.avatar_url.is_empty());

        let logged_out = Session::new(MemorySecrets::new(), CLIENT);
        let mut calls = 0;
        let empty = logged_out
            .profile(|_, _, _, _| {
                calls += 1;
                Err(LoginError::NotRun)
            })
            .unwrap();
        assert!(!empty.connected);
        assert_eq!(calls, 0);
    }

    #[test]
    fn a_failed_secret_load_is_not_reported_as_logged_out() {
        let session = Session::new(BrokenSecrets, CLIENT);
        assert_eq!(session.status().unwrap_err(), LoginError::NotRun);
        assert_eq!(
            session.status().unwrap_err().to_string(),
            "Masuk tidak bisa dijalankan."
        );
        let mut calls = 0;
        let err = session
            .begin(NOW, |_, _, _, _| {
                calls += 1;
                Err(LoginError::NotRun)
            })
            .unwrap_err();
        assert_eq!(err, LoginError::NotRun);
        assert_eq!(calls, 0);
        assert_eq!(session.logout().unwrap_err(), LoginError::NotRun);
    }

    #[test]
    fn login_urls_are_not_download_urls_and_a_token_is_not_attached_early() {
        assert!(login_request_allowed("POST", DEVICE_CODE_URL));
        assert!(login_request_allowed("POST", TOKEN_URL));
        assert!(login_request_allowed("GET", USER_URL));
        for raw in [
            "https://github.com/octocat/Hello-World/archive/HEAD.zip",
            "https://codeload.github.com/octocat/Hello-World/zip/refs",
            "https://api.github.com/search/repositories",
            "http://github.com/login/device/code",
            "https://github.com.evil.test/login/device/code",
            "https://user:pw@github.com/login/device/code",
            "https://github.com:444/login/device/code",
            "https://github.com/login/device",
        ] {
            assert!(!login_request_allowed("POST", raw), "{raw}");
            assert!(!login_request_allowed("GET", raw), "{raw}");
        }
        assert!(!login_request_allowed("GET", DEVICE_CODE_URL));
        assert!(!login_request_allowed("POST", USER_URL));
        let err = fetch_login(
            "POST",
            DEVICE_CODE_URL,
            "client_id=Iv1.peti-test",
            Some(TOKEN),
        )
        .unwrap_err();
        assert_eq!(err, LoginError::NotRun);
        let err = fetch_login("POST", "https://evil.test/login", "x", None).unwrap_err();
        assert_eq!(err, LoginError::NotRun);
    }

    fn signed(login: &str) -> Session<MemorySecrets> {
        let secrets = MemorySecrets::new();
        secrets
            .save(&Account {
                token: TOKEN.to_owned(),
                login: login.to_owned(),
            })
            .unwrap();
        Session::new(secrets, CLIENT)
    }

    fn fork_json(login: &str, repo: &str) -> String {
        format!(
            r#"{{"name":"{repo}","full_name":"{login}/{repo}","html_url":"https://evil.test/steal","owner":{{"login":"{login}"}}}}"#
        )
    }

    fn open_store() -> (PathBuf, crate::store::Store) {
        let path = temp_db();
        let store = crate::store::Store::open(&path).unwrap();
        (path, store)
    }

    #[test]
    fn download_while_logged_in_does_not_call_fork() {
        let session = signed("me");
        let (path, store) = open_store();
        let outcome = session
            .take(
                &store,
                "octocat",
                "Hello-World",
                Take::Download,
                Some("/tmp/peti-download-only"),
                |_, _, _, _| panic!("download must not call fork"),
                || panic!("download must not wait"),
            )
            .unwrap();
        assert!(outcome.queued);
        assert!(outcome.forked.is_empty());
        assert!(!outcome.failed);
        let rows = store.queue().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].owner, "octocat");
        assert_eq!(rows[0].repo, "Hello-World");
        assert!(store.history().unwrap().is_empty());
        assert!(!file_has(&path, TOKEN));
        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn fork_only_writes_no_folder_and_no_queue_row() {
        let session = signed("me");
        let (path, store) = open_store();
        let missing = std::env::temp_dir().join(format!("peti-fork-absent-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&missing);
        assert!(!missing.exists());
        let mut script = Script {
            calls: Vec::new(),
            responses: vec![Ok((202, fork_json("me", "Hello-World")))],
        };
        let outcome = session
            .take(
                &store,
                "octocat",
                "Hello-World",
                Take::Fork,
                missing.to_str(),
                |method, url, body, token| script.next(method, url, body, token),
                || panic!("fork only does not wait"),
            )
            .unwrap();
        assert_eq!(outcome.forked, "me/Hello-World");
        assert!(!outcome.queued);
        assert!(!outcome.failed);
        assert!(!outcome.own_repo);
        assert!(store.queue().unwrap().is_empty());
        assert!(store.history().unwrap().is_empty());
        assert!(!missing.exists());
        assert_eq!(script.calls.len(), 1);
        let (method, url, body, token) = &script.calls[0];
        assert_eq!(method, "POST");
        assert_eq!(
            url,
            "https://api.github.com/repos/octocat/Hello-World/forks"
        );
        assert_eq!(body, "{}");
        assert!(!body.contains("client_secret"));
        assert!(!body.contains("scope"));
        assert_eq!(token.as_deref(), Some(TOKEN));
        assert!(!file_has(&path, TOKEN));
        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn fork_download_queues_the_account_and_history_names_that_account() {
        let session = signed("me");
        let (path, store) = open_store();
        let mut waits = 0;
        let mut script = Script {
            calls: Vec::new(),
            responses: vec![
                Ok((202, fork_json("me", "Hello-World"))),
                Ok((404, "missing".to_owned())),
                Ok((200, fork_json("me", "Hello-World"))),
            ],
        };
        let outcome = session
            .take(
                &store,
                "octocat",
                "Hello-World",
                Take::ForkDownload,
                Some("/tmp/peti-fork-download"),
                |method, url, body, token| script.next(method, url, body, token),
                || waits += 1,
            )
            .unwrap();
        assert_eq!(outcome.forked, "me/Hello-World");
        assert!(outcome.queued);
        assert!(!outcome.failed);
        assert_eq!(waits, 1);
        assert_eq!(script.calls.len(), 3);
        assert_eq!(script.calls[1].0, "GET");
        assert_eq!(
            script.calls[1].1,
            "https://api.github.com/repos/me/Hello-World"
        );
        assert_eq!(script.calls[2].1, script.calls[1].1);
        assert!(
            script
                .calls
                .iter()
                .all(|call| call.3.as_deref() == Some(TOKEN))
        );
        assert!(
            script
                .calls
                .iter()
                .all(|call| !call.1.contains("evil.test"))
        );
        assert!(
            script
                .calls
                .iter()
                .filter(|call| call.0 == "GET")
                .all(|call| !call.1.contains("octocat"))
        );
        let rows = store.queue().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].owner, "me");
        assert_eq!(rows[0].repo, "Hello-World");
        store
            .set_status(rows[0].id, crate::store::Status::Berjalan)
            .unwrap();
        store
            .set_folder(rows[0].id, "/tmp/peti-fork-download/me-Hello-World")
            .unwrap();
        store.finish_item(rows[0].id, 1, Some(1)).unwrap();
        let history = store
            .archive_finished(rows[0].id, "2026-10-14T00:00:00Z")
            .unwrap()
            .unwrap();
        assert_eq!(history.owner, "me");
        assert_ne!(history.owner, "octocat");
        assert!(!file_has(&path, TOKEN));
        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_git_suffix_fork_name_is_not_queued_as_another_repo() {
        let session = signed("me");
        let (path, store) = open_store();
        let mut script = Script {
            calls: Vec::new(),
            responses: vec![Ok((202, fork_json("me", "notes.git")))],
        };
        let outcome = session
            .take(
                &store,
                "octocat",
                "notes",
                Take::ForkDownload,
                Some("/tmp/peti-fork-git-suffix"),
                |method, url, body, token| script.next(method, url, body, token),
                || panic!("a rejected fork name does not wait"),
            )
            .unwrap();
        assert!(outcome.failed);
        assert!(outcome.forked.is_empty());
        assert!(!outcome.queued);
        assert!(store.queue().unwrap().is_empty());
        assert_eq!(script.calls.len(), 1);
        assert_eq!(script.calls[0].0, "POST");
        assert!(!file_has(&path, TOKEN));
        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_fork_that_never_appears_does_not_queue_the_origin() {
        let session = signed("me");
        let (path, store) = open_store();
        let mut waits = 0;
        let mut responses = vec![Ok((202, fork_json("me", "Hello-World")))];
        for _ in 0..FORK_TRIES {
            responses.push(Ok((404, "missing".to_owned())));
        }
        let mut script = Script {
            calls: Vec::new(),
            responses,
        };
        let outcome = session
            .take(
                &store,
                "octocat",
                "Hello-World",
                Take::ForkDownload,
                Some("/tmp/peti-fork-missing"),
                |method, url, body, token| script.next(method, url, body, token),
                || waits += 1,
            )
            .unwrap();
        assert!(outcome.failed);
        assert!(outcome.forked.is_empty());
        assert!(!outcome.queued);
        assert!(store.queue().unwrap().is_empty());
        assert_eq!(waits, FORK_TRIES - 1);
        assert_eq!(script.calls.len(), 1 + FORK_TRIES);
        assert!(
            script
                .calls
                .iter()
                .all(|call| call.1.contains("/me/Hello-World") || call.1.ends_with("/forks"))
        );
        assert!(
            script
                .calls
                .iter()
                .all(|call| !call.1.contains("octocat") || call.0 == "POST")
        );
        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn an_own_repo_is_not_forked() {
        let session = signed("Octocat");
        let (path, store) = open_store();
        let outcome = session
            .take(
                &store,
                "octocat",
                "Hello-World",
                Take::Fork,
                Some("/tmp/peti-own"),
                |_, _, _, _| panic!("an own repo makes no request"),
                || panic!("an own repo does not wait"),
            )
            .unwrap();
        assert!(outcome.own_repo);
        assert!(!outcome.failed);
        assert!(!outcome.queued);
        assert!(store.queue().unwrap().is_empty());
        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn logged_out_or_unnamed_fork_makes_no_request() {
        let (path, store) = open_store();
        let logged_out = Session::new(MemorySecrets::new(), CLIENT);
        let outcome = logged_out
            .take(
                &store,
                "octocat",
                "Hello-World",
                Take::Fork,
                None,
                |_, _, _, _| panic!("logged out fork makes no request"),
                || {},
            )
            .unwrap();
        assert!(outcome.failed);
        assert!(store.queue().unwrap().is_empty());
        let unnamed = signed("");
        let outcome = unnamed
            .take(
                &store,
                "octocat",
                "Hello-World",
                Take::ForkDownload,
                Some("/tmp/peti-unnamed"),
                |_, _, _, _| panic!("empty login makes no request"),
                || {},
            )
            .unwrap();
        assert!(outcome.failed);
        assert!(store.queue().unwrap().is_empty());
        let broken = Session::new(BrokenSecrets, CLIENT);
        let err = broken
            .take(
                &store,
                "octocat",
                "Hello-World",
                Take::Fork,
                None,
                |_, _, _, _| panic!("a failed secret load makes no request"),
                || {},
            )
            .unwrap_err();
        assert_eq!(err, LoginError::NotRun);
        store
            .insert_queue("octocat", "Hello-World", "", "/tmp/peti-kept")
            .unwrap();
        logged_out.logout().unwrap();
        let downloaded = logged_out
            .take(
                &store,
                "torvalds",
                "linux",
                Take::Download,
                Some("/tmp/peti-after-logout"),
                |_, _, _, _| panic!("logout then download does not fork"),
                || {},
            )
            .unwrap();
        assert!(downloaded.queued);
        assert!(downloaded.forked.is_empty());
        let rows = store.queue().unwrap();
        assert!(
            rows.iter()
                .any(|row| row.repo == "linux" && row.owner == "torvalds")
        );
        assert!(rows.iter().all(|row| row.owner != "me"));
        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn fork_download_asks_for_a_folder_before_any_request() {
        let session = signed("me");
        let (path, store) = open_store();
        let outcome = session
            .take(
                &store,
                "octocat",
                "Hello-World",
                Take::ForkDownload,
                None,
                |_, _, _, _| panic!("folder is asked before the fork"),
                || {},
            )
            .unwrap();
        assert!(outcome.need_folder);
        assert!(!outcome.queued);
        assert!(!outcome.failed);
        assert!(store.queue().unwrap().is_empty());
        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_hostile_fork_page_is_not_a_fork_url() {
        assert!(fork_request_allowed(
            "POST",
            "https://api.github.com/repos/octocat/Hello-World/forks"
        ));
        assert!(fork_request_allowed(
            "GET",
            "https://api.github.com/repos/me/Hello-World"
        ));
        for raw in [
            DEVICE_CODE_URL,
            TOKEN_URL,
            USER_URL,
            "https://github.com/octocat/Hello-World/archive/HEAD.zip",
            "https://codeload.github.com/octocat/Hello-World/zip/refs",
            "https://api.github.com/search/repositories",
            "https://evil.test/repos/octocat/Hello-World/forks",
            "https://api.github.com/repos/../Hello-World/forks",
            "https://user:pw@api.github.com/repos/octocat/Hello-World/forks",
            "http://api.github.com/repos/octocat/Hello-World/forks",
            "https://api.github.com/repos/octocat/Hello-World/forks?x=1",
        ] {
            assert!(!fork_request_allowed("POST", raw), "{raw}");
            assert!(
                !login_request_allowed("POST", raw) || raw == DEVICE_CODE_URL || raw == TOKEN_URL,
                "{raw}"
            );
        }
        assert!(!login_request_allowed(
            "POST",
            "https://api.github.com/repos/octocat/Hello-World/forks"
        ));
        let err = fetch_fork("POST", DEVICE_CODE_URL, "{}", Some(TOKEN)).unwrap_err();
        assert_eq!(err, LoginError::NotRun);
        let err = fetch_fork(
            "POST",
            "https://api.github.com/repos/octocat/Hello-World/forks",
            "{}",
            None,
        )
        .unwrap_err();
        assert_eq!(err, LoginError::NotRun);
    }

    #[test]
    fn the_login_page_opener_only_knows_the_device_page() {
        let (_program, args) = login_args();
        let urls: Vec<_> = args
            .iter()
            .copied()
            .filter(|arg| arg.starts_with("http"))
            .collect();
        assert_eq!(urls, vec![DEVICE_PAGE]);
    }

    fn temp_db() -> PathBuf {
        static SEQ: AtomicUsize = AtomicUsize::new(0);
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("peti-github-{}-{nanos}-{n}", std::process::id()))
    }

    fn file_has(path: &Path, needle: &str) -> bool {
        for suffix in ["", "-wal", "-shm"] {
            let candidate = if suffix.is_empty() {
                path.to_path_buf()
            } else {
                PathBuf::from(format!("{}{suffix}", path.display()))
            };
            if let Ok(bytes) = std::fs::read(&candidate) {
                if bytes
                    .windows(needle.len())
                    .any(|window| window == needle.as_bytes())
                {
                    return true;
                }
            }
        }
        false
    }
}
