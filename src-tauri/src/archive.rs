use std::fs::{self, OpenOptions};
use std::io::{self, Read, Seek, Write};
use std::path::{Component, Path, PathBuf};

use zip::ZipArchive;

const MAX_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const MAX_SYMLINK_TARGET: u64 = 4096;

#[derive(Debug)]
pub struct ExtractRejected;

enum Planned {
    Dir {
        rel: PathBuf,
    },
    File {
        index: usize,
        rel: PathBuf,
        declared: u64,
    },
    Link {
        rel: PathBuf,
        target: String,
    },
}

pub fn extract_archive<R: Read + Seek>(reader: R, dest: &Path) -> Result<(), ExtractRejected> {
    if !dest.is_dir() {
        return Err(ExtractRejected);
    }
    let mut existing = fs::read_dir(dest).map_err(|_| ExtractRejected)?;
    if existing.next().is_some() {
        return Err(ExtractRejected);
    }

    let mut archive = ZipArchive::new(reader).map_err(|_| ExtractRejected)?;
    let mut planned = Vec::new();
    let mut declared_total = 0u64;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|_| ExtractRejected)?;
        let name = file.name().map_err(|_| ExtractRejected)?.into_owned();
        let rel = safe_relative(&name).ok_or(ExtractRejected)?;
        if rel.as_os_str().is_empty() {
            if file.is_dir() {
                continue;
            }
            return Err(ExtractRejected);
        }
        if file.is_symlink() {
            if file.size() > MAX_SYMLINK_TARGET {
                return Err(ExtractRejected);
            }
            let mut target_bytes = Vec::new();
            file.by_ref()
                .take(MAX_SYMLINK_TARGET + 1)
                .read_to_end(&mut target_bytes)
                .map_err(|_| ExtractRejected)?;
            if target_bytes.len() as u64 > MAX_SYMLINK_TARGET {
                return Err(ExtractRejected);
            }
            let target = String::from_utf8(target_bytes).map_err(|_| ExtractRejected)?;
            if !symlink_stays_inside(&rel, &target) {
                return Err(ExtractRejected);
            }
            planned.push(Planned::Link { rel, target });
        } else if file.is_dir() {
            planned.push(Planned::Dir { rel });
        } else {
            let declared = file.size();
            declared_total = declared_total.saturating_add(declared);
            if declared_total > MAX_BYTES {
                return Err(ExtractRejected);
            }
            planned.push(Planned::File {
                index,
                rel,
                declared,
            });
        }
    }
    if duplicate_paths(&planned) {
        return Err(ExtractRejected);
    }
    strip_wrapper(&mut planned);
    // The wrapper check runs on the stored path. Stripping that directory can
    // turn a target of ".." into a link that leaves dest, so check again.
    if planned.iter().any(
        |item| matches!(item, Planned::Link { rel, target } if !symlink_stays_inside(rel, target)),
    ) {
        return Err(ExtractRejected);
    }

    let wrote = write_planned(&mut archive, dest, &planned);
    if wrote.is_err() {
        clear_dir(dest);
    }
    wrote
}

fn write_planned<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    dest: &Path,
    planned: &[Planned],
) -> Result<(), ExtractRejected> {
    let mut written_total = 0u64;
    for item in planned {
        match item {
            Planned::Dir { rel } => {
                let _path = inside(dest, rel)?;
                create_dirs_nofollow(dest, rel)?;
            }
            Planned::Link { rel, target } => {
                let path = inside(dest, rel)?;
                create_dirs_nofollow(dest, rel.parent().unwrap_or(Path::new("")))?;
                if fs::symlink_metadata(&path).is_ok() {
                    return Err(ExtractRejected);
                }
                make_symlink(&path, target).map_err(|_| ExtractRejected)?;
            }
            Planned::File {
                index,
                rel,
                declared,
            } => {
                let path = inside(dest, rel)?;
                create_dirs_nofollow(dest, rel.parent().unwrap_or(Path::new("")))?;
                if fs::symlink_metadata(&path).is_ok() {
                    return Err(ExtractRejected);
                }
                let mut file = archive.by_index(*index).map_err(|_| ExtractRejected)?;
                let mut out = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .map_err(|_| ExtractRejected)?;
                let mut buf = [0u8; 64 * 1024];
                let mut file_written = 0u64;
                let limit = if *declared == 0 { MAX_BYTES } else { *declared };
                loop {
                    let read = file.read(&mut buf).map_err(|_| ExtractRejected)?;
                    if read == 0 {
                        break;
                    }
                    file_written += read as u64;
                    written_total = written_total.saturating_add(read as u64);
                    if file_written > limit || written_total > MAX_BYTES {
                        return Err(ExtractRejected);
                    }
                    out.write_all(&buf[..read]).map_err(|_| ExtractRejected)?;
                }
                // Ignore the mode stored in the archive. A zip must not mark a file setuid.
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o644));
                }
            }
        }
    }
    Ok(())
}

