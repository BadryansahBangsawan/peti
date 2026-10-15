use std::path::Path;
use std::sync::mpsc;
use std::thread;

use crate::download::{self, DownloadError};
use crate::links::{self, ParsedLink, RejectedLink};
use crate::store::{QueueItem, Status, Store, StoreError};

pub struct PasteResult {
    pub queued: Vec<QueueItem>,
    pub rejected: Vec<RejectedLink>,
}

#[derive(Debug)]
pub struct Done {
    pub bytes: i64,
    pub total: Option<i64>,
}

#[derive(Debug)]
pub enum FetchEnd {
    Done(Done),
    Failed(String),
    Paused,
    Cancelled,
}

#[derive(Debug)]
pub enum ResumeEnd {
    Finished(download::Continue),
    Paused,
    Cancelled,
}

pub fn enqueue_paste(store: &Store, paste: &str, dest: &str) -> Result<PasteResult, StoreError> {
    let parsed = links::parse_paste(paste);
    let mut active = store.queue()?;
    let mut queued = Vec::new();
    for link in parsed.accepted {
        if active_duplicate(&active, &link) {
            continue;
        }
        let git_ref = link.git_ref.clone().unwrap_or_default();
        let item = store.insert_queue(&link.owner, &link.repo, &git_ref, dest)?;
        active.push(item.clone());
        queued.push(item);
    }
    Ok(PasteResult {
        queued,
        rejected: parsed.rejected,
    })
}

#[derive(Debug)]
pub enum QueueAsk {
    NeedFolder,
    Queued(Vec<QueueItem>),
}

pub fn enqueue_discovered(
    store: &Store,
    repos: &[(&str, &str)],
    folder: Option<&str>,
) -> Result<QueueAsk, StoreError> {
    if repos.is_empty() {
        return Ok(QueueAsk::Queued(Vec::new()));
    }
    let settings = store.settings()?;
    let dest = match folder.map(str::trim).filter(|value| !value.is_empty()) {
        Some(folder) => {
            store.set_settings(Some(folder), settings.concurrency, &settings.theme)?;
            folder.to_owned()
        }
        None => match settings
            .default_folder
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(folder) => folder.to_owned(),
            None => return Ok(QueueAsk::NeedFolder),
        },
    };
    let mut paste = String::new();
    for (owner, repo) in repos {
        if links::is_name(owner) && links::is_name(repo) {
            paste.push_str("https://github.com/");
            paste.push_str(owner);
            paste.push('/');
            paste.push_str(repo);
            paste.push('\n');
        }
    }
    let result = enqueue_paste(store, &paste, &dest)?;
    Ok(QueueAsk::Queued(result.queued))
}

pub fn download_item(store: &Store, item: &QueueItem) -> FetchEnd {
    let parent = Path::new(&item.dest);
    let git_ref = git_ref_of(item);
    let id = item.id;
    let outcome = download::download_haltable(
        &item.owner,
        &item.repo,
        git_ref,
        parent,
        &mut |progress| {
            if let Ok((bytes, total)) = progress_sizes(progress.bytes, progress.total) {
                let _ = store.save_progress(id, bytes, total);
            }
        },
        &mut |folder| {
            if let Some(path) = folder.to_str() {
                let _ = store.set_folder(id, path);
            }
        },
        &mut || halt_for(store, id),
    );
    match outcome {
        Ok(download::Outcome::Finished(downloaded)) => settle_finished(
            store,
            id,
            parent,
            &item.owner,
            &item.repo,
            downloaded,
        ),
        Ok(download::Outcome::Paused {
            folder,
            bytes,
            total,
        }) => {
            if let Some(path) = folder.to_str() {
                let _ = store.set_folder(id, path);
            }
            if let Ok((bytes, total)) = progress_sizes(bytes, total) {
                let _ = store.save_progress(id, bytes, total);
            }
            if store.get(id).ok().map(|row| row.status) == Some(Status::Berjalan) {
                let _ = store.pause_item(id);
            }
            FetchEnd::Paused
        }
        Ok(download::Outcome::Cancelled) => {
            if store.get(id).ok().map(|row| row.status) == Some(Status::Berjalan) {
                let _ = store.cancel_item(id);
            }
            FetchEnd::Cancelled
        }
        Err(err) => FetchEnd::Failed(err.to_string()),
    }
}

pub fn pause(store: &Store, id: i64) -> Result<(), String> {
    store.pause_item(id).map_err(|err| err.to_string())
}

