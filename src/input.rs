//! アクティブな外部ウィンドウへの Unicode 文字送信（Windows）。

use std::sync::Mutex;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentProcessId, GetCurrentThreadId,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
};

/// 自プロセス以外で最後に前面だったウィンドウ。
static TARGET_HWND: Mutex<Option<isize>> = Mutex::new(None);

/// フォアグラウンドが外部アプリならターゲットとして記録する。
pub fn track_foreground() {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return;
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 || pid == GetCurrentProcessId() {
            return;
        }

        if let Ok(mut guard) = TARGET_HWND.lock() {
            *guard = Some(hwnd.0 as isize);
        }
    }
}

/// 記録済みターゲットへフォーカスを戻し、文字列を SendInput で送る。
/// 単一文字でも、絵文字（複数符号点 / サロゲートペア）でも可。
pub fn type_text(text: &str) -> bool {
    track_foreground();

    let target = TARGET_HWND
        .lock()
        .ok()
        .and_then(|g| *g)
        .map(|v| HWND(v as *mut _));

    if let Some(hwnd) = target {
        if !hwnd.0.is_null() {
            focus_window(hwnd);
            // フォーカス切替後に少し間を置くと一部エディタで安定する
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
    }

    send_unicode_text(text)
}

fn focus_window(hwnd: HWND) {
    unsafe {
        let target_thread = GetWindowThreadProcessId(hwnd, None);
        let current_thread = GetCurrentThreadId();

        if target_thread != 0 && target_thread != current_thread {
            let _ = AttachThreadInput(current_thread, target_thread, true);
            let _ = SetForegroundWindow(hwnd);
            let _ = AttachThreadInput(current_thread, target_thread, false);
        } else {
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

/// KEYEVENTF_UNICODE で UTF-16 単位を順に送る（サロゲート・VS16 含む）。
fn send_unicode_text(text: &str) -> bool {
    let mut ok = true;
    for unit in text.encode_utf16() {
        ok &= send_unicode_unit(unit);
    }
    ok
}

fn send_unicode_unit(unit: u16) -> bool {
    let inputs = [
        keyboard_input(unit, KEYEVENTF_UNICODE),
        keyboard_input(unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP),
    ];

    unsafe {
        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        sent == inputs.len() as u32
    }
}

fn keyboard_input(
    scan: u16,
    flags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS,
) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: Default::default(),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
