use std::collections::HashMap;
use std::io::Read;
use std::sync::Mutex;
use std::time::Duration;

use crate::links::is_name;

const CACHE_SECS: u64 = 60;
const PAGE_SIZE: u32 = 20;
const MAX_BODY: usize = 1_000_000;
const USER_AGENT: &str = "Peti/0.1.0";
const SEARCH_URL: &str = "https://api.github.com/search/repositories";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    Day,
    Week,
    Month,
}

impl Window {
    pub fn parse(value: &str) -> Self {
        match value {
            "day" => Self::Day,
            "month" => Self::Month,
            _ => Self::Week,
        }
    }

    fn back(self) -> u64 {
        match self {
            Self::Day => 0,
            Self::Week => 7 * 86_400,
            Self::Month => 30 * 86_400,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoCard {
    pub owner: String,
    pub repo: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub stars: u64,
    pub forks: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPage {
    pub repos: Vec<RepoCard>,
    pub remaining: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoverError {
    Limited,
    NotRun,
}

impl std::fmt::Display for DiscoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Limited => f.write_str("Batas GitHub tercapai, coba lagi nanti"),
            Self::NotRun => f.write_str("Pencarian tidak bisa dijalankan."),
        }
    }
}

pub struct SearchCache {
    entries: Mutex<HashMap<String, (u64, SearchPage)>>,
}

impl SearchCache {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    fn get(&self, key: &str, now: u64) -> Option<SearchPage> {
        let entries = self.entries.lock().ok()?;
        let (at, page) = entries.get(key)?;
        if now >= *at && now - *at < CACHE_SECS {
            Some(page.clone())
        } else {
            None
        }
    }

    fn put(&self, key: &str, now: u64, page: SearchPage) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.insert(key.to_owned(), (now, page));
        }
    }
}

pub fn search_query(text: &str, language: &str, topic: &str, window: Window, now: u64) -> String {
    let date = utc_date(now.saturating_sub(window.back()));
    let mut parts = vec!["is:public".to_owned(), format!("created:>={date}")];
    if let Some(language) = language_token(language) {
        parts.push(format!("language:{language}"));
    }
    if let Some(topic) = topic_token(topic) {
        parts.push(format!("topic:{topic}"));
    }
    let text = clean_text(text);
    if !text.is_empty() {
        parts.push(text);
    }
    parts.join(" ")
}

pub fn search_redirect_allowed(url: &reqwest::Url) -> bool {
    url.scheme() == "https"
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty()
        && url.password().is_none()
        && url.host_str() == Some("api.github.com")
}

pub fn read_remaining(value: Option<&str>) -> Option<u64> {
    let raw = value?.trim();
    if raw.is_empty() {
        return None;
    }
    raw.parse().ok()
}

pub fn interpret_search(
    status: u16,
    remaining: Option<u64>,
    body: &str,
) -> Result<SearchPage, DiscoverError> {
    if status == 403 || status == 429 || (status != 200 && remaining == Some(0)) {
        return Err(DiscoverError::Limited);
    }
    if status != 200 {
        return Err(DiscoverError::NotRun);
    }
    if body.len() > MAX_BODY {
        return Err(DiscoverError::NotRun);
    }
    let value: serde_json::Value = serde_json::from_str(body).map_err(|_| DiscoverError::NotRun)?;
    let items = value
        .get("items")
        .and_then(|item| item.as_array())
        .ok_or(DiscoverError::NotRun)?;
    let mut repos = Vec::new();
    for item in items {
        if let Some(card) = read_card(item) {
            repos.push(card);
        }
    }
    Ok(SearchPage { repos, remaining })
}

pub fn load<F>(
    cache: &SearchCache,
    text: &str,
    language: &str,
    topic: &str,
    window: Window,
    now: u64,
    mut fetch: F,
) -> Result<SearchPage, DiscoverError>
where
    F: FnMut(&str) -> Result<(u16, Option<u64>, String), DiscoverError>,
{
    let query = search_query(text, language, topic, window, now);
    if let Some(page) = cache.get(&query, now) {
        return Ok(page);
    }
    let (status, remaining, body) = fetch(&query)?;
    let page = interpret_search(status, remaining, &body)?;
    cache.put(&query, now, page.clone());
    Ok(page)
}