pub fn dismiss(store: &Store, id: i64) -> Result<(), String> {
    store.dismiss_item(id).map_err(|err| err.to_string())
}

pub fn cancel(store: &Store, id: i64) -> Result<(), String> {
    let item = store.get(id).map_err(|err| err.to_string())?;
    if item.status == Status::Selesai {
        return Err("batal tidak menghapus yang sudah selesai".to_owned());
    }
    if item.status == Status::Dibatalkan {
        return Ok(());
    }
    store.cancel_item(id).map_err(|err| err.to_string())?;
    if let Some(folder) = item.folder.as_deref() {
        download::remove_unfinished(
            Path::new(&item.dest),
            Path::new(folder),
            &item.owner,
            &item.repo,
        )
        .map_err(|err| err.to_string())?;
    }
    Ok(())
}

pub fn retry(store: &Store, id: i64) -> Result<(), String> {
    let item = store.get(id).map_err(|err| err.to_string())?;
    if item.status != Status::Gagal {
        return Err("ulang hanya untuk yang gagal".to_owned());
    }
    if let Some(folder) = item.folder.as_deref() {
        download::remove_unfinished(
            Path::new(&item.dest),
            Path::new(folder),
            &item.owner,
            &item.repo,
        )
        .map_err(|err| err.to_string())?;
    }
    store.retry_item(id).map_err(|err| err.to_string())
}

pub fn resume(store: &Store, id: i64) -> Result<ResumeEnd, String> {
    let item = store.get(id).map_err(|err| err.to_string())?;
    if item.status != Status::Dijeda {
        return Err("lanjut hanya untuk yang dijeda".to_owned());
    }
    let settings = store.settings().map_err(|err| err.to_string())?;
    let running = store
        .queue()
        .map_err(|err| err.to_string())?
        .into_iter()
        .filter(|row| row.status == Status::Berjalan)
        .count();
    if running >= usize::from(settings.concurrency) {
        return Err("slot unduhan penuh".to_owned());
    }
    store
        .set_status(id, Status::Berjalan)
        .map_err(|err| err.to_string())?;
    let git_ref = git_ref_of(&item);
    let outcome = if let Some(folder) = item.folder.clone() {
        download::resume_snapshot(
            &item.owner,
            &item.repo,
            git_ref,
            Path::new(&folder),
            item.total.and_then(|value| u64::try_from(value).ok()),
            &mut |progress| {
                if let Ok((bytes, total)) = progress_sizes(progress.bytes, progress.total) {
                    let _ = store.save_progress(id, bytes, total);
                }
            },
            &mut || halt_for(store, id),
        )
        .map(|end| match end {
            download::ResumeOutcome::Finished { downloaded, how } => {
                Control::Finished { downloaded, how }
            }
            download::ResumeOutcome::Paused { bytes, total, .. } => Control::Paused {
                bytes,
                total,
                folder: None,
            },
            download::ResumeOutcome::Cancelled => Control::Cancelled,
        })
    } else {
        download::download_haltable(
            &item.owner,
            &item.repo,
            git_ref,
            Path::new(&item.dest),
            &mut |progress| {
                if let Ok((bytes, total)) = progress_sizes(progress.bytes, progress.total) {
                    let _ = store.save_progress(id, bytes, total);
                }
            },
            &mut |folder| {
                if let Some(path) = folder.to_str() {
                    let _ = store.set_folder(id, path);
                }
            },
            &mut || halt_for(store, id),
        )
        .map(|end| match end {
            download::Outcome::Finished(downloaded) => Control::Finished {
                downloaded,
                how: download::Continue::Restarted,
            },
            download::Outcome::Paused {
                folder,
                bytes,
                total,
            } => Control::Paused {
                bytes,
                total,
                folder: Some(folder),
            },
            download::Outcome::Cancelled => Control::Cancelled,
        })
    };
    match outcome {
        Err(err) => {
            if store.get(id).ok().map(|row| row.status) == Some(Status::Berjalan) {
                let _ = store.fail_item(id, &err.to_string());
            }
            Err(err.to_string())
        }
        Ok(Control::Finished { downloaded, how }) => {
            if store.get(id).ok().map(|row| row.status) == Some(Status::Berjalan) {
                if let Some(path) = downloaded.folder.to_str() {
                    let _ = store.set_folder(id, path);
                }
                let (bytes, total) = progress_sizes(downloaded.bytes, downloaded.total)?;
                store
                    .finish_item(id, bytes, total)
                    .map_err(|err| err.to_string())?;
                record_finished(store, id).map_err(|err| err.to_string())?;
            }
            Ok(ResumeEnd::Finished(how))
        }
        Ok(Control::Paused {
            bytes,
            total,
            folder,
        }) => {
            if let Some(folder) = folder.as_ref() {
                if let Some(path) = folder.to_str() {
                    let _ = store.set_folder(id, path);
                }
            }
            if let Ok((bytes, total)) = progress_sizes(bytes, total) {
                let _ = store.save_progress(id, bytes, total);
            }
            if store.get(id).ok().map(|row| row.status) == Some(Status::Berjalan) {
                let _ = store.pause_item(id);
            }
            Ok(ResumeEnd::Paused)
        }
        Ok(Control::Cancelled) => {
            if store.get(id).ok().map(|row| row.status) == Some(Status::Berjalan) {
                let _ = store.cancel_item(id);
            }
            Ok(ResumeEnd::Cancelled)
        }
    }
}