fn create_dirs_nofollow(dest: &Path, rel: &Path) -> Result<(), ExtractRejected> {
    let mut current = dest.to_path_buf();
    for component in rel.components() {
        let Component::Normal(part) = component else {
            return Err(ExtractRejected);
        };
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => return Err(ExtractRejected),
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => return Err(ExtractRejected),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|_| ExtractRejected)?;
            }
            Err(_) => return Err(ExtractRejected),
        }
    }
    Ok(())
}

fn inside(dest: &Path, rel: &Path) -> Result<PathBuf, ExtractRejected> {
    if rel.is_absolute()
        || rel
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(ExtractRejected);
    }
    let joined = dest.join(rel);
    if !joined.starts_with(dest) {
        return Err(ExtractRejected);
    }
    Ok(joined)
}

fn safe_relative(name: &str) -> Option<PathBuf> {
    if name.is_empty() || name.contains('\0') || name.contains(':') {
        return None;
    }
    if name.starts_with('/') || name.starts_with('\\') {
        return None;
    }
    let mut out = PathBuf::new();
    let mut depth = 0i32;
    for component in name.split(['/', '\\']) {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            if depth == 0 {
                return None;
            }
            out.pop();
            depth -= 1;
            continue;
        }
        out.push(component);
        depth += 1;
    }
    Some(out)
}

fn symlink_stays_inside(link_rel: &Path, target: &str) -> bool {
    if target.is_empty() || target.contains('\0') || target.contains(':') {
        return false;
    }
    if target.starts_with('/') || target.starts_with('\\') {
        return false;
    }
    let parent = link_rel.parent().unwrap_or(Path::new(""));
    let mut parts: Vec<&str> = parent
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect();
    if parts.len() != parent.components().count() {
        return false;
    }
    for component in target.split(['/', '\\']) {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            if parts.pop().is_none() {
                return false;
            }
            continue;
        }
        parts.push(component);
    }
    true
}

fn duplicate_paths(planned: &[Planned]) -> bool {
    let mut seen = Vec::new();
    for item in planned {
        let rel = match item {
            Planned::Dir { rel } | Planned::File { rel, .. } | Planned::Link { rel, .. } => rel,
        };
        if seen.iter().any(|existing: &PathBuf| existing == rel) {
            return true;
        }
        seen.push(rel.clone());
    }
    false
}

fn strip_wrapper(planned: &mut Vec<Planned>) {
    let prefix = {
        let mut prefix: Option<String> = None;
        for item in planned.iter() {
            let rel = match item {
                Planned::Dir { rel } | Planned::File { rel, .. } | Planned::Link { rel, .. } => rel,
            };
            let mut components = rel.components();
            let Some(first) = components.next() else {
                return;
            };
            let Some(name) = first.as_os_str().to_str() else {
                return;
            };
            if prefix.get_or_insert(name.to_owned()).as_str() != name {
                return;
            }
            let nested = components.next().is_some();
            let is_dir = matches!(item, Planned::Dir { .. });
            if !nested && !is_dir {
                return;
            }
        }
        prefix
    };
    let Some(prefix) = prefix else {
        return;
    };
    planned.retain_mut(|item| {
        let rel = match item {
            Planned::Dir { rel } | Planned::File { rel, .. } | Planned::Link { rel, .. } => rel,
        };
        let stripped = rel.strip_prefix(&prefix).unwrap_or(rel.as_path());
        if stripped.as_os_str().is_empty() {
            return false;
        }
        *rel = stripped.to_path_buf();
        true
    });
}

