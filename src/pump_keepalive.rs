//! UI スレッドのメッセージポンプを起こし続ける。
//!
//! ウィンドウを SW_HIDE すると winit/eframe が長時間 Wait に入り、
//! 同じスレッド上のトレイ用 HWND のメッセージが処理されなくなることがある。
//! 別スレッドからメイン HWND へ定期的に WM_NULL を投げてポンプを起こす。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

use crate::debug_log;
use crate::win_ctl::WinCtl;

const INTERVAL: Duration = Duration::from_millis(100);

/// 終了まで動くキープアライブを開始する。
pub fn start(win: Arc<WinCtl>) -> KeepaliveHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_t = stop.clone();

    let join = thread::Builder::new()
        .name("keisen-pump-keepalive".into())
        .spawn(move || {
            debug_log::log("keepalive", "thread started");
            let mut last_tick = debug_log::update_tick();
            let mut stall_logged = false;
            let mut n: u64 = 0;

            while !stop_t.load(Ordering::Relaxed) {
                thread::sleep(INTERVAL);
                n += 1;

                if let Some(hwnd) = win.hwnd() {
                    // スレッドのメッセージ待ちを起こす
                    unsafe {
                        let _ = PostMessageW(
                            Some(hwnd),
                            0x0000, /* WM_NULL */
                            WPARAM(0),
                            LPARAM(0),
                        );
                    }
                }

                // 2 秒ごとに心拍ログ（非表示中に update が死んでいないか）
                if n % 20 == 0 {
                    let tick = debug_log::update_tick();
                    let visible = win.is_really_visible();
                    let want = win.want_visible();
                    let hwnd_set = win.hwnd().is_some();

                    if tick == last_tick {
                        if !stall_logged {
                            debug_log::log(
                                "keepalive",
                                format!(
                                    "STALL? update_tick stuck at {tick} want_vis={want} really_vis={visible} hwnd={hwnd_set}"
                                ),
                            );
                            stall_logged = true;
                        }
                    } else {
                        if stall_logged {
                            debug_log::log(
                                "keepalive",
                                format!("recovered update_tick={tick}"),
                            );
                        }
                        stall_logged = false;
                        last_tick = tick;
                        debug_log::log(
                            "keepalive",
                            format!(
                                "ok tick={tick} want_vis={want} really_vis={visible}"
                            ),
                        );
                    }
                }
            }

            debug_log::log("keepalive", "thread stopped");
        })
        .expect("spawn keepalive");

    KeepaliveHandle {
        stop,
        join: Some(join),
    }
}

pub struct KeepaliveHandle {
    stop: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}

impl Drop for KeepaliveHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

