use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::archive;
use crate::links::{is_name, is_ref_segment};

const USER_AGENT: &str = "Peti/0.1.0";
const MAX_DOWNLOAD: u64 = 8 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub bytes: u64,
    pub total: Option<u64>,
}

impl Progress {
    pub fn percent(&self) -> Option<u8> {
        let total = self.total.filter(|total| *total > 0)?;
        Some(((self.bytes.saturating_mul(100)) / total).min(100) as u8)
    }

    pub fn bytes_per_second(&self, elapsed_millis: u64) -> Option<u64> {
        if elapsed_millis == 0 {
            return None;
        }
        Some(self.bytes.saturating_mul(1000) / elapsed_millis)
    }
}

#[derive(Debug)]
pub struct Downloaded {
    pub folder: PathBuf,
    pub bytes: u64,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Halt {
    Pause,
    Cancel,
}

#[derive(Debug)]
pub enum Outcome {
    Finished(Downloaded),
    Paused {
        folder: PathBuf,
        bytes: u64,
        total: Option<u64>,
    },
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Continue {
    Range,
    Restarted,
}

impl Continue {
    pub fn notice(self) -> Option<&'static str> {
        match self {
            Continue::Range => None,
            Continue::Restarted => Some("Diunduh ulang dari awal"),
        }
    }
}

#[derive(Debug)]
pub enum ResumeOutcome {
    Finished {
        downloaded: Downloaded,
        how: Continue,
    },
    Paused {
        bytes: u64,
        total: Option<u64>,
        how: Continue,
    },
    Cancelled,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DownloadError {
    InvalidLink(String),
    Unavailable,
    Folder,
    Interrupted,
    NotSaved,
    Limited,
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLink(raw) => write!(f, "Tautan tidak valid, {raw}"),
            Self::Unavailable => write!(f, "Repositori tidak tersedia"),
            Self::Folder => write!(f, "Folder tidak bisa dipakai, pilih yang lain"),
            Self::Interrupted => write!(f, "Unduhan terputus. Ulang"),
            Self::NotSaved => write!(f, "Unduhan tidak disimpan. Ulang"),
            Self::Limited => write!(f, "Batas GitHub tercapai, coba lagi nanti"),
        }
    }
}

impl std::error::Error for DownloadError {}

pub fn redirect_allowed(url: &reqwest::Url) -> bool {
    if url.scheme() != "https" || url.port_or_known_default() != Some(443) {
        return false;
    }
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    matches!(url.host_str(), Some("github.com" | "codeload.github.com"))
}

fn redirect_policy(attempt: reqwest::redirect::Attempt<'_>) -> reqwest::redirect::Action {
    if attempt.previous().len() >= 5 || !redirect_allowed(attempt.url()) {
        return attempt.error("redirect refused");
    }
    attempt.follow()
}

pub fn archive_url(
    owner: &str,
    repo: &str,
    git_ref: Option<&str>,
) -> Result<String, DownloadError> {
    if !is_name(owner) || !is_name(repo) {
        return Err(DownloadError::InvalidLink(format!("{owner}/{repo}")));
    }
    let git_ref = match git_ref {
        None | Some("") => "HEAD",
        Some(value) => {
            if value.split('/').any(|segment| !is_ref_segment(segment)) {
                return Err(DownloadError::InvalidLink(value.to_owned()));
            }
            value
        }
    };
    let encoded = git_ref
        .split('/')
        .map(encode_segment)
        .collect::<Vec<_>>()
        .join("/");
    Ok(format!(
        "https://github.com/{owner}/{repo}/archive/{encoded}.zip"
    ))
}

pub fn allocate_destination(
    parent: &Path,
    owner: &str,
    repo: &str,
) -> Result<PathBuf, DownloadError> {
    if !is_name(owner) || !is_name(repo) {
        return Err(DownloadError::InvalidLink(format!("{owner}/{repo}")));
    }
    let meta = fs::metadata(parent).map_err(|_| DownloadError::Folder)?;
    if !meta.is_dir() {
        return Err(DownloadError::Folder);
    }
    let base = format!("{owner}-{repo}");
    for n in 1..=1000 {
        let name = if n == 1 {
            base.clone()
        } else {
            format!("{base}-{n}")
        };
        let path = parent.join(name);
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(DownloadError::Folder),
        }
    }
    Err(DownloadError::Folder)
}

pub fn partial_for(folder: &Path) -> Result<PathBuf, DownloadError> {
    let parent = folder.parent().ok_or(DownloadError::Folder)?;
    let name = folder
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(DownloadError::Folder)?;
    partial_path(parent, name)
}

