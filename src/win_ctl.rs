//! メインウィンドウの表示制御（Win32）。
//!
//! egui の `ViewportCommand::Visible(false)` は非表示後にイベントループ／
//! 再描画が止まり、トレイ操作を処理できなくなる。
//! そのため **見た目の表示は ShowWindow のみ** で行い、
//! eframe 側は常に「生きた」状態を保つ。

use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Arc;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    PostMessageW, SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WM_CLOSE,
};

use crate::debug_log;
use crate::single_instance;

/// UI スレッドとトレイイベントハンドラで共有する。
#[derive(Debug)]
pub struct WinCtl {
    hwnd: AtomicIsize,
    /// ユーザーから見て「出している」か（SW_HIDE 後は false）
    want_visible: AtomicBool,
    /// 直前フレームで前面だったか（トレイクリックでフォーカスが奪われる前の判断用）
    was_foreground: AtomicBool,
    should_quit: AtomicBool,
}

impl WinCtl {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            hwnd: AtomicIsize::new(0),
            want_visible: AtomicBool::new(true),
            was_foreground: AtomicBool::new(true),
            should_quit: AtomicBool::new(false),
        })
    }

    pub fn set_hwnd(&self, hwnd: isize) {
        self.hwnd.store(hwnd, Ordering::Release);
        debug_log::log("win", format!("set_hwnd={hwnd:#x}"));
    }

    pub fn hwnd(&self) -> Option<HWND> {
        let v = self.hwnd.load(Ordering::Acquire);
        if v == 0 {
            None
        } else {
            Some(HWND(v as *mut _))
        }
    }

    pub fn want_visible(&self) -> bool {
        self.want_visible.load(Ordering::Acquire)
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit.load(Ordering::Acquire)
    }

    pub fn set_was_foreground(&self, v: bool) {
        self.was_foreground.store(v, Ordering::Release);
    }

    pub fn is_really_visible(&self) -> bool {
        if let Some(hwnd) = self.hwnd() {
            single_instance::is_hwnd_visible(hwnd)
        } else {
            self.want_visible()
        }
    }

    pub fn is_really_foreground(&self) -> bool {
        self.hwnd()
            .map(single_instance::is_hwnd_foreground)
            .unwrap_or(false)
    }

    /// 表示して前面・最前面へ（トレイ／メニューから直接呼んでよい）。
    pub fn show(&self) {
        self.want_visible.store(true, Ordering::Release);
        let Some(hwnd) = self.hwnd() else {
            debug_log::log("win", "show: hwnd not set yet");
            return;
        };
        debug_log::log(
            "win",
            format!(
                "show hwnd={hwnd:?} before_vis={}",
                single_instance::is_hwnd_visible(hwnd)
            ),
        );
        single_instance::show_hwnd(hwnd);
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
        }
        wake_hwnd(hwnd);
        debug_log::log(
            "win",
            format!(
                "show done after_vis={}",
                single_instance::is_hwnd_visible(hwnd)
            ),
        );
    }

    /// 隠す。**egui Visible(false) は使わない**（ループ停止防止）。
    pub fn hide(&self) {
        self.want_visible.store(false, Ordering::Release);
        let Some(hwnd) = self.hwnd() else {
            debug_log::log("win", "hide: hwnd not set yet");
            return;
        };
        debug_log::log("win", format!("hide hwnd={hwnd:?}"));
        single_instance::hide_hwnd(hwnd);
        wake_hwnd(hwnd);
        debug_log::log(
            "win",
            format!(
                "hide done after_vis={}",
                single_instance::is_hwnd_visible(hwnd)
            ),
        );
    }

    /// トレイ左クリック用。
    pub fn toggle(&self) {
        let visible = self.want_visible() || self.is_really_visible();
        let foreground = self.was_foreground.load(Ordering::Acquire);
        debug_log::log(
            "win",
            format!(
                "toggle want_vis={} really_vis={} was_fg={} → {}",
                self.want_visible(),
                self.is_really_visible(),
                foreground,
                if visible && foreground {
                    "hide"
                } else {
                    "show"
                }
            ),
        );
        if visible && foreground {
            self.hide();
        } else {
            self.show();
        }
    }

    /// 終了要求。トレイアイコンの drop は呼び出し側。
    pub fn request_quit(&self) {
        debug_log::log("win", "request_quit");
        self.should_quit.store(true, Ordering::Release);
        let Some(hwnd) = self.hwnd() else {
            debug_log::log("win", "request_quit: no hwnd");
            return;
        };
        single_instance::show_hwnd(hwnd);
        self.want_visible.store(true, Ordering::Release);
        unsafe {
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
        debug_log::log("win", "request_quit: WM_CLOSE posted");
    }
}

fn wake_hwnd(hwnd: HWND) {
    unsafe {
        let _ = PostMessageW(Some(hwnd), 0x0000 /* WM_NULL */, WPARAM(0), LPARAM(0));
    }
}