enum Control {
    Finished {
        downloaded: download::Downloaded,
        how: download::Continue,
    },
    Paused {
        bytes: u64,
        total: Option<u64>,
        folder: Option<std::path::PathBuf>,
    },
    Cancelled,
}

fn git_ref_of(item: &QueueItem) -> Option<&str> {
    if item.git_ref.is_empty() {
        None
    } else {
        Some(item.git_ref.as_str())
    }
}

fn halt_for(store: &Store, id: i64) -> Option<download::Halt> {
    match store.get(id).ok().map(|row| row.status) {
        Some(Status::Dijeda) => Some(download::Halt::Pause),
        Some(Status::Dibatalkan) => Some(download::Halt::Cancel),
        _ => None,
    }
}

fn settle_finished(
    store: &Store,
    id: i64,
    parent: &Path,
    owner: &str,
    repo: &str,
    downloaded: download::Downloaded,
) -> FetchEnd {
    if store.get(id).ok().map(|row| row.status) == Some(Status::Dibatalkan) {
        let _ = download::remove_unfinished(parent, &downloaded.folder, owner, repo);
        return FetchEnd::Cancelled;
    }
    match progress_sizes(downloaded.bytes, downloaded.total) {
        Ok((bytes, total)) => FetchEnd::Done(Done { bytes, total }),
        Err(message) => FetchEnd::Failed(message),
    }
}

fn progress_sizes(bytes: u64, total: Option<u64>) -> Result<(i64, Option<i64>), String> {
    let bytes = i64::try_from(bytes).map_err(|_| DownloadError::NotSaved.to_string())?;
    let total = total
        .map(i64::try_from)
        .transpose()
        .map_err(|_| DownloadError::NotSaved.to_string())?;
    Ok((bytes, total))
}

pub fn run_available<F>(store: &Store, concurrency: u8, fetch: F) -> Result<(), StoreError>
where
    F: Fn(&QueueItem) -> FetchEnd + Sync,
{
    let limit = concurrency.clamp(1, 8);
    let fetch = &fetch;
    thread::scope(|scope| {
        let (tx, rx) = mpsc::channel::<(i64, FetchEnd)>();
        let mut inflight = 0usize;
        loop {
            while inflight < usize::from(limit) {
                let Some(item) = store.claim_next(limit)? else {
                    break;
                };
                inflight += 1;
                let tx = tx.clone();
                scope.spawn(move || {
                    let result = fetch(&item);
                    let _ = tx.send((item.id, result));
                });
            }
            if inflight == 0 {
                break;
            }
            let (id, result) = rx
                .recv()
                .map_err(|_| StoreError::Invalid("unduhan berhenti"))?;
            inflight -= 1;
            match result {
                FetchEnd::Done(done) => {
                    let status = store.get(id)?.status;
                    // Extract already finished. A pause that landed during that
                    // extract lost the race; keep the folder. Cancel is handled
                    // in settle_finished, which deletes the tree first.
                    if status == Status::Berjalan || status == Status::Dijeda {
                        store.finish_item(id, done.bytes, done.total)?;
                        record_finished(store, id)?;
                    }
                }
                FetchEnd::Failed(message) => {
                    if store.get(id)?.status == Status::Berjalan {
                        store.fail_item(id, &message)?;
                    }
                }
                FetchEnd::Paused | FetchEnd::Cancelled => {}
            }
        }
        drop(tx);
        Ok(())
    })
}