pub fn remove_unfinished(
    parent: &Path,
    folder: &Path,
    owner: &str,
    repo: &str,
) -> Result<(), DownloadError> {
    let Some(name) = folder.file_name().and_then(|name| name.to_str()) else {
        return Err(DownloadError::NotSaved);
    };
    if folder.parent() != Some(parent) || !folder_name_matches(name, owner, repo) {
        return Err(DownloadError::NotSaved);
    }
    let partial = partial_path(parent, name)?;
    if partial.parent() != Some(parent) {
        return Err(DownloadError::NotSaved);
    }
    remove_nofollow(&partial)?;
    remove_nofollow(folder)?;
    Ok(())
}

pub fn download_snapshot(
    owner: &str,
    repo: &str,
    git_ref: Option<&str>,
    parent: &Path,
    on_progress: &mut dyn FnMut(Progress),
) -> Result<Downloaded, DownloadError> {
    match download_haltable(
        owner,
        repo,
        git_ref,
        parent,
        on_progress,
        &mut |_| {},
        &mut || None,
    )? {
        Outcome::Finished(downloaded) => Ok(downloaded),
        Outcome::Paused { .. } | Outcome::Cancelled => Err(DownloadError::Interrupted),
    }
}

pub fn download_haltable(
    owner: &str,
    repo: &str,
    git_ref: Option<&str>,
    parent: &Path,
    on_progress: &mut dyn FnMut(Progress),
    on_folder: &mut dyn FnMut(&Path),
    halt: &mut dyn FnMut() -> Option<Halt>,
) -> Result<Outcome, DownloadError> {
    let url = archive_url(owner, repo, git_ref)?;
    let dest = allocate_destination(parent, owner, repo)?;
    on_folder(&dest);
    let mut cleanup = Cleanup {
        dest: dest.clone(),
        partial: None,
        armed: true,
    };
    if let Some(halted) = halt() {
        return halted_outcome(halted, &mut cleanup, &dest, 0, None);
    }
    let (partial, mut file) = create_partial(&dest)?;
    cleanup.partial = Some(partial);
    let fetched = fetch_fresh(&url, &mut file, on_progress, halt)?;
    drop(file);
    conclude(fetched, &mut cleanup, &dest, halt)
}

pub fn resume_snapshot(
    owner: &str,
    repo: &str,
    git_ref: Option<&str>,
    folder: &Path,
    total: Option<u64>,
    on_progress: &mut dyn FnMut(Progress),
    halt: &mut dyn FnMut() -> Option<Halt>,
) -> Result<ResumeOutcome, DownloadError> {
    if !is_name(owner) || !is_name(repo) {
        return Err(DownloadError::InvalidLink(format!("{owner}/{repo}")));
    }
    let name = folder
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(DownloadError::Folder)?;
    if !folder_name_matches(name, owner, repo) {
        return Err(DownloadError::NotSaved);
    }
    let meta = fs::symlink_metadata(folder).map_err(|_| DownloadError::Folder)?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err(DownloadError::Folder);
    }
    let url = archive_url(owner, repo, git_ref)?;
    let partial = partial_for(folder)?;
    if let Ok(meta) = fs::symlink_metadata(&partial) {
        if meta.file_type().is_symlink() || meta.is_dir() {
            return Err(DownloadError::NotSaved);
        }
    }
    let partial_len = fs::metadata(&partial).map(|meta| meta.len()).unwrap_or(0);
    let mut cleanup = Cleanup {
        dest: folder.to_path_buf(),
        partial: Some(partial.clone()),
        armed: true,
    };
    let plan = resume_plan(total, partial_len);
    let (fetched, how) = match plan {
        ResumePlan::Complete => (
            Fetched {
                end: End::Complete(Progress {
                    bytes: partial_len,
                    total,
                }),
                restarted: false,
            },
            Continue::Range,
        ),
        ResumePlan::Restart => {
            if fs::symlink_metadata(&partial).is_ok() {
                remove_nofollow(&partial)?;
            }
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&partial)
                .map_err(|_| DownloadError::Folder)?;
            let fetched = fetch_fresh(&url, &mut file, on_progress, halt)?;
            drop(file);
            (fetched, Continue::Restarted)
        }
        ResumePlan::Range { start } => {
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&partial)
                .map_err(|_| DownloadError::Folder)?;
            let fetched = fetch_ranged(
                &url,
                &mut file,
                start,
                total.unwrap_or(0),
                on_progress,
                halt,
            )?;
            drop(file);
            let how = if fetched.restarted {
                Continue::Restarted
            } else {
                Continue::Range
            };
            (fetched, how)
        }
    };
    match conclude(fetched, &mut cleanup, folder, halt)? {
        Outcome::Finished(downloaded) => Ok(ResumeOutcome::Finished { downloaded, how }),
        Outcome::Paused { bytes, total, .. } => Ok(ResumeOutcome::Paused { bytes, total, how }),
        Outcome::Cancelled => Ok(ResumeOutcome::Cancelled),
    }
}