pub fn fetch_github(query: &str) -> Result<(u16, Option<u64>, String), DiscoverError> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .redirect(reqwest::redirect::Policy::custom(search_redirect))
        .connect_timeout(Duration::from_secs(20))
        .https_only(true)
        .build()
        .map_err(|_| DiscoverError::NotRun)?;
    let per_page = PAGE_SIZE.to_string();
    let url = reqwest::Url::parse_with_params(
        SEARCH_URL,
        [
            ("q", query),
            ("sort", "stars"),
            ("order", "desc"),
            ("per_page", per_page.as_str()),
        ],
    )
    .map_err(|_| DiscoverError::NotRun)?;
    let mut response = client
        .get(url)
        .header("accept", "application/vnd.github+json")
        .header("x-github-api-version", "2022-11-28")
        .send()
        .map_err(|_| DiscoverError::NotRun)?;
    let status = response.status().as_u16();
    let remaining = read_remaining(
        response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|value| value.to_str().ok()),
    );
    Ok(taken_body(status, remaining, read_capped(&mut response)))
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

fn taken_body(
    status: u16,
    remaining: Option<u64>,
    bytes: Result<Vec<u8>, ()>,
) -> (u16, Option<u64>, String) {
    let body = match bytes {
        Ok(bytes) => String::from_utf8(bytes).unwrap_or_default(),
        Err(()) => String::new(),
    };
    (status, remaining, body)
}

fn search_redirect(attempt: reqwest::redirect::Attempt<'_>) -> reqwest::redirect::Action {
    if attempt.previous().len() >= 2 || !search_redirect_allowed(attempt.url()) {
        return attempt.error("redirect refused");
    }
    attempt.follow()
}

fn language_token(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.is_empty() || value.len() > 32 {
        return None;
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '#' | '.' | '-'))
    {
        Some(value)
    } else {
        None
    }
}

fn topic_token(value: &str) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() || value.len() > 50 || value.ends_with('-') {
        return None;
    }
    let mut chars = value.chars();
    let first = chars.next()?;
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return None;
    }
    if chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
        Some(value)
    } else {
        None
    }
}

fn clean_text(value: &str) -> String {
    let mut out = String::new();
    let mut space = false;
    for ch in value.chars() {
        if ch.is_whitespace() {
            space = !out.is_empty();
            continue;
        }
        if ch.is_control() {
            continue;
        }
        if space {
            out.push(' ');
            space = false;
        }
        if out.len() + ch.len_utf8() > 256 {
            break;
        }
        out.push(ch);
    }
    out
}

fn read_card(item: &serde_json::Value) -> Option<RepoCard> {
    let owner = item.get("owner")?.get("login")?.as_str()?;
    let repo = item.get("name")?.as_str()?;
    if !is_name(owner) || !is_name(repo) {
        return None;
    }
    let stars = count(item, &["stargazers_count"])?;
    let forks = count(item, &["forks_count", "forks"])?;
    Some(RepoCard {
        owner: owner.to_owned(),
        repo: repo.to_owned(),
        description: text_field(item, "description"),
        language: text_field(item, "language"),
        stars,
        forks,
    })
}

fn count(item: &serde_json::Value, keys: &[&str]) -> Option<u64> {
    for key in keys {
        if let Some(value) = item.get(*key).and_then(|value| value.as_u64()) {
            return Some(value);
        }
    }
    None
}

fn text_field(item: &serde_json::Value, key: &str) -> Option<String> {
    let text = item.get(key)?.as_str()?.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_owned())
    }
}

