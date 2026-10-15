pub mod archive;
pub mod discover;
pub mod download;
pub mod github;
pub mod links;
pub mod queue;
pub mod reveal;
pub mod store;

use std::collections::HashSet;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

static RESUMING: LazyLock<Mutex<HashSet<i64>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

struct Downloads {
    nudge: std::sync::mpsc::Sender<()>,
}

fn wake(downloads: &Downloads) {
    let _ = downloads.nudge.send(());
}

fn serve_queue(store: store::Store, rx: std::sync::mpsc::Receiver<()>) {
    while rx.recv().is_ok() {
        loop {
            while rx.try_recv().is_ok() {}
            let concurrency = store
                .settings()
                .map(|settings| settings.concurrency)
                .unwrap_or(3);
            if queue::run_available(&store, concurrency, |item| {
                queue::download_item(&store, item)
            })
            .is_err()
            {
                break;
            }
            let waiting = store
                .queue()
                .ok()
                .is_some_and(|rows| rows.iter().any(|row| row.status == store::Status::Menunggu));
            let nudged = rx.try_recv().is_ok();
            while rx.try_recv().is_ok() {}
            if !waiting && !nudged {
                break;
            }
        }
    }
}

struct ResumeGuard(i64);

impl Drop for ResumeGuard {
    fn drop(&mut self) {
        if let Ok(mut ids) = RESUMING.lock() {
            ids.remove(&self.0);
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoryRow {
    id: i64,
    owner: String,
    repo: String,
    git_ref: String,
    folder: String,
    finished_at: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct QueueRow {
    id: i64,
    owner: String,
    repo: String,
    git_ref: String,
    status: String,
    error: String,
    bytes: i64,
    total: Option<i64>,
}

#[tauri::command]
fn queue_rows(store: tauri::State<'_, store::Store>) -> Result<Vec<QueueRow>, String> {
    store
        .queue()
        .map(|rows| {
            rows.into_iter()
                .map(|row| QueueRow {
                    id: row.id,
                    owner: row.owner,
                    repo: row.repo,
                    git_ref: row.git_ref,
                    status: row.status.as_str().to_owned(),
                    error: row.error.unwrap_or_default(),
                    bytes: row.bytes,
                    total: row.total,
                })
                .collect()
        })
        .map_err(|err| err.to_string())
}

fn present_history(rows: Vec<store::HistoryItem>) -> Vec<HistoryRow> {
    rows.into_iter()
        .filter(|row| reveal::real_directory(Path::new(&row.folder)).is_ok())
        .map(|row| HistoryRow {
            id: row.id,
            owner: row.owner,
            repo: row.repo,
            git_ref: row.git_ref,
            folder: row.folder,
            finished_at: row.finished_at,
        })
        .collect()
}

#[tauri::command]
fn history_rows(store: tauri::State<'_, store::Store>) -> Result<Vec<HistoryRow>, String> {
    store
        .history()
        .map(present_history)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn open_history_folder(store: tauri::State<'_, store::Store>, id: i64) -> Result<(), String> {
    let path = reveal::stored_directory(&store, id)?;
    reveal::open_directory(&path).map_err(str::to_owned)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscoverRepo {
    owner: String,
    repo: String,
    description: String,
    language: String,
    stars: u64,
    forks: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscoverBody {
    repos: Vec<DiscoverRepo>,
    limited: bool,
    failed: bool,
}

#[derive(serde::Deserialize)]
struct DiscoverPick {
    owner: String,
    repo: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct QueueDiscoverBody {
    queued: usize,
    need_folder: bool,
}

#[tauri::command]
fn queue_discovered(
    store: tauri::State<'_, store::Store>,
    downloads: tauri::State<'_, Downloads>,
    repos: Vec<DiscoverPick>,
    folder: String,
) -> Result<QueueDiscoverBody, String> {
    let picks: Vec<(&str, &str)> = repos
        .iter()
        .map(|repo| (repo.owner.as_str(), repo.repo.as_str()))
        .collect();
    let folder = {
        let trimmed = folder.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    };
    match queue::enqueue_discovered(&store, &picks, folder).map_err(|err| err.to_string())? {
        queue::QueueAsk::NeedFolder => Ok(QueueDiscoverBody {
            queued: 0,
            need_folder: true,
        }),
        queue::QueueAsk::Queued(items) => {
            if !items.is_empty() {
                wake(&downloads);
            }
            Ok(QueueDiscoverBody {
                queued: items.len(),
                need_folder: false,
            })
        }
    }
}

#[tauri::command]
fn discover_search(
    cache: tauri::State<'_, discover::SearchCache>,
    text: String,
    language: String,
    topic: String,
    period: String,
) -> DiscoverBody {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    match discover::load(
        &cache,
        &text,
        &language,
        &topic,
        discover::Window::parse(&period),
        now,
        discover::fetch_github,
    ) {
        Ok(page) => DiscoverBody {
            repos: page
                .repos
                .into_iter()
                .map(|repo| DiscoverRepo {
                    owner: repo.owner,
                    repo: repo.repo,
                    description: repo.description.unwrap_or_default(),
                    language: repo.language.unwrap_or_default(),
                    stars: repo.stars,
                    forks: repo.forks,
                })
                .collect(),
            limited: false,
            failed: false,
        },
        Err(discover::DiscoverError::Limited) => DiscoverBody {
            repos: Vec::new(),
            limited: true,
            failed: false,
        },
        Err(discover::DiscoverError::NotRun) => DiscoverBody {
            repos: Vec::new(),
            limited: false,
            failed: true,
        },
    }
}

#[tauri::command]
fn saved_theme(store: tauri::State<'_, store::Store>) -> Result<String, String> {
    store
        .settings()
        .map(|settings| settings.theme)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn set_saved_theme(store: tauri::State<'_, store::Store>, theme: String) -> Result<(), String> {
    let current = store.settings().map_err(|err| err.to_string())?;
    store
        .set_settings(
            current.default_folder.as_deref(),
            current.concurrency,
            &theme,
        )
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn saved_language(store: tauri::State<'_, store::Store>) -> Result<String, String> {
    store
        .settings()
        .map(|settings| settings.language)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn set_saved_language(
    store: tauri::State<'_, store::Store>,
    language: String,
) -> Result<(), String> {
    store
        .set_language(&language)
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubBody {
    connected: bool,
    login: String,
    user_code: String,
    interval: u64,
}

fn github_body(view: github::LoginView) -> GithubBody {
    GithubBody {
        connected: view.connected,
        login: view.login,
        user_code: view.user_code,
        interval: view.interval,
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[tauri::command]
fn github_status(
    session: tauri::State<'_, github::Session<github::OsSecrets>>,
) -> Result<GithubBody, String> {
    session
        .status()
        .map(github_body)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn github_begin(
    session: tauri::State<'_, github::Session<github::OsSecrets>>,
) -> Result<GithubBody, String> {
    let view = session
        .begin(now_secs(), github::fetch_login)
        .map_err(|err| err.to_string())?;
    if view.open_page {
        github::open_login_page();
    }
    Ok(github_body(view))
}

#[tauri::command]
fn github_poll(
    session: tauri::State<'_, github::Session<github::OsSecrets>>,
) -> Result<GithubBody, String> {
    session
        .poll(now_secs(), github::fetch_login)
        .map(github_body)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn github_logout(
    session: tauri::State<'_, github::Session<github::OsSecrets>>,
) -> Result<GithubBody, String> {
    session
        .logout()
        .map(github_body)
        .map_err(|err| err.to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TakeBody {
    forked: String,
    queued: bool,
    need_folder: bool,
    own_repo: bool,
    failed: bool,
}

#[tauri::command]
fn github_take(
    session: tauri::State<'_, github::Session<github::OsSecrets>>,
    store: tauri::State<'_, store::Store>,
    downloads: tauri::State<'_, Downloads>,
    owner: String,
    repo: String,
    action: String,
    folder: String,
) -> Result<TakeBody, String> {
    let action = match action.as_str() {
        "download" => github::Take::Download,
        "fork" => github::Take::Fork,
        "fork-download" => github::Take::ForkDownload,
        _ => return Err("Fork tidak dibuat.".to_owned()),
    };
    let trimmed = folder.trim();
    let folder = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    };
    session
        .take(
            &store,
            &owner,
            &repo,
            action,
            folder,
            github::fetch_fork,
            || std::thread::sleep(std::time::Duration::from_secs(1)),
        )
        .map(|outcome| {
            if outcome.queued {
                wake(&downloads);
            }
            TakeBody {
                forked: outcome.forked,
                queued: outcome.queued,
                need_folder: outcome.need_folder,
                own_repo: outcome.own_repo,
                failed: outcome.failed,
            }
        })
        .map_err(|err| err.to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RejectedPaste {
    link: String,
    reason: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PasteBody {
    queued: usize,
    need_folder: bool,
    rejected: Vec<RejectedPaste>,
}

#[tauri::command]
fn queue_paste(
    store: tauri::State<'_, store::Store>,
    downloads: tauri::State<'_, Downloads>,
    paste: String,
    folder: String,
) -> Result<PasteBody, String> {
    let settings = store.settings().map_err(|err| err.to_string())?;
    let provided = folder.trim();
    let dest = if provided.is_empty() {
        settings
            .default_folder
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    } else {
        store
            .set_settings(Some(provided), settings.concurrency, &settings.theme)
            .map_err(|err| err.to_string())?;
        Some(provided.to_owned())
    };
    let Some(dest) = dest else {
        return Ok(PasteBody {
            queued: 0,
            need_folder: true,
            rejected: Vec::new(),
        });
    };
    let result = queue::enqueue_paste(&store, &paste, &dest).map_err(|err| err.to_string())?;
    if !result.queued.is_empty() {
        wake(&downloads);
    }
    Ok(PasteBody {
        queued: result.queued.len(),
        need_folder: false,
        rejected: result
            .rejected
            .into_iter()
            .map(|item| RejectedPaste {
                link: item.raw,
                reason: item.reason,
            })
            .collect(),
    })
}

#[tauri::command]
fn queue_pause(store: tauri::State<'_, store::Store>, id: i64) -> Result<(), String> {
    queue::pause(&store, id)
}

#[tauri::command]
fn queue_cancel(store: tauri::State<'_, store::Store>, id: i64) -> Result<(), String> {
    queue::cancel(&store, id)
}

#[tauri::command]
fn queue_retry(
    store: tauri::State<'_, store::Store>,
    downloads: tauri::State<'_, Downloads>,
    id: i64,
) -> Result<(), String> {
    queue::retry(&store, id)?;
    wake(&downloads);
    Ok(())
}

#[tauri::command]
fn queue_resume(store: tauri::State<'_, store::Store>, id: i64) -> Result<(), String> {
    let owned = store.inner().clone();
    let item = owned.get(id).map_err(|err| err.to_string())?;
    if item.status != store::Status::Dijeda {
        return Err("lanjut hanya untuk yang dijeda".to_owned());
    }
    let settings = owned.settings().map_err(|err| err.to_string())?;
    let running = owned
        .queue()
        .map_err(|err| err.to_string())?
        .into_iter()
        .filter(|row| row.status == store::Status::Berjalan)
        .count();
    if running >= usize::from(settings.concurrency) {
        return Err("slot unduhan penuh".to_owned());
    }
    {
        let mut ids = RESUMING
            .lock()
            .map_err(|_| "kunci basis data rusak".to_owned())?;
        if !ids.insert(id) {
            return Ok(());
        }
    }
    std::thread::spawn(move || {
        let _guard = ResumeGuard(id);
        let _ = queue::resume(&owned, id);
    });
    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettingsBody {
    default_folder: String,
    concurrency: u8,
    github_client_id: String,
    language: String,
}

#[tauri::command]
fn app_settings(store: tauri::State<'_, store::Store>) -> Result<AppSettingsBody, String> {
    let settings = store.settings().map_err(|err| err.to_string())?;
    let github_client_id = store.github_client_id().map_err(|err| err.to_string())?;
    Ok(AppSettingsBody {
        default_folder: settings.default_folder.unwrap_or_default(),
        concurrency: settings.concurrency,
        github_client_id,
        language: settings.language,
    })
}

#[tauri::command]
fn set_download_settings(
    store: tauri::State<'_, store::Store>,
    folder: String,
    concurrency: u8,
) -> Result<(), String> {
    let current = store.settings().map_err(|err| err.to_string())?;
    let trimmed = folder.trim();
    let folder = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    };
    store
        .set_settings(folder, concurrency, &current.theme)
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn set_github_client_id(
    store: tauri::State<'_, store::Store>,
    session: tauri::State<'_, github::Session<github::OsSecrets>>,
    client_id: String,
) -> Result<(), String> {
    let trimmed = client_id.trim();
    if !trimmed.is_empty() && !github::client_id_ok(trimmed) {
        return Err(github::LoginError::Unprepared.to_string());
    }
    store
        .set_github_client_id(trimmed)
        .map_err(|err| err.to_string())?;
    session.set_client_id(trimmed);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager;
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db = store::Store::open(&dir.join("peti.db"))?;
            let from_env = std::env::var("PETI_GITHUB_CLIENT_ID").unwrap_or_default();
            let from_env = from_env.trim();
            let client_id = if from_env.is_empty() {
                db.github_client_id().unwrap_or_default()
            } else {
                from_env.to_owned()
            };
            let (tx, rx) = std::sync::mpsc::channel();
            let worker = db.clone();
            std::thread::spawn(move || serve_queue(worker, rx));
            let downloads = Downloads { nudge: tx };
            wake(&downloads);
            app.manage(db);
            app.manage(downloads);
            app.manage(discover::SearchCache::new());
            app.manage(github::Session::new(github::OsSecrets, client_id));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            saved_theme,
            set_saved_theme,
            saved_language,
            set_saved_language,
            history_rows,
            open_history_folder,
            queue_rows,
            queue_paste,
            queue_pause,
            queue_cancel,
            queue_retry,
            queue_resume,
            app_settings,
            set_download_settings,
            set_github_client_id,
            discover_search,
            queue_discovered,
            github_status,
            github_begin,
            github_poll,
            github_logout,
            github_take
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: i64, folder: &str) -> store::HistoryItem {
        store::HistoryItem {
            id,
            owner: "octocat".to_owned(),
            repo: "hello".to_owned(),
            git_ref: String::new(),
            folder: folder.to_owned(),
            finished_at: "2026-10-14T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn a_deleted_download_folder_drops_out_of_completed_history() {
        let root = std::env::temp_dir().join(format!(
            "peti-history-{}",
            std::process::id()
        ));
        let kept = root.join("kept");
        let gone = root.join("gone");
        std::fs::create_dir_all(&kept).unwrap();
        std::fs::create_dir_all(&gone).unwrap();
        std::fs::remove_dir(&gone).unwrap();
        let rows = present_history(vec![
            item(1, kept.to_str().unwrap()),
            item(2, gone.to_str().unwrap()),
            item(3, ""),
        ]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, 1);
        assert_eq!(rows[0].folder, kept.to_str().unwrap());
        let _ = std::fs::remove_dir_all(&root);
    }
}