fn halted_outcome(
    halted: Halt,
    cleanup: &mut Cleanup,
    dest: &Path,
    bytes: u64,
    total: Option<u64>,
) -> Result<Outcome, DownloadError> {
    match halted {
        Halt::Pause => {
            cleanup.armed = false;
            Ok(Outcome::Paused {
                folder: dest.to_path_buf(),
                bytes,
                total,
            })
        }
        Halt::Cancel => Ok(Outcome::Cancelled),
    }
}

fn conclude(
    fetched: Fetched,
    cleanup: &mut Cleanup,
    dest: &Path,
    halt: &mut dyn FnMut() -> Option<Halt>,
) -> Result<Outcome, DownloadError> {
    match fetched.end {
        End::Paused(progress) => {
            halted_outcome(Halt::Pause, cleanup, dest, progress.bytes, progress.total)
        }
        End::Cancelled => Ok(Outcome::Cancelled),
        End::Complete(progress) => {
            if let Some(halted) = halt() {
                return halted_outcome(halted, cleanup, dest, progress.bytes, progress.total);
            }
            if let Some(total) = progress.total {
                if progress.bytes != total {
                    return Err(DownloadError::Interrupted);
                }
            }
            let partial = cleanup.partial.clone().ok_or(DownloadError::NotSaved)?;
            let reader = File::open(&partial).map_err(|_| DownloadError::NotSaved)?;
            if archive::extract_archive(reader, dest).is_err() {
                return Err(DownloadError::NotSaved);
            }
            let _ = fs::remove_file(&partial);
            // A cancel that lands while the archive is being written must still
            // delete the tree. Pause after the bytes are already extracted keeps
            // the folder: the download finished.
            if matches!(halt(), Some(Halt::Cancel)) {
                return Ok(Outcome::Cancelled);
            }
            cleanup.armed = false;
            Ok(Outcome::Finished(Downloaded {
                folder: dest.to_path_buf(),
                bytes: progress.bytes,
                total: progress.total,
            }))
        }
    }
}

enum End {
    Complete(Progress),
    Paused(Progress),
    Cancelled,
}

