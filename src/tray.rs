//! システムトレイ（タスクトレイ）常駐。

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::debug_log;
use crate::win_ctl::WinCtl;

const ID_SHOW: &str = "keisen.show";
const ID_HIDE: &str = "keisen.hide";
const ID_QUIT: &str = "keisen.quit";
#[cfg(debug_assertions)]
const ID_OPEN_LOG: &str = "keisen.open_log";

/// ダブルクリック時の DOWN/UP 連打でトグルが 2 回走らないようにする。
const TOGGLE_DEBOUNCE: Duration = Duration::from_millis(350);

/// トレイ本体。
pub struct TrayHandle {
    /// ドロップするとトレイが消えるので保持する
    #[allow(dead_code)]
    tray: TrayIcon,
    /// MenuItem をドロップするとメニュー項目が無効になる場合があるため保持
    #[allow(dead_code)]
    items: Vec<MenuItem>,
}

impl TrayHandle {
    /// メイン UI スレッド上で構築する（Windows では必須）。
    ///
    /// 表示／非表示／終了は **ハンドラ内で WinCtl を直接操作**する。
    pub fn create(
        win: Arc<WinCtl>,
        request_repaint: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, String> {
        let icon = load_tray_icon().map_err(|e| format!("tray icon: {e}"))?;

        let show = MenuItem::with_id(ID_SHOW, "表示", true, None);
        let hide = MenuItem::with_id(ID_HIDE, "隠す", true, None);
        let quit = MenuItem::with_id(ID_QUIT, "終了", true, None);

        let menu = Menu::new();
        menu.append(&show)
            .map_err(|e| format!("tray menu append: {e}"))?;
        menu.append(&hide)
            .map_err(|e| format!("tray menu append: {e}"))?;
        menu.append(&PredefinedMenuItem::separator())
            .map_err(|e| format!("tray menu append: {e}"))?;

        #[cfg(debug_assertions)]
        let open_log = MenuItem::with_id(ID_OPEN_LOG, "デバッグログを開く", true, None);
        #[cfg(debug_assertions)]
        {
            menu.append(&open_log)
                .map_err(|e| format!("tray menu append: {e}"))?;
            menu.append(&PredefinedMenuItem::separator())
                .map_err(|e| format!("tray menu append: {e}"))?;
        }

        menu.append(&quit)
            .map_err(|e| format!("tray menu append: {e}"))?;

        #[cfg(debug_assertions)]
        let items = vec![show, hide, open_log, quit];
        #[cfg(not(debug_assertions))]
        let items = vec![show, hide, quit];

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("罫線 — 左クリックで表示/非表示")
            .with_icon(icon)
            .with_menu_on_left_click(false)
            .with_menu_on_right_click(true)
            .build()
            .map_err(|e| format!("tray build: {e}"))?;

        debug_log::log("tray", "icon created");

        let repaint = Arc::new(request_repaint);
        let last_toggle = Arc::new(Mutex::new(None::<Instant>));

        let win_tray = win.clone();
        let repaint_tray = repaint.clone();
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            // 全イベントをログ（非表示中に届いているか切り分け）
            debug_log::log("tray_evt", format!("{event:?}"));

            let is_left_up = matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            );
            if !is_left_up {
                return;
            }

            let now = Instant::now();
            if let Ok(mut last) = last_toggle.lock() {
                if let Some(t) = *last {
                    if now.duration_since(t) < TOGGLE_DEBOUNCE {
                        debug_log::log("tray", "left-up debounced");
                        return;
                    }
                }
                *last = Some(now);
            }

            debug_log::log("tray", "left-up → toggle");
            win_tray.toggle();
            repaint_tray();
        }));

        let win_menu = win.clone();
        let repaint_menu = repaint;
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            debug_log::log("menu_evt", format!("id={}", event.id.0));

            if event.id == ID_SHOW {
                debug_log::log("menu", "show");
                win_menu.show();
            } else if event.id == ID_HIDE {
                debug_log::log("menu", "hide");
                win_menu.hide();
            } else if event.id == ID_QUIT {
                debug_log::log("menu", "quit");
                win_menu.request_quit();
            } else {
                #[cfg(debug_assertions)]
                if event.id == ID_OPEN_LOG {
                    debug_log::log("menu", "open_log");
                    debug_log::open_log_file();
                    repaint_menu();
                    return;
                }
                debug_log::log("menu", format!("unknown id={}", event.id.0));
                return;
            }
            repaint_menu();
        }));

        Ok(Self { tray, items })
    }
}

fn load_tray_icon() -> Result<Icon, String> {
    let image = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .map_err(|e| e.to_string())?
        .into_rgba8();
    let resized = image::imageops::resize(&image, 32, 32, image::imageops::FilterType::Lanczos3);
    let (width, height) = resized.dimensions();
    Icon::from_rgba(resized.into_raw(), width, height).map_err(|e| e.to_string())
}
