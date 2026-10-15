use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OptionalExtension, params};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS queue (
    id INTEGER PRIMARY KEY,
    owner TEXT NOT NULL,
    repo TEXT NOT NULL,
    git_ref TEXT NOT NULL,
    dest TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('menunggu','berjalan','dijeda','selesai','gagal','dibatalkan')),
    bytes INTEGER NOT NULL DEFAULT 0 CHECK (bytes >= 0),
    total INTEGER CHECK (total IS NULL OR total >= 0),
    error TEXT,
    folder TEXT
);
CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY,
    owner TEXT NOT NULL,
    repo TEXT NOT NULL,
    git_ref TEXT NOT NULL,
    folder TEXT NOT NULL,
    finished_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    default_folder TEXT,
    concurrency INTEGER NOT NULL DEFAULT 3 CHECK (concurrency BETWEEN 1 AND 8),
    theme TEXT NOT NULL DEFAULT 'system' CHECK (theme IN ('system','light','dark')),
    github_client_id TEXT,
    language TEXT NOT NULL DEFAULT 'id'
);
INSERT INTO settings (id, concurrency, theme)
VALUES (1, 3, 'system')
ON CONFLICT (id) DO NOTHING;
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Menunggu,
    Berjalan,
    Dijeda,
    Selesai,
    Gagal,
    Dibatalkan,
}