struct Fetched {
    end: End,
    restarted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResumePlan {
    Range { start: u64 },
    Restart,
    Complete,
}

fn resume_plan(total: Option<u64>, partial_len: u64) -> ResumePlan {
    match total {
        Some(total) if total > 0 && partial_len == total => ResumePlan::Complete,
        Some(total) if total > 0 && partial_len > 0 && partial_len < total => {
            ResumePlan::Range { start: partial_len }
        }
        _ => ResumePlan::Restart,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RangeDecision {
    Append,
    Replace,
    Reject,
}

fn decide_range(
    status: u16,
    content_range: Option<&str>,
    start: u64,
    known_total: u64,
) -> RangeDecision {
    if status == 200 {
        return RangeDecision::Replace;
    }
    if status != 206 {
        return RangeDecision::Reject;
    }
    let Some((got_start, _end, got_total)) = content_range.and_then(parse_content_range) else {
        return RangeDecision::Reject;
    };
    if got_start != start {
        return RangeDecision::Reject;
    }
    match got_total {
        Some(got_total) if got_total == known_total => RangeDecision::Append,
        _ => RangeDecision::Reject,
    }
}

fn parse_content_range(value: &str) -> Option<(u64, u64, Option<u64>)> {
    let rest = value.strip_prefix("bytes ")?;
    let (range, total) = rest.split_once('/')?;
    let (start, end) = range.split_once('-')?;
    if start.is_empty() || end.is_empty() {
        return None;
    }
    let start: u64 = start.parse().ok()?;
    let end: u64 = end.parse().ok()?;
    if end < start {
        return None;
    }
    let total = if total == "*" {
        None
    } else {
        Some(total.parse::<u64>().ok()?)
    };
    if total.is_some_and(|total| end.saturating_add(1) > total) {
        return None;
    }
    Some((start, end, total))
}

fn client() -> Result<reqwest::blocking::Client, DownloadError> {
    reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .redirect(reqwest::redirect::Policy::custom(redirect_policy))
        .connect_timeout(Duration::from_secs(20))
        .https_only(true)
        .build()
        .map_err(|_| DownloadError::Interrupted)
}

fn send(url: &str, range_start: Option<u64>) -> Result<reqwest::blocking::Response, DownloadError> {
    let client = client()?;
    let mut request = client.get(url);
    if let Some(start) = range_start {
        request = request.header(reqwest::header::RANGE, format!("bytes={start}-"));
    }
    request.send().map_err(map_transport)
}

fn fetch_fresh(
    url: &str,
    file: &mut File,
    on_progress: &mut dyn FnMut(Progress),
    halt: &mut dyn FnMut() -> Option<Halt>,
) -> Result<Fetched, DownloadError> {
    if let Some(halted) = halt() {
        let end = match halted {
            Halt::Pause => End::Paused(Progress {
                bytes: 0,
                total: None,
            }),
            Halt::Cancel => End::Cancelled,
        };
        return Ok(Fetched {
            end,
            restarted: false,
        });
    }
    let mut response = send(url, None)?;
    let status = response.status();
    if !status.is_success() {
        return Err(classify(status, response.headers()));
    }
    let total = response.content_length().filter(|total| *total > 0);
    if total.is_some_and(|total| total > MAX_DOWNLOAD) {
        return Err(DownloadError::NotSaved);
    }
    let end = copy_halted(&mut response, file, 0, total, on_progress, halt)?;
    Ok(Fetched {
        end,
        restarted: false,
    })
}

fn fetch_ranged(
    url: &str,
    file: &mut File,
    start: u64,
    known_total: u64,
    on_progress: &mut dyn FnMut(Progress),
    halt: &mut dyn FnMut() -> Option<Halt>,
) -> Result<Fetched, DownloadError> {
    if known_total == 0 || known_total > MAX_DOWNLOAD || start == 0 || start >= known_total {
        return Err(DownloadError::NotSaved);
    }
    let len = file
        .seek(SeekFrom::End(0))
        .map_err(|_| DownloadError::Interrupted)?;
    if len != start {
        return Err(DownloadError::NotSaved);
    }
    let mut response = send(url, Some(start))?;
    let status = response.status().as_u16();
    let header = response
        .headers()
        .get(reqwest::header::CONTENT_RANGE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    match decide_range(status, header.as_deref(), start, known_total) {
        RangeDecision::Append => {
            let end = copy_halted(
                &mut response,
                file,
                start,
                Some(known_total),
                on_progress,
                halt,
            )?;
            Ok(Fetched {
                end,
                restarted: false,
            })
        }
        RangeDecision::Replace => {
            rewind_for_replace(file)?;
            let total = response.content_length().filter(|total| *total > 0);
            if total.is_some_and(|total| total > MAX_DOWNLOAD) {
                return Err(DownloadError::NotSaved);
            }
            let end = copy_halted(&mut response, file, 0, total, on_progress, halt)?;
            Ok(Fetched {
                end,
                restarted: true,
            })
        }
        RangeDecision::Reject => {
            drop(response);
            rewind_for_replace(file)?;
            let mut response = send(url, None)?;
            let status = response.status();
            if !status.is_success() {
                return Err(classify(status, response.headers()));
            }
            let total = response.content_length().filter(|total| *total > 0);
            if total.is_some_and(|total| total > MAX_DOWNLOAD) {
                return Err(DownloadError::NotSaved);
            }
            let end = copy_halted(&mut response, file, 0, total, on_progress, halt)?;
            Ok(Fetched {
                end,
                restarted: true,
            })
        }
    }
}

fn copy_halted<R: Read>(
    reader: &mut R,
    file: &mut File,
    mut bytes: u64,
    total: Option<u64>,
    on_progress: &mut dyn FnMut(Progress),
    halt: &mut dyn FnMut() -> Option<Halt>,
) -> Result<End, DownloadError> {
    let mut buf = [0u8; 64 * 1024];
    loop {
        if let Some(halted) = halt() {
            file.sync_all().map_err(|_| DownloadError::Interrupted)?;
            return Ok(match halted {
                Halt::Pause => End::Paused(Progress { bytes, total }),
                Halt::Cancel => End::Cancelled,
            });
        }
        let read = reader
            .read(&mut buf)
            .map_err(|_| DownloadError::Interrupted)?;
        if read == 0 {
            break;
        }
        file.write_all(&buf[..read])
            .map_err(|_| DownloadError::Folder)?;
        bytes += read as u64;
        if bytes > MAX_DOWNLOAD || total.is_some_and(|total| bytes > total) {
            return Err(DownloadError::NotSaved);
        }
        on_progress(Progress { bytes, total });
    }
    file.sync_all().map_err(|_| DownloadError::Interrupted)?;
    let progress = Progress { bytes, total };
    on_progress(progress.clone());
    Ok(End::Complete(progress))
}

fn rewind_for_replace(file: &mut File) -> Result<(), DownloadError> {
    file.set_len(0).map_err(|_| DownloadError::Folder)?;
    file.seek(SeekFrom::Start(0))
        .map_err(|_| DownloadError::Interrupted)?;
    Ok(())
}

fn classify(status: reqwest::StatusCode, headers: &reqwest::header::HeaderMap) -> DownloadError {
    let limited = status.as_u16() == 429
        || headers
            .get("x-ratelimit-remaining")
            .and_then(|value| value.to_str().ok())
            == Some("0");
    if limited {
        return DownloadError::Limited;
    }
    if matches!(status.as_u16(), 401 | 403 | 404 | 451) {
        return DownloadError::Unavailable;
    }
    DownloadError::Interrupted
}

fn map_transport(err: reqwest::Error) -> DownloadError {
    if err.is_redirect() {
        DownloadError::NotSaved
    } else {
        DownloadError::Interrupted
    }
}

fn create_partial(dest: &Path) -> Result<(PathBuf, File), DownloadError> {
    let parent = dest.parent().ok_or(DownloadError::Folder)?;
    let name = dest
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(DownloadError::Folder)?;
    let path = partial_path(parent, name)?;
    if let Ok(meta) = fs::symlink_metadata(&path) {
        if meta.is_dir() && !meta.file_type().is_symlink() {
            return Err(DownloadError::Folder);
        }
        remove_nofollow(&path)?;
    }
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|_| DownloadError::Folder)?;
    Ok((path, file))
}

fn partial_path(parent: &Path, name: &str) -> Result<PathBuf, DownloadError> {
    if !safe_single_segment(name) {
        return Err(DownloadError::Folder);
    }
    Ok(parent.join(format!(".{name}.peti-partial")))
}

fn folder_name_matches(name: &str, owner: &str, repo: &str) -> bool {
    if !safe_single_segment(name) || !is_name(owner) || !is_name(repo) {
        return false;
    }
    let base = format!("{owner}-{repo}");
    if name == base {
        return true;
    }
    let Some(rest) = name.strip_prefix(&format!("{base}-")) else {
        return false;
    };
    let Ok(n) = rest.parse::<u16>() else {
        return false;
    };
    (2..=1000).contains(&n) && rest == n.to_string()
}

fn safe_single_segment(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
        && !name.contains(':')
}

fn remove_nofollow(path: &Path) -> Result<(), DownloadError> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(DownloadError::Folder),
    };
    if meta.file_type().is_symlink() || !meta.is_dir() {
        return fs::remove_file(path).map_err(|_| DownloadError::Folder);
    }
    let children = fs::read_dir(path)
        .map_err(|_| DownloadError::Folder)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| DownloadError::Folder)?;
    for child in children {
        remove_nofollow(&child.path())?;
    }
    fs::remove_dir(path).map_err(|_| DownloadError::Folder)
}

