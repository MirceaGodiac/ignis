//! Consistent snapshots of a server state directory, and restore onto an empty volume.
//!
//! A snapshot is a directory:
//! ```text
//! manifest.json          { format, created_at, files: [{ path, sha256, bytes, kind }] }
//! files/<path>           the captured files, same relative paths as in the state directory
//! ```
//! SQLite databases (`*.sqlite`) are captured with `VACUUM INTO`, a read transaction that yields a
//! consistent, self-contained copy while the server keeps writing (never a raw copy of a live WAL
//! database). Other files (Codex thread logs, the reveal log, local artifacts) are copied as they
//! are; they are append-only or immutable. Left out: WAL/SHM/journal side files, upload and
//! artifact staging (never referenced), and credentials (`auth.json`, `*.key`, `*.env`).
//!
//! Restore refuses a non-empty target, verifies every file's hash and size before copying, then
//! runs `PRAGMA integrity_check` on every restored database. Artifacts in R2 are not part of the
//! snapshot: they are immutable and content-addressed in the bucket, and the restored manifests
//! keep pointing at them.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::s3::{Payload, S3Client, S3Error};

pub const FORMAT: &str = "ignis-state-snapshot.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    Sqlite,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotFile {
    /// Relative path with `/` separators.
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub kind: FileKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotManifest {
    pub format: String,
    pub created_at: String,
    pub files: Vec<SnapshotFile>,
}

type Result<T> = std::result::Result<T, String>;

fn ctx<E: std::fmt::Display>(what: impl std::fmt::Display) -> impl FnOnce(E) -> String {
    move |e| format!("{what}: {e}")
}

/// Whether `rel` (relative, `/`-separated) is left out of snapshots.
fn excluded(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    let parts: Vec<&str> = rel.split('/').collect();
    name.ends_with("-wal")
        || name.ends_with("-shm")
        || name.ends_with("-journal")
        || name == "auth.json"
        || name.ends_with(".key")
        || name.ends_with(".env")
        || name.ends_with(".part")
        || parts.iter().any(|p| *p == "uploads" || *p == "staging")
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for e in std::fs::read_dir(dir).map_err(ctx(dir.display()))? {
        let e = e.map_err(ctx(dir.display()))?;
        let ty = e.file_type().map_err(ctx(e.path().display()))?;
        if ty.is_dir() {
            walk(root, &e.path(), out)?;
        } else if ty.is_file() {
            out.push(
                e.path()
                    .strip_prefix(root)
                    .expect("under root")
                    .to_path_buf(),
            );
        }
        // Symlinks and special files are never captured.
    }
    Ok(())
}

fn rel_string(p: &Path) -> String {
    p.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn hash_file(p: &Path) -> Result<(String, u64)> {
    let mut f = File::open(p).map_err(ctx(p.display()))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 256 * 1024];
    let mut n_total = 0u64;
    loop {
        let n = f.read(&mut buf).map_err(ctx(p.display()))?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
        n_total += n as u64;
    }
    Ok((hex::encode(h.finalize()), n_total))
}