fn record_finished(store: &Store, id: i64) -> Result<(), StoreError> {
    store.archive_finished(id, &finished_stamp()).map(|_| ())
}

fn finished_stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format_utc(secs)
}

fn format_utc(secs: u64) -> String {
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = rem / 3_600;
    let minute = (rem % 3_600) / 60;
    let second = rem % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
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

fn active_duplicate(items: &[QueueItem], link: &ParsedLink) -> bool {
    let git_ref = link.git_ref.as_deref().unwrap_or("");
    items.iter().any(|item| {
        matches!(item.status, Status::Menunggu | Status::Berjalan)
            && item.owner.eq_ignore_ascii_case(&link.owner)
            && item.repo.eq_ignore_ascii_case(&link.repo)
            && item.git_ref == git_ref
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    fn temp_db() -> std::path::PathBuf {
        static SEQ: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "peti-queue-{}-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_file(&path);
        path
    }

    fn ten_links() -> String {
        (1..=10)
            .map(|n| format!("https://github.com/octocat/repo-{n}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn ten_links_in_one_paste_become_ten_items_and_junk_stays_out() {
        let store = Store::open(&temp_db()).unwrap();
        let paste = format!("{}\nnot-a-link\nhttps://gitlab.com/a/b", ten_links());
        let result = enqueue_paste(&store, &paste, "/tmp/peti-out").unwrap();
        assert_eq!(result.queued.len(), 10);
        assert_eq!(result.rejected.len(), 2);
        assert_eq!(result.rejected[0].raw, "not-a-link");
        assert_eq!(result.rejected[0].reason, "Tautan tidak valid, not-a-link");
        assert_eq!(result.rejected[1].raw, "https://gitlab.com/a/b");
        assert_eq!(
            result.rejected[1].reason,
            "Tautan tidak valid, https://gitlab.com/a/b"
        );
        let items = store.queue().unwrap();
        assert_eq!(items.len(), 10);
        assert!(items.iter().all(|item| item.status == Status::Menunggu));
        assert_eq!(items[0].owner, "octocat");
        assert_eq!(items[0].repo, "repo-1");
        assert_eq!(items[0].git_ref, "");
        assert_eq!(items[9].repo, "repo-10");
    }

    #[test]
    fn waiting_or_running_duplicate_is_not_queued_again() {
        let store = Store::open(&temp_db()).unwrap();
        let first = enqueue_paste(
            &store,
            "https://github.com/OctoCat/Hello-World\nhttps://github.com/octocat/Hello-World",
            "/tmp/peti-out",
        )
        .unwrap();
        assert_eq!(first.queued.len(), 1);
        store
            .set_status(first.queued[0].id, Status::Berjalan)
            .unwrap();
        let again = enqueue_paste(
            &store,
            "https://github.com/octocat/Hello-World",
            "/tmp/peti-out",
        )
        .unwrap();
        assert!(again.queued.is_empty());
        assert_eq!(store.queue().unwrap().len(), 1);

        let other_ref = enqueue_paste(
            &store,
            "https://github.com/octocat/Hello-World/tree/feature/one",
            "/tmp/peti-out",
        )
        .unwrap();
        assert_eq!(other_ref.queued.len(), 1);
        assert_eq!(other_ref.queued[0].git_ref, "feature/one");

        store
            .fail_item(first.queued[0].id, "Unduhan terputus. Ulang")
            .unwrap();
        let after_fail =
            enqueue_paste(&store, "https://github.com/octocat/Hello-World", "/tmp/out").unwrap();
        assert_eq!(after_fail.queued.len(), 1);
    }

    #[test]
    fn fourth_item_stays_waiting_until_a_slot_frees() {
        let store = Store::open(&temp_db()).unwrap();
        enqueue_paste(&store, &ten_links(), "/tmp/peti-out").unwrap();
        assert_eq!(store.settings().unwrap().concurrency, 3);
        let mut claimed = Vec::new();
        for _ in 0..3 {
            claimed.push(store.claim_next(3).unwrap().unwrap());
        }
        assert!(store.claim_next(3).unwrap().is_none());
        let waiting = store
            .queue()
            .unwrap()
            .into_iter()
            .filter(|item| item.status == Status::Menunggu)
            .count();
        assert_eq!(waiting, 7);
        store.finish_item(claimed[0].id, 12, None).unwrap();
        let next = store.claim_next(3).unwrap().unwrap();
        assert_eq!(next.repo, "repo-4");
        let running = store
            .queue()
            .unwrap()
            .into_iter()
            .filter(|item| item.status == Status::Berjalan)
            .count();
        assert_eq!(running, 3);
    }

    #[test]
    fn runner_never_has_four_downloads_at_once() {
        let store = Store::open(&temp_db()).unwrap();
        enqueue_paste(&store, &ten_links(), "/tmp/peti-out").unwrap();
        let seen = Arc::new(AtomicUsize::new(0));
        let seen_slot = seen.clone();
        run_available(&store, store.settings().unwrap().concurrency, |_item| {
            let running = store
                .queue()
                .unwrap()
                .into_iter()
                .filter(|item| item.status == Status::Berjalan)
                .count();
            seen_slot.fetch_max(running, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(30));
            FetchEnd::Done(Done {
                bytes: 4,
                total: None,
            })
        })
        .unwrap();
        assert!(seen.load(Ordering::SeqCst) >= 1);
        assert!(seen.load(Ordering::SeqCst) <= 3);
        let items = store.queue().unwrap();
        assert_eq!(items.len(), 10);
        assert!(items.iter().all(|item| item.status == Status::Selesai));
        assert!(items.iter().all(|item| item.total.is_none()));
        assert!(items.iter().all(|item| item.bytes == 4));
    }

    #[test]
    fn reopen_restores_the_queue_and_fails_what_was_running() {
        let path = temp_db();
        {
            let store = Store::open(&path).unwrap();
            enqueue_paste(&store, &ten_links(), "/tmp/peti-out").unwrap();
            for _ in 0..3 {
                store.claim_next(3).unwrap().unwrap();
            }
        }
        let store = Store::open(&path).unwrap();
        let items = store.queue().unwrap();
        assert_eq!(items.len(), 10);
        let failed = items
            .iter()
            .filter(|item| item.status == Status::Gagal)
            .count();
        let waiting = items
            .iter()
            .filter(|item| item.status == Status::Menunggu)
            .count();
        assert_eq!(failed, 3);
        assert_eq!(waiting, 7);
        assert!(
            items
                .iter()
                .filter(|item| item.status == Status::Gagal)
                .all(|item| { item.error.as_deref() == Some("ditutup saat berjalan") })
        );
    }

    #[test]
    fn one_queued_public_repo_finishes_without_a_git_directory() {
        let parent = std::env::temp_dir().join(format!("peti-q-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).unwrap();
        let store = Store::open(&temp_db()).unwrap();
        enqueue_paste(
            &store,
            "https://github.com/octocat/Hello-World",
            parent.to_str().unwrap(),
        )
        .unwrap();
        run_available(&store, 3, |item| download_item(&store, item)).unwrap();
        assert!(store.queue().unwrap().is_empty());
        let rows = store.history().unwrap();
        assert_eq!(rows.len(), 1);
        let folder = parent.join("octocat-Hello-World");
        assert_eq!(rows[0].folder, folder.to_str().unwrap());
        assert_eq!(rows[0].owner, "octocat");
        assert_eq!(rows[0].repo, "Hello-World");
        assert!(rows[0].git_ref.is_empty());
        assert!(!rows[0].finished_at.is_empty());
        let readme = std::fs::read_to_string(folder.join("README")).unwrap();
        assert!(readme.contains("Hello World"));
        assert!(!folder.join(".git").exists());
        let _ = std::fs::remove_dir_all(&parent);
    }

    fn write_readme_zip(path: &std::path::Path) {
        use std::io::Write;
        let file = std::fs::File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        writer
            .start_file("README", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"Hello World\n").unwrap();
        writer.finish().unwrap();
    }

    #[test]
    fn resume_of_a_complete_partial_extracts_without_starting_over() {
        let parent = std::env::temp_dir().join(format!(
            "peti-resume-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).unwrap();
        let folder = parent.join("octocat-Hello-World");
        std::fs::create_dir(&folder).unwrap();
        let partial = download::partial_for(&folder).unwrap();
        write_readme_zip(&partial);
        let len = i64::try_from(std::fs::metadata(&partial).unwrap().len()).unwrap();
        let store = Store::open(&temp_db()).unwrap();
        let item = store
            .insert_queue("octocat", "Hello-World", "", parent.to_str().unwrap())
            .unwrap();
        store.set_status(item.id, Status::Berjalan).unwrap();
        store.pause_item(item.id).unwrap();
        store.set_folder(item.id, folder.to_str().unwrap()).unwrap();
        store.save_progress(item.id, len, Some(len)).unwrap();
        let end = resume(&store, item.id).unwrap();
        assert!(matches!(
            end,
            ResumeEnd::Finished(download::Continue::Range)
        ));
        assert!(download::Continue::Range.notice().is_none());
        assert!(store.get(item.id).is_err());
        let rows = store.history().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].folder, folder.to_str().unwrap());
        assert!(store.queue().unwrap().is_empty());
        let readme = std::fs::read_to_string(folder.join("README")).unwrap();
        assert!(readme.contains("Hello World"));
        assert!(!folder.join(".git").exists());
        assert!(!partial.exists());
        let names = std::fs::read_dir(&parent)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(names, vec![std::ffi::OsString::from("octocat-Hello-World")]);
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn a_finished_extract_is_removed_when_cancel_already_won() {
        let parent = std::env::temp_dir().join(format!(
            "peti-settle-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).unwrap();
        std::fs::write(parent.join("keep.txt"), b"keep").unwrap();
        let folder = parent.join("octocat-Hello-World");
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("README"), b"hi\n").unwrap();
        let store = Store::open(&temp_db()).unwrap();
        let item = store
            .insert_queue("octocat", "Hello-World", "", parent.to_str().unwrap())
            .unwrap();
        store.set_status(item.id, Status::Berjalan).unwrap();
        store
            .set_folder(item.id, folder.to_str().unwrap())
            .unwrap();
        store.cancel_item(item.id).unwrap();
        assert!(folder.is_dir());
        let end = settle_finished(
            &store,
            item.id,
            &parent,
            "octocat",
            "Hello-World",
            download::Downloaded {
                folder: folder.clone(),
                bytes: 3,
                total: Some(3),
            },
        );
        assert!(matches!(end, FetchEnd::Cancelled));
        assert!(!folder.exists());
        assert_eq!(std::fs::read(parent.join("keep.txt")).unwrap(), b"keep");
        assert_eq!(store.get(item.id).unwrap().status, Status::Dibatalkan);
        assert!(store.history().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn a_download_that_finishes_after_a_pause_is_kept() {
        let store = Store::open(&temp_db()).unwrap();
        let parent = std::env::temp_dir().join(format!(
            "peti-late-keep-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).unwrap();
        let folder = parent.join("octocat-Hello-World");
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("README"), b"hi\n").unwrap();
        enqueue_paste(
            &store,
            "https://github.com/octocat/Hello-World",
            parent.to_str().unwrap(),
        )
        .unwrap();
        run_available(&store, 1, |item| {
            store
                .set_folder(item.id, folder.to_str().unwrap())
                .unwrap();
            store.pause_item(item.id).unwrap();
            FetchEnd::Done(Done {
                bytes: 3,
                total: Some(3),
            })
        })
        .unwrap();
        let rows = store.history().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].folder, folder.to_str().unwrap());
        assert!(store.queue().unwrap().is_empty());
        assert_eq!(std::fs::read(folder.join("README")).unwrap(), b"hi\n");
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn cancel_removes_the_unfinished_folder_and_keeps_a_finished_one() {
        let parent = std::env::temp_dir().join(format!(
            "peti-cancel-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).unwrap();
        std::fs::write(parent.join("keep.txt"), b"keep").unwrap();
        let folder = parent.join("octocat-Hello-World");
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("half.txt"), b"half").unwrap();
        let partial = download::partial_for(&folder).unwrap();
        std::fs::write(&partial, b"partial").unwrap();
        let store = Store::open(&temp_db()).unwrap();
        let item = store
            .insert_queue("octocat", "Hello-World", "", parent.to_str().unwrap())
            .unwrap();
        store.set_status(item.id, Status::Berjalan).unwrap();
        store.pause_item(item.id).unwrap();
        store.set_folder(item.id, folder.to_str().unwrap()).unwrap();
        cancel(&store, item.id).unwrap();
        assert!(!folder.exists());
        assert!(!partial.exists());
        assert_eq!(std::fs::read(parent.join("keep.txt")).unwrap(), b"keep");
        assert_eq!(store.get(item.id).unwrap().status, Status::Dibatalkan);

        let finished = parent.join("octocat-Other");
        std::fs::create_dir(&finished).unwrap();
        std::fs::write(finished.join("README"), b"stay").unwrap();
        let done = store
            .insert_queue("octocat", "Other", "", parent.to_str().unwrap())
            .unwrap();
        store.set_status(done.id, Status::Berjalan).unwrap();
        store
            .set_folder(done.id, finished.to_str().unwrap())
            .unwrap();
        store.finish_item(done.id, 4, None).unwrap();
        let err = cancel(&store, done.id).unwrap_err();
        assert_eq!(err, "batal tidak menghapus yang sudah selesai");
        assert_eq!(std::fs::read(finished.join("README")).unwrap(), b"stay");
        assert_eq!(store.get(done.id).unwrap().status, Status::Selesai);
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn dismiss_removes_only_a_cancelled_row() {
        let store = Store::open(&temp_db()).unwrap();
        let cancelled = store
            .insert_queue("octocat", "Hello-World", "", "/tmp/peti-dismiss")
            .unwrap();
        store.cancel_item(cancelled.id).unwrap();
        store
            .insert_history(
                "octocat",
                "Kept",
                "",
                "/tmp/peti-dismiss/kept",
                "2026-10-15 12:00:00 UTC",
            )
            .unwrap();
        dismiss(&store, cancelled.id).unwrap();
        assert!(store.get(cancelled.id).is_err());
        assert_eq!(store.history().unwrap().len(), 1);

        let waiting = store
            .insert_queue("octocat", "Other", "", "/tmp/peti-dismiss")
            .unwrap();
        let err = dismiss(&store, waiting.id).unwrap_err();
        assert_eq!(err, "hapus hanya untuk yang dibatalkan");
        assert_eq!(store.get(waiting.id).unwrap().status, Status::Menunggu);

        let missing = dismiss(&store, 999_999).unwrap_err();
        assert_eq!(missing, "item antrean tidak ada");
    }

    #[test]
    fn retry_only_starts_again_from_failure_and_clears_the_error() {
        let parent = std::env::temp_dir().join(format!(
            "peti-retry-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).unwrap();
        let folder = parent.join("octocat-Hello-World");
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("half.txt"), b"x").unwrap();
        let partial = download::partial_for(&folder).unwrap();
        std::fs::write(&partial, b"zip").unwrap();
        let path = temp_db();
        let store = Store::open(&path).unwrap();
        let item = store
            .insert_queue("octocat", "Hello-World", "", parent.to_str().unwrap())
            .unwrap();
        store.set_status(item.id, Status::Berjalan).unwrap();
        store.set_folder(item.id, folder.to_str().unwrap()).unwrap();
        drop(store);
        let store = Store::open(&path).unwrap();
        let crashed = store.get(item.id).unwrap();
        assert_eq!(crashed.status, Status::Gagal);
        assert_eq!(crashed.error.as_deref(), Some("ditutup saat berjalan"));
        assert_eq!(crashed.folder.as_deref(), Some(folder.to_str().unwrap()));
        retry(&store, item.id).unwrap();
        let again = store.get(item.id).unwrap();
        assert_eq!(again.status, Status::Menunggu);
        assert_eq!(again.bytes, 0);
        assert_eq!(again.total, None);
        assert!(again.error.is_none());
        assert!(again.folder.is_none());
        assert!(!folder.exists());
        assert!(!partial.exists());
        let refused = retry(&store, item.id).unwrap_err();
        assert_eq!(refused, "ulang hanya untuk yang gagal");
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn resume_stays_paused_when_every_slot_is_taken() {
        let store = Store::open(&temp_db()).unwrap();
        store.set_settings(None, 1, "system").unwrap();
        let running = store
            .insert_queue("octocat", "one", "", "/tmp/peti-out")
            .unwrap();
        store.set_status(running.id, Status::Berjalan).unwrap();
        let paused = store
            .insert_queue("octocat", "two", "", "/tmp/peti-out")
            .unwrap();
        store.set_status(paused.id, Status::Berjalan).unwrap();
        pause(&store, paused.id).unwrap();
        let err = resume(&store, paused.id).unwrap_err();
        assert_eq!(err, "slot unduhan penuh");
        assert_eq!(store.get(paused.id).unwrap().status, Status::Dijeda);
        let early = resume(&store, running.id).unwrap_err();
        assert_eq!(early, "lanjut hanya untuk yang dijeda");
    }

    #[test]
    fn a_failed_download_stays_in_the_queue_and_out_of_history() {
        let store = Store::open(&temp_db()).unwrap();
        enqueue_paste(
            &store,
            "https://github.com/octocat/missing",
            "/tmp/peti-out",
        )
        .unwrap();
        run_available(&store, 3, |_item| {
            FetchEnd::Failed("Repositori tidak tersedia".to_owned())
        })
        .unwrap();
        assert!(store.history().unwrap().is_empty());
        let items = store.queue().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, Status::Gagal);
        assert_eq!(items[0].error.as_deref(), Some("Repositori tidak tersedia"));
    }

    #[test]
    fn an_unwritable_folder_is_the_table_message_on_the_row() {
        let store = Store::open(&temp_db()).unwrap();
        let parent = std::env::temp_dir().join(format!(
            "peti-not-a-folder-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_file(&parent);
        std::fs::write(&parent, b"not a directory").unwrap();
        enqueue_paste(
            &store,
            "https://github.com/octocat/Hello-World",
            parent.to_str().unwrap(),
        )
        .unwrap();
        run_available(&store, 1, |item| download_item(&store, item)).unwrap();
        let items = store.queue().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, Status::Gagal);
        assert_eq!(
            items[0].error.as_deref(),
            Some("Folder tidak bisa dipakai, pilih yang lain")
        );
        assert!(items[0].folder.is_none());
        assert!(store.history().unwrap().is_empty());
        assert!(parent.is_file());
        let _ = std::fs::remove_file(&parent);
    }

    #[test]
    fn a_missing_repo_is_the_table_message_on_the_row() {
        let store = Store::open(&temp_db()).unwrap();
        let parent = std::env::temp_dir().join(format!(
            "peti-missing-row-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).unwrap();
        enqueue_paste(
            &store,
            "https://github.com/octocat/peti-missing-repo-zz",
            parent.to_str().unwrap(),
        )
        .unwrap();
        run_available(&store, 1, |item| download_item(&store, item)).unwrap();
        let items = store.queue().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, Status::Gagal);
        assert_eq!(items[0].error.as_deref(), Some("Repositori tidak tersedia"));
        assert!(store.history().unwrap().is_empty());
        assert!(std::fs::read_dir(&parent).unwrap().next().is_none());
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn discovered_repos_join_the_same_queue_and_wait_for_a_slot() {
        let store = Store::open(&temp_db()).unwrap();
        let asked =
            enqueue_discovered(&store, &[("octocat", "one"), ("octocat", "two")], None).unwrap();
        assert!(matches!(asked, QueueAsk::NeedFolder));
        assert!(store.queue().unwrap().is_empty());
        assert!(store.settings().unwrap().default_folder.is_none());
        let queued = enqueue_discovered(
            &store,
            &[
                ("octocat", "one"),
                ("octocat", "two"),
                ("../x", "nope"),
                ("octocat", "one"),
            ],
            Some("  /tmp/peti-discovered  "),
        )
        .unwrap();
        let QueueAsk::Queued(items) = queued else {
            panic!("folder was given");
        };
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.status == Status::Menunggu));
        assert!(items.iter().all(|item| item.git_ref.is_empty()));
        assert!(items.iter().all(|item| item.dest == "/tmp/peti-discovered"));
        let settings = store.settings().unwrap();
        assert_eq!(
            settings.default_folder.as_deref(),
            Some("/tmp/peti-discovered")
        );
        assert_eq!(settings.concurrency, 3);
        assert_eq!(settings.theme, "system");
        let again = enqueue_discovered(
            &store,
            &[
                ("Octocat", "two"),
                ("octocat", "three"),
                ("octocat", "four"),
            ],
            None,
        )
        .unwrap();
        let QueueAsk::Queued(more) = again else {
            panic!("saved folder was not used");
        };
        assert_eq!(more.len(), 2);
        assert_eq!(store.queue().unwrap().len(), 4);
        assert!(store.claim_next(3).unwrap().is_some());
        assert!(store.claim_next(3).unwrap().is_some());
        assert!(store.claim_next(3).unwrap().is_some());
        assert!(store.claim_next(3).unwrap().is_none());
        let waiting = store
            .queue()
            .unwrap()
            .into_iter()
            .filter(|item| item.status == Status::Menunggu)
            .count();
        assert_eq!(waiting, 1);
    }

    #[test]
    fn utc_stamp_is_a_real_calendar_time() {
        assert_eq!(format_utc(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(format_utc(86_400), "1970-01-02 00:00:00 UTC");
        assert_eq!(format_utc(1_709_164_800), "2024-02-29 00:00:00 UTC");
    }
}
