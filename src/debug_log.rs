//! デバッグ用ログ（**debug ビルドのみ**有効）。
//!
//! `cargo run` / `cargo build` 時: `%TEMP%\keisen-debug.log` に追記。
//! `cargo run --release` / `cargo build --release` 時: 何もしない。
//!
//! 使い方（debug のみ）:
//! 1. アプリ起動 → 一度隠す → トレイ操作
//! 2. ログを開いて `update heartbeat` が止まっていないか、
//!    `tray click` / `menu` が記録されているかを見る

#[cfg(debug_assertions)]
mod imp {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
    /// `update` が呼ばれるたびに増える（ポンプ生存確認）
    static UPDATE_TICK: AtomicU64 = AtomicU64::new(0);

    pub fn init() {
        let path = log_path();
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
        {
            let _ = writeln!(
                f,
                "=== keisen debug log start pid={} ===",
                std::process::id()
            );
            let _ = writeln!(f, "log_path={}", path.display());
            let _ = writeln!(
                f,
                "hint: hide window, then tray click/menu. Check heartbeat vs tray events."
            );
        }

        if let Ok(mut g) = LOG_PATH.lock() {
            *g = Some(path.clone());
        }

        eprintln!("[keisen] debug log: {}", path.display());
        log("init", "debug log ready");
    }

    pub fn log_path() -> PathBuf {
        std::env::temp_dir().join("keisen-debug.log")
    }

    pub fn path_string() -> String {
        log_path().display().to_string()
    }

    pub fn note_update() {
        UPDATE_TICK.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_tick() -> u64 {
        UPDATE_TICK.load(Ordering::Relaxed)
    }

    pub fn log(tag: &str, msg: impl AsRef<str>) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let line = format!(
            "{ts} tick={} [{tag}] {}\n",
            update_tick(),
            msg.as_ref()
        );

        eprint!("[keisen] {line}");

        let path = LOG_PATH
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .unwrap_or_else(log_path);

        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = f.write_all(line.as_bytes());
        }
    }

    pub fn open_log_file() {
        let path = log_path();
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .spawn();
        log("debug", format!("open_log_file {}", path.display()));
    }
}

#[cfg(debug_assertions)]
pub use imp::*;

#[cfg(not(debug_assertions))]
#[allow(dead_code)]
mod release_stub {
    pub fn init() {}
    pub fn path_string() -> String {
        String::new()
    }
    pub fn note_update() {}
    pub fn update_tick() -> u64 {
        0
    }
    pub fn log(_tag: &str, _msg: impl AsRef<str>) {}
    pub fn open_log_file() {}
}

#[cfg(not(debug_assertions))]
pub use release_stub::*;
