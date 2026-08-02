use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rusqlite::{Connection, OpenFlags};

use crate::Result;

/// A SQLite connection, plus an optional temp copy cleaned up when dropped.
pub(crate) struct Db {
  pub conn: Connection,
  _temp: Option<TempCopy>,
}

/// Open a browser cookie database for reading without disturbing the live file.
///
/// A plain read-only open is tried first: it keeps SQLite's locking and WAL
/// handling intact, so the browser can keep writing and we still read a
/// consistent snapshot. If that fails (e.g. the file is exclusively locked, as
/// on Windows), the database and its `-wal`/`-shm` sidecars are copied to a temp
/// directory and the copy is opened instead.
pub(crate) fn open(path: &Path) -> Result<Db> {
  match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
    Ok(conn) => Ok(Db { conn, _temp: None }),
    Err(_) => open_copy(path),
  }
}

pub(crate) fn open_copy(path: &Path) -> Result<Db> {
  let temp = TempCopy::of(path)?;
  // The copy is ours alone, so a normal read-write open is safe and lets SQLite
  // replay the copied WAL to surface the latest committed cookies.
  let conn = Connection::open(&temp.db)?;
  Ok(Db { conn, _temp: Some(temp) })
}

struct TempCopy {
  dir: PathBuf,
  db: PathBuf,
}

impl TempCopy {
  fn of(path: &Path) -> Result<Self> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);

    let dir = std::env::temp_dir().join(format!("unjar-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir)?;

    let name = path.file_name().ok_or("invalid database path")?;
    let db = dir.join(name);
    std::fs::copy(path, &db)?;

    // Copy the WAL/SHM sidecars if present so no committed writes are missed.
    for suffix in ["-wal", "-shm"] {
      let src = with_suffix(path.as_os_str(), suffix);
      if Path::new(&src).exists() {
        std::fs::copy(&src, with_suffix(db.as_os_str(), suffix))?;
      }
    }

    Ok(Self { dir, db })
  }
}

impl Drop for TempCopy {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.dir);
  }
}

fn with_suffix(base: &std::ffi::OsStr, suffix: &str) -> PathBuf {
  let mut s = OsString::from(base);
  s.push(suffix);
  PathBuf::from(s)
}
