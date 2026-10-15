use std::path::{Path, PathBuf};
use std::process::Command;

use crate::store::Store;

pub const MISSING: &str = "Folder tidak bisa dipakai, pilih yang lain";

pub fn real_directory(path: &Path) -> Result<(), &'static str> {
    let meta = std::fs::symlink_metadata(path).map_err(|_| MISSING)?;
    if meta.file_type().is_symlink() || !meta.is_dir() {
        return Err(MISSING);
    }
    Ok(())
}

pub fn stored_directory(store: &Store, id: i64) -> Result<PathBuf, String> {
    let item = store
        .history_item(id)
        .map_err(|err| err.to_string())?
        .ok_or(MISSING)?;
    let path = PathBuf::from(&item.folder);
    real_directory(&path).map_err(str::to_owned)?;
    Ok(path)
}

pub fn open_directory(path: &Path) -> Result<(), &'static str> {
    real_directory(path)?;
    spawn_open(path)
}

fn spawn_open(path: &Path) -> Result<(), &'static str> {
    let status = open_command(path).status().map_err(|_| MISSING)?;
    #[cfg(target_os = "windows")]
    {
        let _ = status;
        return Ok(());
    }
    #[cfg(not(target_os = "windows"))]
    {
        if status.success() {
            Ok(())
        } else {
            Err(MISSING)
        }
    }
}

fn open_command(path: &Path) -> Command {
    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("open");
        cmd.arg(path);
        return cmd;
    }
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("explorer");
        cmd.arg(path);
        return cmd;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(path);
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "peti-reveal-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn only_a_real_directory_can_be_opened() {
        let parent = temp_dir();
        assert_eq!(real_directory(&parent.join("gone")), Err(MISSING));
        let file = parent.join("file.txt");
        std::fs::write(&file, b"x").unwrap();
        assert_eq!(real_directory(&file), Err(MISSING));
        let dir = parent.join("dir");
        std::fs::create_dir(&dir).unwrap();
        assert_eq!(real_directory(&dir), Ok(()));
        #[cfg(unix)]
        {
            let outside = parent.join("secret");
            std::fs::create_dir(&outside).unwrap();
            std::fs::write(outside.join("secret.txt"), b"no").unwrap();
            let link = parent.join("link");
            std::os::unix::fs::symlink(&outside, &link).unwrap();
            assert_eq!(real_directory(&link), Err(MISSING));
            assert_eq!(std::fs::read(outside.join("secret.txt")).unwrap(), b"no");
        }
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn opening_uses_the_stored_history_path_and_refuses_a_symlink() {
        let parent = temp_dir();
        let dir = parent.join("octocat-hello");
        std::fs::create_dir(&dir).unwrap();
        let store = Store::open(&parent.join("peti.db")).unwrap();
        let row = store
            .insert_history(
                "octocat",
                "hello",
                "",
                dir.to_str().unwrap(),
                "2026-10-14 00:00:00 UTC",
            )
            .unwrap();
        assert_eq!(stored_directory(&store, row.id).unwrap(), dir);
        assert!(stored_directory(&store, row.id + 1).is_err());

        #[cfg(unix)]
        {
            let outside = parent.join("secret");
            std::fs::create_dir(&outside).unwrap();
            std::fs::write(outside.join("secret.txt"), b"no").unwrap();
            let link = parent.join("linked");
            std::os::unix::fs::symlink(&outside, &link).unwrap();
            let linked = store
                .insert_history(
                    "octocat",
                    "linked",
                    "",
                    link.to_str().unwrap(),
                    "2026-10-14 00:00:01 UTC",
                )
                .unwrap();
            assert!(stored_directory(&store, linked.id).is_err());
            assert_eq!(std::fs::read(outside.join("secret.txt")).unwrap(), b"no");
        }
        let _ = std::fs::remove_dir_all(&parent);
    }
}