fn encode_segment(segment: &str) -> String {
    let mut out = String::new();
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

struct Cleanup {
    dest: PathBuf,
    partial: Option<PathBuf>,
    armed: bool,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if let Some(partial) = &self.partial {
            let _ = remove_nofollow(partial);
        }
        let _ = remove_nofollow(&self.dest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_exists_only_when_the_total_is_known() {
        assert_eq!(
            Progress {
                bytes: 50,
                total: Some(200)
            }
            .percent(),
            Some(25)
        );
        assert_eq!(
            Progress {
                bytes: 50,
                total: None
            }
            .percent(),
            None
        );
        assert_eq!(
            Progress {
                bytes: 1,
                total: Some(0)
            }
            .percent(),
            None
        );
        assert_eq!(
            Progress {
                bytes: 1000,
                total: None
            }
            .bytes_per_second(0),
            None
        );
        assert_eq!(
            Progress {
                bytes: 1000,
                total: None
            }
            .bytes_per_second(500),
            Some(2000)
        );
    }

    #[test]
    fn archive_url_stays_on_github_and_rejects_escape() {
        assert_eq!(
            archive_url("octocat", "Hello-World", None).unwrap(),
            "https://github.com/octocat/Hello-World/archive/HEAD.zip"
        );
        assert_eq!(
            archive_url("octocat", "Hello-World", Some("feature/one")).unwrap(),
            "https://github.com/octocat/Hello-World/archive/feature/one.zip"
        );
        assert!(archive_url("..", "Hello-World", None).is_err());
        assert!(archive_url("octocat", "Hello-World", Some("..")).is_err());
    }

    #[test]
    fn redirects_only_to_github_and_codeload() {
        assert!(redirect_allowed(
            &reqwest::Url::parse("https://github.com/octocat/Hello-World").unwrap()
        ));
        assert!(redirect_allowed(
            &reqwest::Url::parse("https://codeload.github.com/octocat/Hello-World/zip/abc")
                .unwrap()
        ));
        for raw in [
            "http://github.com/octocat/Hello-World",
            "https://gist.github.com/octocat/1",
            "https://github.com.evil.test/octocat/Hello-World",
            "https://evil.test/github.com",
            "https://user@github.com/octocat/Hello-World",
            "https://github.com:444/octocat/Hello-World",
        ] {
            assert!(
                !redirect_allowed(&reqwest::Url::parse(raw).unwrap()),
                "{raw}"
            );
        }
    }

    #[test]
    fn error_text_matches_the_table() {
        assert_eq!(
            DownloadError::InvalidLink("nope".to_owned()).to_string(),
            "Tautan tidak valid, nope"
        );
        assert_eq!(
            DownloadError::Unavailable.to_string(),
            "Repositori tidak tersedia"
        );
        assert_eq!(
            DownloadError::Folder.to_string(),
            "Folder tidak bisa dipakai, pilih yang lain"
        );
        assert_eq!(
            DownloadError::Interrupted.to_string(),
            "Unduhan terputus. Ulang"
        );
        assert_eq!(
            DownloadError::NotSaved.to_string(),
            "Unduhan tidak disimpan. Ulang"
        );
        assert_eq!(
            DownloadError::Limited.to_string(),
            "Batas GitHub tercapai, coba lagi nanti"
        );
    }

    #[test]
    fn does_not_overwrite_an_existing_folder() {
        let parent = std::env::temp_dir().join(format!("peti-dest-{}", std::process::id()));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&parent).unwrap();
        let first = parent.join("octocat-Hello-World");
        fs::create_dir(&first).unwrap();
        fs::write(first.join("keep.txt"), b"mine").unwrap();
        let allocated = allocate_destination(&parent, "octocat", "Hello-World").unwrap();
        assert_eq!(allocated, parent.join("octocat-Hello-World-2"));
        assert_eq!(fs::read(first.join("keep.txt")).unwrap(), b"mine");
        assert!(allocate_destination(&parent.join("missing"), "octocat", "Hello-World").is_err());
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn downloads_a_public_repo_without_a_git_directory() {
        let parent = std::env::temp_dir().join(format!("peti-live-{}", std::process::id()));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&parent).unwrap();
        let mut saw_percent_without_total = false;
        let downloaded =
            download_snapshot("octocat", "Hello-World", None, &parent, &mut |progress| {
                if progress.percent().is_some() && progress.total.is_none() {
                    saw_percent_without_total = true;
                }
            })
            .unwrap();
        assert!(!saw_percent_without_total);
        assert_eq!(
            downloaded.folder.file_name().unwrap(),
            "octocat-Hello-World"
        );
        let readme = fs::read_to_string(downloaded.folder.join("README")).unwrap();
        assert!(readme.contains("Hello World"));
        assert!(!downloaded.folder.join(".git").exists());
        assert!(downloaded.bytes > 0);
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn missing_repo_is_unavailable_without_guessing_why() {
        let parent = std::env::temp_dir().join(format!("peti-miss-{}", std::process::id()));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&parent).unwrap();
        let err = download_snapshot(
            "octocat",
            "peti-missing-repo-zz",
            None,
            &parent,
            &mut |_| {},
        )
        .unwrap_err();
        assert_eq!(err, DownloadError::Unavailable);
        assert!(fs::read_dir(&parent).unwrap().next().is_none());
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn resume_plan_uses_range_only_when_a_total_and_a_partial_exist() {
        assert_eq!(resume_plan(None, 40), ResumePlan::Restart);
        assert_eq!(resume_plan(Some(0), 40), ResumePlan::Restart);
        assert_eq!(resume_plan(Some(100), 0), ResumePlan::Restart);
        assert_eq!(resume_plan(Some(100), 140), ResumePlan::Restart);
        assert_eq!(resume_plan(Some(100), 40), ResumePlan::Range { start: 40 });
        assert_eq!(resume_plan(Some(100), 100), ResumePlan::Complete);
        assert_eq!(
            Continue::Restarted.notice(),
            Some("Diunduh ulang dari awal")
        );
        assert_eq!(Continue::Range.notice(), None);
    }

    #[test]
    fn range_response_appends_only_a_matching_206() {
        assert_eq!(
            decide_range(206, Some("bytes 10-99/100"), 10, 100),
            RangeDecision::Append
        );
        assert_eq!(
            decide_range(206, Some("bytes 0-99/100"), 10, 100),
            RangeDecision::Reject
        );
        assert_eq!(
            decide_range(206, Some("bytes 10-99/999"), 10, 100),
            RangeDecision::Reject
        );
        assert_eq!(
            decide_range(206, Some("bytes 10-99/*"), 10, 100),
            RangeDecision::Reject
        );
        assert_eq!(decide_range(206, None, 10, 100), RangeDecision::Reject);
        assert_eq!(
            decide_range(200, Some("bytes 10-99/100"), 10, 100),
            RangeDecision::Replace
        );
        assert_eq!(decide_range(416, None, 10, 100), RangeDecision::Reject);
    }

    #[test]
    fn replacing_a_partial_does_not_keep_the_old_bytes() {
        let path = std::env::temp_dir().join(format!("peti-rewind-{}", std::process::id()));
        let _ = fs::remove_file(&path);
        fs::write(&path, b"OLD").unwrap();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        rewind_for_replace(&mut file).unwrap();
        file.write_all(b"NEW").unwrap();
        drop(file);
        assert_eq!(fs::read(&path).unwrap(), b"NEW");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn pause_keeps_bytes_already_written() {
        struct OneByte {
            left: usize,
        }
        impl Read for OneByte {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                if self.left == 0 || buf.is_empty() {
                    return Ok(0);
                }
                buf[0] = 7;
                self.left -= 1;
                Ok(1)
            }
        }
        let mut reader = OneByte { left: 30 };
        let path = std::env::temp_dir().join(format!("peti-pause-{}", std::process::id()));
        let _ = fs::remove_file(&path);
        let mut file = File::create(&path).unwrap();
        let mut calls = 0u32;
        let end = copy_halted(
            &mut reader,
            &mut file,
            0,
            Some(30),
            &mut |_| {},
            &mut || {
                calls += 1;
                if calls > 1 { Some(Halt::Pause) } else { None }
            },
        )
        .unwrap();
        drop(file);
        match end {
            End::Paused(progress) => {
                assert!(progress.bytes > 0);
                assert!(progress.bytes < 30);
                assert_eq!(progress.total, Some(30));
                assert_eq!(fs::metadata(&path).unwrap().len(), progress.bytes);
            }
            _ => panic!("pause should stop before the reader ends"),
        }
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn pause_before_any_byte_does_not_extract() {
        let parent = std::env::temp_dir().join(format!("peti-hold-{}", std::process::id()));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&parent).unwrap();
        let outcome = download_haltable(
            "octocat",
            "Hello-World",
            None,
            &parent,
            &mut |_| {},
            &mut |_| {},
            &mut || Some(Halt::Pause),
        )
        .unwrap();
        match outcome {
            Outcome::Paused { bytes, folder, .. } => {
                assert_eq!(bytes, 0);
                assert!(folder.is_dir());
                assert!(!folder.join("README").exists());
                assert_eq!(folder, parent.join("octocat-Hello-World"));
            }
            _ => panic!("pause before the request must not finish"),
        }
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn cancel_before_any_byte_leaves_the_parent_empty() {
        let parent = std::env::temp_dir().join(format!("peti-stop-{}", std::process::id()));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&parent).unwrap();
        fs::write(parent.join("keep.txt"), b"keep").unwrap();
        let outcome = download_haltable(
            "octocat",
            "Hello-World",
            None,
            &parent,
            &mut |_| {},
            &mut |_| {},
            &mut || Some(Halt::Cancel),
        )
        .unwrap();
        assert!(matches!(outcome, Outcome::Cancelled));
        assert!(!parent.join("octocat-Hello-World").exists());
        assert_eq!(fs::read(parent.join("keep.txt")).unwrap(), b"keep");
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn cancel_drop_removes_only_the_unfinished_folder() {
        let parent = std::env::temp_dir().join(format!("peti-drop-{}", std::process::id()));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&parent).unwrap();
        fs::write(parent.join("keep.txt"), b"keep").unwrap();
        let dest = parent.join("octocat-Hello-World");
        fs::create_dir(&dest).unwrap();
        let partial = partial_for(&dest).unwrap();
        fs::write(&partial, b"abc").unwrap();
        {
            let _cleanup = Cleanup {
                dest: dest.clone(),
                partial: Some(partial.clone()),
                armed: true,
            };
        }
        assert!(!dest.exists());
        assert!(!partial.exists());
        assert_eq!(fs::read(parent.join("keep.txt")).unwrap(), b"keep");
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn remove_unfinished_refuses_a_path_outside_the_parent() {
        let parent = std::env::temp_dir().join(format!("peti-rm-{}", std::process::id()));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&parent).unwrap();
        fs::write(parent.join("keep.txt"), b"keep").unwrap();
        let elsewhere = std::env::temp_dir().join(format!("peti-rm-out-{}", std::process::id()));
        let _ = fs::remove_dir_all(&elsewhere);
        fs::create_dir_all(&elsewhere).unwrap();
        fs::write(elsewhere.join("secret.txt"), b"secret").unwrap();
        assert!(remove_unfinished(&parent, &elsewhere, "octocat", "Hello-World").is_err());
        assert_eq!(fs::read(elsewhere.join("secret.txt")).unwrap(), b"secret");
        assert_eq!(fs::read(parent.join("keep.txt")).unwrap(), b"keep");
        assert!(remove_unfinished(&parent, &parent, "octocat", "Hello-World").is_err());

        #[cfg(unix)]
        {
            let link = parent.join("octocat-Hello-World");
            std::os::unix::fs::symlink(&elsewhere, &link).unwrap();
            remove_unfinished(&parent, &link, "octocat", "Hello-World").unwrap();
            assert!(!link.exists());
            assert_eq!(fs::read(elsewhere.join("secret.txt")).unwrap(), b"secret");
        }
        let _ = fs::remove_dir_all(&parent);
        let _ = fs::remove_dir_all(&elsewhere);
    }

    #[cfg(unix)]
    #[test]
    fn cancel_does_not_follow_a_symlink_inside_the_unfinished_folder() {
        let parent = std::env::temp_dir().join(format!("peti-inner-{}", std::process::id()));
        let outside = std::env::temp_dir().join(format!("peti-inner-out-{}", std::process::id()));
        let _ = fs::remove_dir_all(&parent);
        let _ = fs::remove_dir_all(&outside);
        fs::create_dir_all(&parent).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("secret.txt"), b"secret").unwrap();
        let folder = parent.join("octocat-Hello-World");
        fs::create_dir(&folder).unwrap();
        std::os::unix::fs::symlink(&outside, folder.join("link")).unwrap();
        remove_unfinished(&parent, &folder, "octocat", "Hello-World").unwrap();
        assert!(!folder.exists());
        assert_eq!(fs::read(outside.join("secret.txt")).unwrap(), b"secret");
        let _ = fs::remove_dir_all(&parent);
        let _ = fs::remove_dir_all(&outside);
    }

    fn readme_zip() -> Vec<u8> {
        use std::io::Cursor;
        use zip::ZipWriter;
        use zip::write::SimpleFileOptions;
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut cursor);
            writer
                .start_file("Hello-World-main/README", SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"hi\n").unwrap();
            writer.finish().unwrap();
        }
        cursor.into_inner()
    }

    #[test]
    fn cancel_after_extract_removes_the_new_tree() {
        let parent = std::env::temp_dir().join(format!(
            "peti-late-cancel-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&parent).unwrap();
        fs::write(parent.join("keep.txt"), b"keep").unwrap();
        let dest = parent.join("octocat-Hello-World");
        fs::create_dir(&dest).unwrap();
        let bytes = readme_zip();
        let partial = partial_for(&dest).unwrap();
        fs::write(&partial, &bytes).unwrap();
        let mut calls = 0u8;
        let mut cleanup = Cleanup {
            dest: dest.clone(),
            partial: Some(partial.clone()),
            armed: true,
        };
        let outcome = conclude(
            Fetched {
                end: End::Complete(Progress {
                    bytes: bytes.len() as u64,
                    total: Some(bytes.len() as u64),
                }),
                restarted: false,
            },
            &mut cleanup,
            &dest,
            &mut || {
                calls += 1;
                if calls >= 2 {
                    Some(Halt::Cancel)
                } else {
                    None
                }
            },
        )
        .unwrap();
        assert!(matches!(outcome, Outcome::Cancelled));
        assert!(cleanup.armed);
        drop(cleanup);
        assert!(!dest.exists());
        assert!(!partial.exists());
        assert_eq!(fs::read(parent.join("keep.txt")).unwrap(), b"keep");
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn pause_after_extract_keeps_the_finished_tree() {
        let parent = std::env::temp_dir().join(format!(
            "peti-late-pause-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&parent).unwrap();
        let dest = parent.join("octocat-Hello-World");
        fs::create_dir(&dest).unwrap();
        let bytes = readme_zip();
        let partial = partial_for(&dest).unwrap();
        fs::write(&partial, &bytes).unwrap();
        let mut calls = 0u8;
        let mut cleanup = Cleanup {
            dest: dest.clone(),
            partial: Some(partial.clone()),
            armed: true,
        };
        let outcome = conclude(
            Fetched {
                end: End::Complete(Progress {
                    bytes: bytes.len() as u64,
                    total: Some(bytes.len() as u64),
                }),
                restarted: false,
            },
            &mut cleanup,
            &dest,
            &mut || {
                calls += 1;
                if calls >= 2 {
                    Some(Halt::Pause)
                } else {
                    None
                }
            },
        )
        .unwrap();
        match outcome {
            Outcome::Finished(downloaded) => {
                assert_eq!(downloaded.folder, dest);
                assert_eq!(fs::read(dest.join("README")).unwrap(), b"hi\n");
            }
            other => panic!("pause after extract must keep the tree, got {other:?}"),
        }
        assert!(!cleanup.armed);
        drop(cleanup);
        assert!(dest.join("README").is_file());
        assert!(!partial.exists());
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn resume_against_github_restarts_when_range_is_ignored() {
        let parent = std::env::temp_dir().join(format!("peti-range-{}", std::process::id()));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&parent).unwrap();
        let folder = parent.join("octocat-Hello-World");
        fs::create_dir(&folder).unwrap();
        let partial = partial_for(&folder).unwrap();
        fs::write(&partial, b"not-a-complete-zip").unwrap();
        let outcome = resume_snapshot(
            "octocat",
            "Hello-World",
            None,
            &folder,
            Some(50),
            &mut |_| {},
            &mut || None,
        )
        .unwrap();
        match outcome {
            ResumeOutcome::Finished { downloaded, how } => {
                assert_eq!(how, Continue::Restarted);
                assert_eq!(how.notice(), Some("Diunduh ulang dari awal"));
                assert_eq!(downloaded.folder, folder);
                assert!(downloaded.bytes > 16);
                let readme = fs::read_to_string(folder.join("README")).unwrap();
                assert!(readme.contains("Hello World"));
                assert!(!folder.join(".git").exists());
                assert!(!partial.exists());
            }
            other => panic!("resume should finish, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&parent);
    }
}
