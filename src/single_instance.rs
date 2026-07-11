//! 単一起動（名前付き Mutex）。2 つ目は既存ウィンドウを前面に出して終了。

use std::sync::OnceLock;

use windows::core::w;
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HWND};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetForegroundWindow, IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow,
    SW_RESTORE, SW_SHOW,
};

/// ミューテックス名（ユーザーローカル、衝突しにくいプレフィックス）。
const MUTEX_NAME: windows::core::PCWSTR = w!("Local\\keisen_single_instance_v1");

/// ウィンドウタイトル（`eframe::run_native` / ViewportBuilder と一致させる）。
pub const WINDOW_TITLE: &str = "罫線";

/// プロセス終了までハンドルを保持するための置き場（Close しない）。
/// `HANDLE` は `Sync` ではないので生ポインタ値として持つ。
static HELD_MUTEX: OnceLock<isize> = OnceLock::new();

/// 既に起動済みなら `false`（呼び出し側で終了）。初回なら `true`。
pub fn try_acquire() -> bool {
    unsafe {
        let handle = match CreateMutexW(None, true, MUTEX_NAME) {
            Ok(h) => h,
            Err(_) => {
                // 作成失敗時は多重起動防止を諦めて起動を許可
                return true;
            }
        };

        if GetLastError() == ERROR_ALREADY_EXISTS {
            let _ = windows::Win32::Foundation::CloseHandle(handle);
            activate_existing();
            return false;
        }

        // 所有権をプロセス寿命まで維持（明示的に Close しない）
        let _ = HELD_MUTEX.set(handle.0 as isize);
        true
    }
}

fn activate_existing() {
    unsafe {
        // タイトルで検索（枠なしでもタイトル文字列は保持される）
        let title: Vec<u16> = WINDOW_TITLE
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let Ok(hwnd) = FindWindowW(None, windows::core::PCWSTR(title.as_ptr())) else {
            return;
        };
        if hwnd.0.is_null() {
            return;
        }
        show_hwnd(hwnd);
    }
}

/// 既存／自ウィンドウを表示して前面へ。
pub fn show_hwnd(hwnd: HWND) {
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        } else {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }
        let _ = SetForegroundWindow(hwnd);
    }
}

pub fn hide_hwnd(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, windows::Win32::UI::WindowsAndMessaging::SW_HIDE);
    }
}

/// 実際に表示されているか（内部フラグより信頼できる）。
pub fn is_hwnd_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd).as_bool() }
}

/// 前面ウィンドウかどうか。
pub fn is_hwnd_foreground(hwnd: HWND) -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        !fg.0.is_null() && fg.0 == hwnd.0
    }
}