fn utc_date(secs: u64) -> String {
    let (year, month, day) = civil_from_days(secs / 86_400);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(days: u64) -> (i32, u32, u32) {
    let z = i64::try_from(days).unwrap_or(i64::MAX) + 719_468;
    let era = z / 146_097;
    let doe = u64::try_from(z - era * 146_097).unwrap_or(0);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = i64::try_from(yoe).unwrap_or(0) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    if month <= 2 {
        year += 1;
    }
    (
        i32::try_from(year).unwrap_or(0),
        u32::try_from(month).unwrap_or(0),
        u32::try_from(day).unwrap_or(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEAP: u64 = 1_709_164_800;

    fn sample_body() -> String {
        r#"{"total_count":1,"items":[{"name":"Hello-World","owner":{"login":"octocat"},"description":" hi ","language":"Rust","stargazers_count":12,"forks_count":3},{"name":"../x","owner":{"login":"octocat"},"stargazers_count":1,"forks_count":1},{"name":"no-counts","owner":{"login":"octocat"},"description":"x"}]}"#.to_owned()
    }

    #[test]
    fn windows_are_public_and_use_the_clock_not_a_fixed_list() {
        assert_eq!(utc_date(0), "1970-01-01");
        assert_eq!(utc_date(86_400), "1970-01-02");
        assert_eq!(utc_date(LEAP), "2024-02-29");
        assert_eq!(
            search_query("", "", "", Window::Day, LEAP),
            "is:public created:>=2024-02-29"
        );
        assert_eq!(
            search_query("", "", "", Window::Week, LEAP),
            "is:public created:>=2024-02-22"
        );
        assert_eq!(
            search_query("", "", "", Window::Month, LEAP),
            "is:public created:>=2024-01-30"
        );
        assert_eq!(Window::parse("day"), Window::Day);
        assert_eq!(Window::parse("month"), Window::Month);
        assert_eq!(Window::parse("year"), Window::Week);
        assert_eq!(Window::parse(""), Window::Week);
    }

    #[test]
    fn filters_are_added_and_a_broken_filter_is_not_sent() {
        assert_eq!(
            search_query("rust cli", "C++", "CLI", Window::Week, LEAP),
            "is:public created:>=2024-02-22 language:C++ topic:cli rust cli"
        );
        let broken = search_query(
            "is:private",
            "Rust OR is:private",
            "bad topic",
            Window::Day,
            LEAP,
        );
        assert_eq!(broken, "is:public created:>=2024-02-29 is:private");
        assert!(!broken.contains("OR"));
        assert!(!broken.contains("bad"));
        assert!(search_query("  a\nb  ", "", "", Window::Day, LEAP).ends_with(" a b"));
    }

    #[test]
    fn search_redirects_stay_on_the_api_host() {
        let allowed = reqwest::Url::parse("https://api.github.com/search/repositories").unwrap();
        assert!(search_redirect_allowed(&allowed));
        for raw in [
            "https://github.com/search",
            "https://codeload.github.com/a/b",
            "https://api.github.com.evil.test/search",
            "http://api.github.com/search",
            "https://user:pw@api.github.com/search",
            "https://api.github.com:444/search",
        ] {
            let url = reqwest::Url::parse(raw).unwrap();
            assert!(!search_redirect_allowed(&url), "{raw}");
        }
    }

    #[test]
    fn quota_is_the_limit_message_and_a_real_empty_page_stays_empty() {
        let limited =
            interpret_search(403, Some(0), r#"{"message":"API rate limit exceeded"}"#).unwrap_err();
        assert_eq!(limited, DiscoverError::Limited);
        assert_eq!(
            limited.to_string(),
            "Batas GitHub tercapai, coba lagi nanti"
        );
        assert_eq!(
            interpret_search(429, None, "slow down").unwrap_err(),
            DiscoverError::Limited
        );
        assert_eq!(
            interpret_search(500, Some(0), "nope").unwrap_err(),
            DiscoverError::Limited
        );
        assert_eq!(
            interpret_search(500, Some(4), "nope").unwrap_err(),
            DiscoverError::NotRun
        );
        assert_eq!(
            interpret_search(200, Some(9), "not-json").unwrap_err(),
            DiscoverError::NotRun
        );
        let empty = interpret_search(200, Some(9), r#"{"items":[]}"#).unwrap();
        assert!(empty.repos.is_empty());
        assert_eq!(empty.remaining, Some(9));
        let empty_at_zero = interpret_search(200, Some(0), r#"{"items":[]}"#).unwrap();
        assert!(empty_at_zero.repos.is_empty());
        assert_eq!(
            interpret_search(200, Some(4), r#"{"total_count":1}"#).unwrap_err(),
            DiscoverError::NotRun
        );
        let zero = interpret_search(
            200,
            None,
            r#"{"items":[{"name":"zero","owner":{"login":"octocat"},"stargazers_count":0,"forks":0},{"name":"floaty","owner":{"login":"octocat"},"stargazers_count":1.5,"forks_count":1}]}"#,
        )
        .unwrap();
        assert_eq!(zero.repos.len(), 1);
        assert_eq!(zero.repos[0].repo, "zero");
        assert_eq!(zero.repos[0].stars, 0);
        assert_eq!(zero.repos[0].forks, 0);
        assert!(zero.remaining.is_none());
        let kept = interpret_search(200, Some(0), &sample_body()).unwrap();
        assert_eq!(kept.remaining, Some(0));
        assert_eq!(kept.repos.len(), 1);
        assert_eq!(kept.repos[0].owner, "octocat");
        assert_eq!(kept.repos[0].repo, "Hello-World");
        assert_eq!(kept.repos[0].description.as_deref(), Some("hi"));
        assert_eq!(kept.repos[0].language.as_deref(), Some("Rust"));
        assert_eq!(kept.repos[0].stars, 12);
        assert_eq!(kept.repos[0].forks, 3);
        assert!(read_remaining(None).is_none());
        assert!(read_remaining(Some("")).is_none());
        assert_eq!(read_remaining(Some(" 8 ")), Some(8));
        assert!(read_remaining(Some("banyak")).is_none());
    }

    #[test]
    fn an_unreadable_limit_body_is_still_the_limit_message() {
        for (status, remaining) in [(403_u16, Some(0)), (429, None), (500, Some(0))] {
            for bytes in [Err(()), Ok(vec![0xff, 0xfe])] {
                let (status, remaining, body) = taken_body(status, remaining, bytes);
                assert_eq!(
                    interpret_search(status, remaining, &body).unwrap_err(),
                    DiscoverError::Limited,
                    "{status}"
                );
            }
            let mut huge = std::io::Cursor::new(vec![b'x'; MAX_BODY + 1]);
            let (status, remaining, body) = taken_body(status, remaining, read_capped(&mut huge));
            assert_eq!(
                interpret_search(status, remaining, &body).unwrap_err(),
                DiscoverError::Limited,
                "{status}"
            );
        }
        let (status, remaining, body) = taken_body(200, Some(0), Err(()));
        assert_eq!(
            interpret_search(status, remaining, &body).unwrap_err(),
            DiscoverError::NotRun
        );
        let cache = SearchCache::new();
        let calls = std::cell::Cell::new(0);
        let mut fetch = |_query: &str| {
            calls.set(calls.get() + 1);
            Ok(taken_body(403, Some(0), Err(())))
        };
        let err = load(&cache, "limited", "", "", Window::Day, LEAP, &mut fetch).unwrap_err();
        assert_eq!(err, DiscoverError::Limited);
        assert_eq!(err.to_string(), "Batas GitHub tercapai, coba lagi nanti");
        let _ = load(&cache, "limited", "", "", Window::Day, LEAP, &mut fetch).unwrap_err();
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn a_repeat_inside_a_minute_does_not_fetch_again_and_a_limit_is_not_cached() {
        let cache = SearchCache::new();
        let calls = std::cell::Cell::new(0);
        let mut fetch = |query: &str| {
            calls.set(calls.get() + 1);
            assert!(query.starts_with("is:public "));
            Ok((200, Some(8), sample_body()))
        };
        let page = load(&cache, "", "", "", Window::Week, LEAP, &mut fetch).unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(page.repos[0].stars, 12);
        let again = load(&cache, "", "", "", Window::Week, LEAP + 59, &mut fetch).unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(again.remaining, Some(8));
        let _ = load(&cache, "", "", "", Window::Week, LEAP + 60, &mut fetch).unwrap();
        assert_eq!(calls.get(), 2);
        let _ = load(&cache, "", "", "", Window::Week, LEAP + 9, &mut fetch).unwrap();
        assert_eq!(calls.get(), 3);

        let mut denied = 0;
        let mut fail = |_query: &str| {
            denied += 1;
            Ok((
                403,
                Some(0),
                r#"{"message":"API rate limit exceeded"}"#.to_owned(),
            ))
        };
        let err = load(&cache, "other", "", "", Window::Day, LEAP, &mut fail).unwrap_err();
        assert_eq!(err, DiscoverError::Limited);
        let _ = load(&cache, "other", "", "", Window::Day, LEAP, &mut fail).unwrap_err();
        assert_eq!(denied, 2);
    }

    #[test]
    fn a_public_search_reads_remaining_from_the_header() {
        let cache = SearchCache::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let page = load(&cache, "", "", "", Window::Week, now, fetch_github).unwrap();
        assert!(!page.repos.is_empty());
        assert!(page.remaining.is_some());
        assert!(!page.repos[0].owner.is_empty());
        assert!(!page.repos[0].repo.is_empty());
        let mut calls = 0;
        let again = load(&cache, "", "", "", Window::Week, now, |query| {
            calls += 1;
            fetch_github(query)
        })
        .unwrap();
        assert_eq!(calls, 0);
        assert_eq!(again.repos.len(), page.repos.len());
    }
}