/// Capture `state` into the new directory `out` (must not exist).
pub fn snapshot(state: &Path, out: &Path) -> Result<SnapshotManifest> {
    if out.exists() {
        return Err(format!("{} already exists", out.display()));
    }
    let mut rels = Vec::new();
    walk(state, state, &mut rels)?;
    rels.sort();
    let files_dir = out.join("files");
    std::fs::create_dir_all(&files_dir).map_err(ctx(out.display()))?;
    let mut files = Vec::new();
    for rel in rels {
        let rs = rel_string(&rel);
        if excluded(&rs) {
            continue;
        }
        let src = state.join(&rel);
        let dst = files_dir.join(&rel);
        std::fs::create_dir_all(dst.parent().expect("has parent")).map_err(ctx(dst.display()))?;
        let kind = if rs.ends_with(".sqlite") {
            let conn = rusqlite::Connection::open_with_flags(
                &src,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                    | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
            .map_err(ctx(src.display()))?;
            conn.busy_timeout(std::time::Duration::from_secs(30))
                .map_err(ctx(src.display()))?;
            conn.execute("VACUUM INTO ?1", [dst.to_string_lossy()])
                .map_err(ctx(format!("VACUUM INTO from {}", src.display())))?;
            FileKind::Sqlite
        } else {
            std::fs::copy(&src, &dst).map_err(ctx(src.display()))?;
            FileKind::File
        };
        let (sha256, bytes) = hash_file(&dst)?;
        files.push(SnapshotFile {
            path: rs,
            sha256,
            bytes,
            kind,
        });
    }
    let m = SnapshotManifest {
        format: FORMAT.into(),
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        files,
    };
    let text = serde_json::to_vec_pretty(&m).map_err(ctx("manifest"))?;
    let mut f = File::create(out.join("manifest.json")).map_err(ctx("manifest"))?;
    f.write_all(&text).map_err(ctx("manifest"))?;
    f.sync_all().map_err(ctx("manifest"))?;
    Ok(m)
}

fn read_manifest(snap: &Path) -> Result<SnapshotManifest> {
    let text = std::fs::read(snap.join("manifest.json")).map_err(ctx("manifest.json"))?;
    let m: SnapshotManifest = serde_json::from_slice(&text).map_err(ctx("manifest.json"))?;
    if m.format != FORMAT {
        return Err(format!("unsupported snapshot format {}", m.format));
    }
    for f in &m.files {
        let safe = !f.path.is_empty()
            && !f.path.starts_with('/')
            && !f.path.contains('\\')
            && f.path
                .split('/')
                .all(|c| !c.is_empty() && c != "." && c != ".." && !c.contains(':'));
        if !safe {
            return Err(format!("unsafe path in manifest: {:?}", f.path));
        }
    }
    Ok(m)
}

/// Check every file of the snapshot against its manifest.
pub fn verify(snap: &Path) -> Result<SnapshotManifest> {
    let m = read_manifest(snap)?;
    for f in &m.files {
        let (sha, bytes) = hash_file(&snap.join("files").join(&f.path))?;
        if sha != f.sha256 || bytes != f.bytes {
            return Err(format!("{} does not match the manifest", f.path));
        }
    }
    Ok(m)
}

/// Restore `snap` into `target`, which must be missing or empty.
pub fn restore(snap: &Path, target: &Path) -> Result<SnapshotManifest> {
    if target.exists()
        && std::fs::read_dir(target)
            .map_err(ctx(target.display()))?
            .next()
            .is_some()
    {
        return Err(format!(
            "{} is not empty: restore only onto an empty volume",
            target.display()
        ));
    }
    let m = verify(snap)?;
    for f in &m.files {
        let dst = target.join(&f.path);
        std::fs::create_dir_all(dst.parent().expect("has parent")).map_err(ctx(dst.display()))?;
        std::fs::copy(snap.join("files").join(&f.path), &dst).map_err(ctx(dst.display()))?;
        // Opened writable: Windows refuses to flush a read-only handle.
        std::fs::OpenOptions::new()
            .write(true)
            .open(&dst)
            .and_then(|f| f.sync_all())
            .map_err(ctx(dst.display()))?;
    }
    for f in m.files.iter().filter(|f| f.kind == FileKind::Sqlite) {
        let conn = rusqlite::Connection::open(target.join(&f.path)).map_err(ctx(&f.path))?;
        let ok: String = conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(ctx(&f.path))?;
        if ok != "ok" {
            return Err(format!("{}: integrity check failed: {ok}", f.path));
        }
    }
    Ok(m)
}

/// Upload a verified snapshot under `<prefix><name>/`, files first, manifest last (a snapshot whose
/// manifest is in the bucket is complete).
pub fn push(s3: &S3Client, snap: &Path, prefix: &str, name: &str) -> Result<SnapshotManifest> {
    let m = verify(snap)?;
    let base = format!("{prefix}{name}/");
    for f in &m.files {
        let p = snap.join("files").join(&f.path);
        let file = File::open(&p).map_err(ctx(p.display()))?;
        s3.put(
            &format!("{base}files/{}", f.path),
            Payload::File(&file),
            &f.sha256,
            "application/octet-stream",
            false,
        )
        .map_err(ctx(format!("upload {}", f.path)))?;
    }
    let text = std::fs::read(snap.join("manifest.json")).map_err(ctx("manifest.json"))?;
    let sha = hex::encode(Sha256::digest(&text));
    match s3.put(
        &format!("{base}manifest.json"),
        Payload::Bytes(&text),
        &sha,
        "application/json",
        true,
    ) {
        Ok(()) => Ok(m),
        Err(S3Error::Exists) => Err(format!("snapshot {name} already exists in the bucket")),
        Err(e) => Err(format!("upload manifest: {e}")),
    }
}

/// Download `<prefix><name>/` into the new directory `out` and verify it.
pub fn pull(s3: &S3Client, prefix: &str, name: &str, out: &Path) -> Result<SnapshotManifest> {
    if out.exists() {
        return Err(format!("{} already exists", out.display()));
    }
    let base = format!("{prefix}{name}/");
    let text = s3
        .get_bytes(&format!("{base}manifest.json"), 64 << 20)
        .map_err(ctx(format!("snapshot {name}")))?;
    std::fs::create_dir_all(out.join("files")).map_err(ctx(out.display()))?;
    std::fs::write(out.join("manifest.json"), &text).map_err(ctx("manifest.json"))?;
    let m = read_manifest(out)?;
    for f in &m.files {
        let dst = out.join("files").join(&f.path);
        std::fs::create_dir_all(dst.parent().expect("has parent")).map_err(ctx(dst.display()))?;
        let mut r = s3
            .get_range(&format!("{base}files/{}", f.path), 0, f.bytes)
            .map_err(ctx(format!("download {}", f.path)))?;
        let mut w = File::create(&dst).map_err(ctx(dst.display()))?;
        std::io::copy(&mut r, &mut w).map_err(ctx(dst.display()))?;
        w.sync_all().map_err(ctx(dst.display()))?;
    }
    verify(out)
}

#[cfg(test)]
mod tests {
    use super::excluded;

    #[test]
    fn credentials_and_transient_files_are_never_captured() {
        for p in [
            "state.sqlite-wal",
            "state.sqlite-shm",
            "codex/home/auth.json",
            "uploads/up_1/data",
            "artifacts/staging/put_1.part",
            "x/secret.key",
            "codex/codex.env",
        ] {
            assert!(excluded(p), "{p}");
        }
        for p in [
            "state.sqlite",
            "codex/sessions/2026/x.jsonl",
            "artifacts/objects/ab/abcd",
        ] {
            assert!(!excluded(p), "{p}");
        }
    }
}