fn clear_dir(dir: &Path) {
    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
        if is_dir {
            let _ = fs::remove_dir_all(path);
        } else {
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(unix)]
fn make_symlink(path: &Path, target: &str) -> io::Result<()> {
    std::os::unix::fs::symlink(target, path)
}

#[cfg(windows)]
fn make_symlink(path: &Path, target: &str) -> io::Result<()> {
    std::os::windows::fs::symlink_file(target, path)
}

#[cfg(not(any(unix, windows)))]
fn make_symlink(path: &Path, _target: &str) -> io::Result<()> {
    let _ = path;
    Err(io::Error::new(io::ErrorKind::Unsupported, "symlink"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    enum Body<'a> {
        File(&'a [u8]),
        Link(&'a str),
    }

    fn make_zip(entries: &[(&str, Body<'_>)]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut cursor);
            let options = SimpleFileOptions::default();
            for (name, body) in entries {
                match body {
                    Body::File(bytes) => {
                        writer.start_file(*name, options).unwrap();
                        writer.write_all(bytes).unwrap();
                    }
                    Body::Link(target) => {
                        writer.add_symlink(*name, *target, options).unwrap();
                    }
                }
            }
            writer.finish().unwrap();
        }
        cursor.into_inner()
    }

    fn dest(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("peti-arc-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn strips_the_single_wrapper_directory() {
        let bytes = make_zip(&[("Hello-World-main/README", Body::File(b"hi\n"))]);
        let folder = dest("wrap");
        extract_archive(Cursor::new(bytes), &folder).unwrap();
        assert_eq!(fs::read(folder.join("README")).unwrap(), b"hi\n");
        assert!(!folder.join("Hello-World-main").exists());
        assert!(!folder.join(".git").exists());
        let _ = fs::remove_dir_all(folder);
    }

    #[test]
    fn keeps_a_parent_segment_that_stays_inside() {
        let bytes = make_zip(&[("dir/../ok.txt", Body::File(b"ok"))]);
        let folder = dest("stay");
        extract_archive(Cursor::new(bytes), &folder).unwrap();
        assert_eq!(fs::read(folder.join("ok.txt")).unwrap(), b"ok");
        let _ = fs::remove_dir_all(folder);
    }

    #[test]
    fn rejects_the_whole_archive_when_one_path_escapes() {
        let bytes = make_zip(&[
            ("ok.txt", Body::File(b"keep")),
            ("../outside.txt", Body::File(b"bad")),
        ]);
        let folder = dest("slip");
        let parent = folder.parent().unwrap().to_path_buf();
        assert!(extract_archive(Cursor::new(bytes), &folder).is_err());
        assert!(!folder.join("ok.txt").exists());
        assert!(!parent.join("outside.txt").exists());
        let _ = fs::remove_dir_all(folder);
    }

    #[test]
    fn rejects_absolute_paths_and_backslash_escapes() {
        for name in [
            "/tmp/pwned.txt",
            r"..\..\outside.txt",
            "C:/Windows/note.txt",
        ] {
            let bytes = make_zip(&[(name, Body::File(b"bad"))]);
            let folder = dest("abs");
            assert!(
                extract_archive(Cursor::new(bytes), &folder).is_err(),
                "{name}"
            );
            assert!(fs::read_dir(&folder).unwrap().next().is_none(), "{name}");
            let _ = fs::remove_dir_all(&folder);
        }
    }

    #[test]
    fn rejects_a_symlink_that_points_outside_and_writes_nothing() {
        let bytes = make_zip(&[
            ("README", Body::File(b"hi")),
            ("link", Body::Link("/etc/passwd")),
        ]);
        let folder = dest("link-out");
        assert!(extract_archive(Cursor::new(bytes), &folder).is_err());
        assert!(!folder.join("README").exists());
        assert!(!folder.join("link").exists());
        let _ = fs::remove_dir_all(folder);
    }

    #[test]
    fn rejects_a_symlink_chain_that_would_write_outside() {
        let bytes = make_zip(&[
            ("README", Body::File(b"hi")),
            ("level1/up", Body::Link("..")),
            ("level1/up/link", Body::Link("..")),
            ("level1/up/link/pwned.txt", Body::File(b"pwned")),
        ]);
        let folder = dest("chain");
        let parent = folder.parent().unwrap().to_path_buf();
        assert!(extract_archive(Cursor::new(bytes), &folder).is_err());
        assert!(!parent.join("pwned.txt").exists());
        assert!(!folder.join("link").exists());
        assert!(fs::read_dir(&folder).unwrap().next().is_none());
        let _ = fs::remove_dir_all(folder);
    }

    #[test]
    fn rejects_a_wrapper_symlink_that_points_outside_after_strip() {
        let bytes = make_zip(&[
            ("repo-main/link", Body::Link("..")),
            ("repo-main/link/escaped.txt", Body::File(b"bad")),
        ]);
        let folder = dest("wrap-link");
        let parent = folder.parent().unwrap().to_path_buf();
        assert!(extract_archive(Cursor::new(bytes), &folder).is_err());
        assert!(!parent.join("escaped.txt").exists());
        assert!(!folder.join("link").exists());
        assert!(fs::read_dir(&folder).unwrap().next().is_none());
        let _ = fs::remove_dir_all(folder);
    }

    #[cfg(unix)]
    #[test]
    fn keeps_a_relative_symlink_under_a_wrapper() {
        let bytes = make_zip(&[
            ("repo-main/README", Body::File(b"hi")),
            ("repo-main/link", Body::Link("README")),
        ]);
        let folder = dest("wrap-link-in");
        extract_archive(Cursor::new(bytes), &folder).unwrap();
        assert_eq!(fs::read(folder.join("README")).unwrap(), b"hi");
        assert_eq!(
            fs::read_link(folder.join("link")).unwrap(),
            PathBuf::from("README")
        );
        let _ = fs::remove_dir_all(folder);
    }

    #[cfg(unix)]
    #[test]
    fn extracts_a_relative_symlink_that_stays_inside() {
        let bytes = make_zip(&[
            ("README", Body::File(b"hi")),
            ("link", Body::Link("README")),
        ]);
        let folder = dest("link-in");
        extract_archive(Cursor::new(bytes), &folder).unwrap();
        assert_eq!(fs::read(folder.join("README")).unwrap(), b"hi");
        assert_eq!(
            fs::read_link(folder.join("link")).unwrap(),
            PathBuf::from("README")
        );
        let _ = fs::remove_dir_all(folder);
    }
}