impl Status {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Status::Menunggu => "menunggu",
            Status::Berjalan => "berjalan",
            Status::Dijeda => "dijeda",
            Status::Selesai => "selesai",
            Status::Gagal => "gagal",
            Status::Dibatalkan => "dibatalkan",
        }
    }

    fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "menunggu" => Ok(Status::Menunggu),
            "berjalan" => Ok(Status::Berjalan),
            "dijeda" => Ok(Status::Dijeda),
            "selesai" => Ok(Status::Selesai),
            "gagal" => Ok(Status::Gagal),
            "dibatalkan" => Ok(Status::Dibatalkan),
            _ => Err(StoreError::Invalid("status antrean tidak dikenal")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueItem {
    pub id: i64,
    pub owner: String,
    pub repo: String,
    pub git_ref: String,
    pub dest: String,
    pub status: Status,
    pub bytes: i64,
    pub total: Option<i64>,
    pub error: Option<String>,
    pub folder: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryItem {
    pub id: i64,
    pub owner: String,
    pub repo: String,
    pub git_ref: String,
    pub folder: String,
    pub finished_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub default_folder: Option<String>,
    pub concurrency: u8,
    pub theme: String,
    pub language: String,
}

#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Invalid(&'static str),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Sqlite(err) => write!(f, "{err}"),
            StoreError::Invalid(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(err: rusqlite::Error) -> Self {
        StoreError::Sqlite(err)
    }
}

#[derive(Clone)]
pub struct Store {
    conn: Arc<Mutex<Connection>>,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        ensure_columns(&conn)?;
        conn.execute(
            "UPDATE queue SET status = 'gagal', error = 'ditutup saat berjalan' WHERE status = 'berjalan'",
            [],
        )?;
        Ok(Store {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn insert_queue(
        &self,
        owner: &str,
        repo: &str,
        git_ref: &str,
        dest: &str,
    ) -> Result<QueueItem, StoreError> {
        require_text(owner, "pemilik kosong")?;
        require_text(repo, "repo kosong")?;
        require_text(dest, "folder tujuan kosong")?;
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO queue (owner, repo, git_ref, dest, status, bytes, total, error)
             VALUES (?1, ?2, ?3, ?4, 'menunggu', 0, NULL, NULL)",
            params![owner, repo, git_ref, dest],
        )?;
        let id = conn.last_insert_rowid();
        read_queue(&conn, id)
    }

    pub fn queue(&self) -> Result<Vec<QueueItem>, StoreError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, owner, repo, git_ref, dest, status, bytes, total, error, folder
             FROM queue ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })?;
        let mut items = Vec::new();
        for row in rows {
            let (id, owner, repo, git_ref, dest, status, bytes, total, error, folder) = row?;
            items.push(QueueItem {
                id,
                owner,
                repo,
                git_ref,
                dest,
                status: Status::parse(&status)?,
                bytes,
                total,
                error,
                folder,
            });
        }
        Ok(items)
    }

    pub fn get(&self, id: i64) -> Result<QueueItem, StoreError> {
        let conn = self.lock()?;
        read_queue(&conn, id)
    }

    pub fn set_folder(&self, id: i64, folder: &str) -> Result<(), StoreError> {
        require_text(folder, "folder unduhan kosong")?;
        let conn = self.lock()?;
        conn.execute(
            "UPDATE queue SET folder = ?1
             WHERE id = ?2 AND status IN ('berjalan','dijeda')",
            params![folder, id],
        )?;
        Ok(())
    }

    pub fn save_progress(&self, id: i64, bytes: i64, total: Option<i64>) -> Result<(), StoreError> {
        if bytes < 0 || total.is_some_and(|value| value < 0) {
            return Err(StoreError::Invalid("ukuran unduhan rusak"));
        }
        let conn = self.lock()?;
        conn.execute(
            "UPDATE queue SET bytes = ?1, total = ?2
             WHERE id = ?3 AND status IN ('berjalan','dijeda')",
            params![bytes, total, id],
        )?;
        Ok(())
    }

    pub fn pause_item(&self, id: i64) -> Result<(), StoreError> {
        let conn = self.lock()?;
        let changed = conn.execute(
            "UPDATE queue SET status = 'dijeda' WHERE id = ?1 AND status = 'berjalan'",
            [id],
        )?;
        if changed == 0 {
            return Err(StoreError::Invalid("jeda hanya untuk yang berjalan"));
        }
        Ok(())
    }

    pub fn cancel_item(&self, id: i64) -> Result<(), StoreError> {
        let conn = self.lock()?;
        let status: Option<String> = conn
            .query_row("SELECT status FROM queue WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()?;
        match status.as_deref() {
            None => return Err(StoreError::Invalid("item antrean tidak ada")),
            Some("selesai") => {
                return Err(StoreError::Invalid(
                    "batal tidak menghapus yang sudah selesai",
                ));
            }
            Some("dibatalkan") => return Ok(()),
            Some("menunggu" | "berjalan" | "dijeda" | "gagal") => {}
            Some(_) => return Err(StoreError::Invalid("status antrean tidak dikenal")),
        }
        conn.execute(
            "UPDATE queue
             SET status = 'dibatalkan', bytes = 0, total = NULL, error = NULL, folder = NULL
             WHERE id = ?1 AND status IN ('menunggu','berjalan','dijeda','gagal')",
            [id],
        )?;
        Ok(())
    }

    pub fn dismiss_item(&self, id: i64) -> Result<(), StoreError> {
        let conn = self.lock()?;
        let changed = conn.execute(
            "DELETE FROM queue WHERE id = ?1 AND status = 'dibatalkan'",
            [id],
        )?;
        if changed == 1 {
            return Ok(());
        }
        let status: Option<String> = conn
            .query_row("SELECT status FROM queue WHERE id = ?1", [id], |row| row.get(0))
            .optional()?;
        match status {
            None => Err(StoreError::Invalid("item antrean tidak ada")),
            Some(_) => Err(StoreError::Invalid("hapus hanya untuk yang dibatalkan")),
        }
    }

    pub fn retry_item(&self, id: i64) -> Result<(), StoreError> {
        let conn = self.lock()?;
        let changed = conn.execute(
            "UPDATE queue
             SET status = 'menunggu', bytes = 0, total = NULL, error = NULL, folder = NULL
             WHERE id = ?1 AND status = 'gagal'",
            [id],
        )?;
        if changed == 0 {
            return Err(StoreError::Invalid("ulang hanya untuk yang gagal"));
        }
        Ok(())
    }

    pub fn claim_next(&self, concurrency: u8) -> Result<Option<QueueItem>, StoreError> {
        if concurrency == 0 {
            return Ok(None);
        }
        let conn = self.lock()?;
        let id = conn
            .query_row(
                "UPDATE queue SET status = 'berjalan'
                 WHERE id = (
                     SELECT id FROM queue
                     WHERE status = 'menunggu'
                       AND (SELECT COUNT(*) FROM queue WHERE status = 'berjalan') < ?1
                     ORDER BY id
                     LIMIT 1
                 )
                 RETURNING id",
                [i64::from(concurrency)],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        let Some(id) = id else {
            return Ok(None);
        };
        read_queue(&conn, id).map(Some)
    }

    pub fn finish_item(&self, id: i64, bytes: i64, total: Option<i64>) -> Result<(), StoreError> {
        if bytes < 0 || total.is_some_and(|value| value < 0) {
            return Err(StoreError::Invalid("ukuran unduhan rusak"));
        }
        let conn = self.lock()?;
        let changed = conn.execute(
            "UPDATE queue
             SET status = 'selesai', bytes = ?1, total = ?2, error = NULL
             WHERE id = ?3",
            params![bytes, total, id],
        )?;
        if changed == 0 {
            return Err(StoreError::Invalid("item antrean tidak ada"));
        }
        Ok(())
    }

    pub fn fail_item(&self, id: i64, error: &str) -> Result<(), StoreError> {
        require_text(error, "alasan gagal kosong")?;
        let conn = self.lock()?;
        let changed = conn.execute(
            "UPDATE queue SET status = 'gagal', error = ?1 WHERE id = ?2",
            params![error, id],
        )?;
        if changed == 0 {
            return Err(StoreError::Invalid("item antrean tidak ada"));
        }
        Ok(())
    }

    pub fn set_status(&self, id: i64, status: Status) -> Result<(), StoreError> {
        let conn = self.lock()?;
        let changed = conn.execute(
            "UPDATE queue SET status = ?1 WHERE id = ?2",
            params![status.as_str(), id],
        )?;
        if changed == 0 {
            return Err(StoreError::Invalid("item antrean tidak ada"));
        }
        Ok(())
    }

    pub fn insert_history(
        &self,
        owner: &str,
        repo: &str,
        git_ref: &str,
        folder: &str,
        finished_at: &str,
    ) -> Result<HistoryItem, StoreError> {
        require_text(owner, "pemilik kosong")?;
        require_text(repo, "repo kosong")?;
        require_text(folder, "folder riwayat kosong")?;
        require_text(finished_at, "waktu selesai kosong")?;
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO history (owner, repo, git_ref, folder, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![owner, repo, git_ref, folder, finished_at],
        )?;
        let id = conn.last_insert_rowid();
        Ok(HistoryItem {
            id,
            owner: owner.to_owned(),
            repo: repo.to_owned(),
            git_ref: git_ref.to_owned(),
            folder: folder.to_owned(),
            finished_at: finished_at.to_owned(),
        })
    }

    pub fn archive_finished(
        &self,
        id: i64,
        finished_at: &str,
    ) -> Result<Option<HistoryItem>, StoreError> {
        require_text(finished_at, "waktu selesai kosong")?;
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let row = tx
            .query_row(
                "SELECT owner, repo, git_ref, status, folder FROM queue WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .optional()?;
        let Some((owner, repo, git_ref, status, folder)) = row else {
            return Ok(None);
        };
        if status != Status::Selesai.as_str() {
            return Ok(None);
        }
        let Some(folder) = folder.filter(|value| !value.trim().is_empty()) else {
            return Ok(None);
        };
        tx.execute(
            "INSERT INTO history (owner, repo, git_ref, folder, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![owner, repo, git_ref, folder, finished_at],
        )?;
        let history_id = tx.last_insert_rowid();
        let deleted = tx.execute(
            "DELETE FROM queue WHERE id = ?1 AND status = 'selesai'",
            [id],
        )?;
        if deleted != 1 {
            return Err(StoreError::Invalid("item antrean tidak ada"));
        }
        tx.commit()?;
        Ok(Some(HistoryItem {
            id: history_id,
            owner,
            repo,
            git_ref,
            folder,
            finished_at: finished_at.to_owned(),
        }))
    }

    pub fn history(&self) -> Result<Vec<HistoryItem>, StoreError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, owner, repo, git_ref, folder, finished_at
             FROM history ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(HistoryItem {
                id: row.get(0)?,
                owner: row.get(1)?,
                repo: row.get(2)?,
                git_ref: row.get(3)?,
                folder: row.get(4)?,
                finished_at: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn history_item(&self, id: i64) -> Result<Option<HistoryItem>, StoreError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT id, owner, repo, git_ref, folder, finished_at
             FROM history WHERE id = ?1",
            [id],
            |row| {
                Ok(HistoryItem {
                    id: row.get(0)?,
                    owner: row.get(1)?,
                    repo: row.get(2)?,
                    git_ref: row.get(3)?,
                    folder: row.get(4)?,
                    finished_at: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(StoreError::from)
    }

    pub fn settings(&self) -> Result<Settings, StoreError> {
        let conn = self.lock()?;
        read_settings(&conn)
    }

    pub fn set_settings(
        &self,
        default_folder: Option<&str>,
        concurrency: u8,
        theme: &str,
    ) -> Result<Settings, StoreError> {
        if !(1..=8).contains(&concurrency) {
            return Err(StoreError::Invalid("jumlah bersamaan di luar 1 sampai 8"));
        }
        if !matches!(theme, "system" | "light" | "dark") {
            return Err(StoreError::Invalid("tema tidak dikenal"));
        }
        let folder = default_folder
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let conn = self.lock()?;
        conn.execute(
            "UPDATE settings SET default_folder = ?1, concurrency = ?2, theme = ?3 WHERE id = 1",
            params![folder, concurrency, theme],
        )?;
        read_settings(&conn)
    }

    pub fn set_language(&self, language: &str) -> Result<Settings, StoreError> {
        if normalize_language(language) != language {
            return Err(StoreError::Invalid("bahasa tidak dikenal"));
        }
        let conn = self.lock()?;
        conn.execute(
            "UPDATE settings SET language = ?1 WHERE id = 1",
            params![language],
        )?;
        read_settings(&conn)
    }

    pub fn github_client_id(&self) -> Result<String, StoreError> {
        let conn = self.lock()?;
        let value: Option<String> = conn.query_row(
            "SELECT github_client_id FROM settings WHERE id = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(value.unwrap_or_default())
    }

    pub fn set_github_client_id(&self, value: &str) -> Result<(), StoreError> {
        let trimmed = value.trim();
        let stored: Option<&str> = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        };
        let conn = self.lock()?;
        conn.execute(
            "UPDATE settings SET github_client_id = ?1 WHERE id = 1",
            params![stored],
        )?;
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, StoreError> {
        self.conn
            .lock()
            .map_err(|_| StoreError::Invalid("kunci basis data rusak"))
    }
}

fn has_column(conn: &Connection, table: &str, name: &str) -> Result<bool, StoreError> {
    let sql = match table {
        "queue" => "SELECT name FROM pragma_table_info('queue')",
        "settings" => "SELECT name FROM pragma_table_info('settings')",
        _ => return Err(StoreError::Invalid("tabel tidak dikenal")),
    };
    let mut stmt = conn.prepare(sql)?;
    let names = stmt.query_map([], |row| row.get::<_, String>(0))?;
    for found in names {
        if found? == name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn ensure_columns(conn: &Connection) -> Result<(), StoreError> {
    if !has_column(conn, "queue", "folder")? {
        conn.execute("ALTER TABLE queue ADD COLUMN folder TEXT", [])?;
    }
    if !has_column(conn, "settings", "github_client_id")? {
        conn.execute("ALTER TABLE settings ADD COLUMN github_client_id TEXT", [])?;
    }
    if !has_column(conn, "settings", "language")? {
        conn.execute(
            "ALTER TABLE settings ADD COLUMN language TEXT NOT NULL DEFAULT 'id'",
            [],
        )?;
    }
    Ok(())
}

fn normalize_language(value: &str) -> String {
    if matches!(
        value,
        "en" | "hi" | "zh" | "pt" | "en-GB" | "ja" | "id" | "de" | "ru" | "fr"
    ) {
        value.to_owned()
    } else {
        "id".to_owned()
    }
}

fn require_text(value: &str, message: &'static str) -> Result<(), StoreError> {
    if value.trim().is_empty() {
        Err(StoreError::Invalid(message))
    } else {
        Ok(())
    }
}

fn read_queue(conn: &Connection, id: i64) -> Result<QueueItem, StoreError> {
    let row = conn
        .query_row(
            "SELECT id, owner, repo, git_ref, dest, status, bytes, total, error, folder
             FROM queue WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            },
        )
        .optional()?;
    let Some((id, owner, repo, git_ref, dest, status, bytes, total, error, folder)) = row else {
        return Err(StoreError::Invalid("item antrean tidak ada"));
    };
    Ok(QueueItem {
        id,
        owner,
        repo,
        git_ref,
        dest,
        status: Status::parse(&status)?,
        bytes,
        total,
        error,
        folder,
    })
}

fn read_settings(conn: &Connection) -> Result<Settings, StoreError> {
    let (default_folder, concurrency, theme, language) = conn.query_row(
        "SELECT default_folder, concurrency, theme, language FROM settings WHERE id = 1",
        [],
        |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    )?;
    let concurrency =
        u8::try_from(concurrency).map_err(|_| StoreError::Invalid("jumlah bersamaan rusak"))?;
    Ok(Settings {
        default_folder,
        concurrency,
        theme,
        language: normalize_language(&language),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static N: AtomicU64 = AtomicU64::new(0);

    fn temp_db() -> std::path::PathBuf {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("peti-store-{}-{n}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        path
    }

    #[test]
    fn queue_item_survives_close_and_reopen() {
        let path = temp_db();
        let item = {
            let store = Store::open(&path).unwrap();
            store
                .insert_queue("octocat", "hello", "", "/tmp/out")
                .unwrap()
        };
        let store = Store::open(&path).unwrap();
        let again = store.queue().unwrap();
        assert_eq!(again.len(), 1);
        assert_eq!(again[0].id, item.id);
        assert_eq!(again[0].owner, "octocat");
        assert_eq!(again[0].repo, "hello");
        assert_eq!(again[0].git_ref, "");
        assert_eq!(again[0].dest, "/tmp/out");
        assert_eq!(again[0].status, Status::Menunggu);
        assert_eq!(again[0].bytes, 0);
        assert_eq!(again[0].total, None);
        assert_eq!(again[0].error, None);
    }

    #[test]
    fn running_item_is_failed_when_reopened() {
        let path = temp_db();
        {
            let store = Store::open(&path).unwrap();
            let item = store
                .insert_queue("octocat", "hello", "main", "/tmp/out")
                .unwrap();
            store.set_status(item.id, Status::Berjalan).unwrap();
            let paused = store
                .insert_queue("octocat", "other", "", "/tmp/other")
                .unwrap();
            store.set_status(paused.id, Status::Dijeda).unwrap();
        }
        let store = Store::open(&path).unwrap();
        let items = store.queue().unwrap();
        assert_eq!(items[0].status, Status::Gagal);
        assert_eq!(items[0].error.as_deref(), Some("ditutup saat berjalan"));
        assert_eq!(items[1].status, Status::Dijeda);
        assert_eq!(items[1].error, None);
    }

    #[test]
    fn schema_has_no_token_column() {
        let path = temp_db();
        let store = Store::open(&path).unwrap();
        let conn = store.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT name FROM pragma_table_info('queue')
                 UNION ALL SELECT name FROM pragma_table_info('history')
                 UNION ALL SELECT name FROM pragma_table_info('settings')",
            )
            .unwrap();
        let names: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(!names.is_empty());
        for name in &names {
            let lower = name.to_ascii_lowercase();
            assert!(!lower.contains("token"), "{name}");
            assert!(!lower.contains("secret"), "{name}");
            assert!(!lower.contains("password"), "{name}");
        }
    }

    #[test]
    fn settings_roundtrip_and_reject_bad_values() {
        let path = temp_db();
        let store = Store::open(&path).unwrap();
        let defaults = store.settings().unwrap();
        assert_eq!(defaults.default_folder, None);
        assert_eq!(defaults.concurrency, 3);
        assert_eq!(defaults.theme, "system");
        assert_eq!(defaults.language, "id");
        store.set_language("ja").unwrap();
        store.set_settings(Some("/tmp/peti"), 8, "dark").unwrap();
        drop(store);
        let store = Store::open(&path).unwrap();
        let saved = store.settings().unwrap();
        assert_eq!(saved.default_folder.as_deref(), Some("/tmp/peti"));
        assert_eq!(saved.concurrency, 8);
        assert_eq!(saved.theme, "dark");
        assert_eq!(saved.language, "ja");
        assert!(store.set_settings(None, 0, "dark").is_err());
        assert!(store.set_settings(None, 9, "dark").is_err());
        assert!(store.set_settings(None, 3, "blue").is_err());
        assert!(store.set_language("blue").is_err());
        let still = store.settings().unwrap();
        assert_eq!(still.concurrency, 8);
        assert_eq!(still.theme, "dark");
        assert_eq!(still.language, "ja");
    }

    #[test]
    fn github_client_id_is_stored_and_empty_is_rejected() {
        let path = temp_db();
        let store = Store::open(&path).unwrap();
        assert_eq!(store.github_client_id().unwrap(), "");
        store.set_github_client_id("Iv1.abc_DEF-1").unwrap();
        drop(store);
        let store = Store::open(&path).unwrap();
        assert_eq!(store.github_client_id().unwrap(), "Iv1.abc_DEF-1");
        store.set_settings(Some("/tmp/peti"), 2, "light").unwrap();
        assert_eq!(store.github_client_id().unwrap(), "Iv1.abc_DEF-1");
        assert_eq!(store.settings().unwrap().concurrency, 2);
        store.set_github_client_id("  ").unwrap();
        assert_eq!(store.github_client_id().unwrap(), "");
        assert!(!crate::github::client_id_ok(""));
        assert!(!crate::github::client_id_ok("bad id"));
        assert!(crate::github::client_id_ok("Iv1.abc_DEF-1"));
    }

    #[test]
    fn history_survives_reopen_without_failed_rows() {
        let path = temp_db();
        {
            let store = Store::open(&path).unwrap();
            store
                .insert_history(
                    "octocat",
                    "hello",
                    "main",
                    "/tmp/out/octocat-hello",
                    "2026-10-14T10:00:00Z",
                )
                .unwrap();
        }
        let store = Store::open(&path).unwrap();
        let rows = store.history().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].folder, "/tmp/out/octocat-hello");
        assert_eq!(rows[0].finished_at, "2026-10-14T10:00:00Z");
        assert!(store.queue().unwrap().is_empty());
    }

    #[test]
    fn history_lists_the_newest_finish_first() {
        let store = Store::open(&temp_db()).unwrap();
        let older = store
            .insert_history(
                "octocat",
                "older",
                "",
                "/tmp/out/older",
                "2026-10-13 00:00:00 UTC",
            )
            .unwrap();
        let newer = store
            .insert_history(
                "octocat",
                "newer",
                "",
                "/tmp/out/newer",
                "2026-10-14 00:00:00 UTC",
            )
            .unwrap();
        let rows = store.history().unwrap();
        assert_eq!(rows[0].id, newer.id);
        assert_eq!(rows[1].id, older.id);
    }

    #[test]
    fn finished_item_moves_to_history_and_a_failure_stays_out() {
        let store = Store::open(&temp_db()).unwrap();
        let done = store
            .insert_queue("octocat", "hello", "main", "/tmp/out")
            .unwrap();
        store.set_status(done.id, Status::Berjalan).unwrap();
        store.set_folder(done.id, "/tmp/out/octocat-hello").unwrap();
        store.finish_item(done.id, 4, None).unwrap();
        let failed = store
            .insert_queue("octocat", "nope", "", "/tmp/out")
            .unwrap();
        store.set_status(failed.id, Status::Berjalan).unwrap();
        store
            .set_folder(failed.id, "/tmp/out/octocat-nope")
            .unwrap();
        store
            .fail_item(failed.id, "Repositori tidak tersedia")
            .unwrap();
        let blank = store
            .insert_queue("octocat", "blank", "", "/tmp/out")
            .unwrap();
        store.finish_item(blank.id, 1, None).unwrap();

        assert!(
            store
                .archive_finished(failed.id, "2026-10-14 00:00:00 UTC")
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .archive_finished(blank.id, "2026-10-14 00:00:00 UTC")
                .unwrap()
                .is_none()
        );
        let moved = store
            .archive_finished(done.id, "2026-10-14 00:00:01 UTC")
            .unwrap()
            .unwrap();
        assert_eq!(moved.folder, "/tmp/out/octocat-hello");
        assert_eq!(moved.git_ref, "main");
        assert!(store.get(done.id).is_err());
        assert!(
            store
                .archive_finished(done.id, "2026-10-14 00:00:02 UTC")
                .unwrap()
                .is_none()
        );
        let rows = store.history().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].repo, "hello");
        let left = store.queue().unwrap();
        assert!(
            left.iter()
                .any(|item| item.id == failed.id && item.status == Status::Gagal)
        );
        assert!(
            left.iter()
                .any(|item| item.id == blank.id && item.status == Status::Selesai)
        );
        assert!(
            left.iter()
                .all(|item| item.status != Status::Selesai || item.folder.is_none())
        );
    }

    #[test]
    fn older_database_gains_folder_without_losing_the_row() {
        let path = temp_db();
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE queue (
                    id INTEGER PRIMARY KEY,
                    owner TEXT NOT NULL,
                    repo TEXT NOT NULL,
                    git_ref TEXT NOT NULL,
                    dest TEXT NOT NULL,
                    status TEXT NOT NULL,
                    bytes INTEGER NOT NULL DEFAULT 0,
                    total INTEGER,
                    error TEXT
                );
                INSERT INTO queue (owner, repo, git_ref, dest, status, bytes)
                VALUES ('octocat', 'hello', '', '/tmp/out', 'berjalan', 12);",
            )
            .unwrap();
        }
        let store = Store::open(&path).unwrap();
        let items = store.queue().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, Status::Gagal);
        assert_eq!(items[0].error.as_deref(), Some("ditutup saat berjalan"));
        assert_eq!(items[0].bytes, 12);
        assert_eq!(items[0].folder, None);
    }

    #[test]
    fn controls_only_change_the_status_they_own() {
        let store = Store::open(&temp_db()).unwrap();
        let item = store
            .insert_queue("octocat", "hello", "", "/tmp/out")
            .unwrap();
        assert!(store.pause_item(item.id).is_err());
        assert!(store.retry_item(item.id).is_err());
        store.set_status(item.id, Status::Berjalan).unwrap();
        store.pause_item(item.id).unwrap();
        assert_eq!(store.get(item.id).unwrap().status, Status::Dijeda);
        store.set_status(item.id, Status::Gagal).unwrap();
        store.fail_item(item.id, "Unduhan terputus. Ulang").unwrap();
        store.retry_item(item.id).unwrap();
        let again = store.get(item.id).unwrap();
        assert_eq!(again.status, Status::Menunggu);
        assert_eq!(again.bytes, 0);
        assert_eq!(again.total, None);
        assert_eq!(again.error, None);
        assert_eq!(again.folder, None);

        store.set_status(item.id, Status::Berjalan).unwrap();
        store.set_folder(item.id, "/tmp/out/octocat-hello").unwrap();
        store.save_progress(item.id, 4, Some(8)).unwrap();
        store.cancel_item(item.id).unwrap();
        let cancelled = store.get(item.id).unwrap();
        assert_eq!(cancelled.status, Status::Dibatalkan);
        assert_eq!(cancelled.bytes, 0);
        assert_eq!(cancelled.total, None);
        assert_eq!(cancelled.error, None);
        assert_eq!(cancelled.folder, None);
        store.save_progress(item.id, 9, Some(9)).unwrap();
        let still = store.get(item.id).unwrap();
        assert_eq!(still.status, Status::Dibatalkan);
        assert_eq!(still.bytes, 0);
        store.set_status(item.id, Status::Selesai).unwrap();
        assert!(store.cancel_item(item.id).is_err());
        assert_eq!(store.get(item.id).unwrap().status, Status::Selesai);
    }
}
