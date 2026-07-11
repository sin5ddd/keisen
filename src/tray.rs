//! システムトレイ（タスクトレイ）常駐。

use tray_icon::menu::{Menu, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// トレイメニュー項目 ID とアイコン本体。
pub struct TrayHandle {
    /// ドロップするとトレイが消えるので保持する（直接は触らない）
    #[allow(dead_code)]
    pub tray: TrayIcon,
    pub show_id: MenuId,
    pub hide_id: MenuId,
    pub quit_id: MenuId,
}

impl TrayHandle {
    /// メイン UI スレッド上で構築する（Windows では必須）。
    pub fn create() -> Result<Self, String> {
        let icon = load_tray_icon().map_err(|e| format!("tray icon: {e}"))?;

        let show = MenuItem::new("表示", true, None);
        let hide = MenuItem::new("隠す", true, None);
        let quit = MenuItem::new("終了", true, None);

        let menu = Menu::new();
        menu.append(&show)
            .map_err(|e| format!("tray menu append: {e}"))?;
        menu.append(&hide)
            .map_err(|e| format!("tray menu append: {e}"))?;
        menu.append(&PredefinedMenuItem::separator())
            .map_err(|e| format!("tray menu append: {e}"))?;
        menu.append(&quit)
            .map_err(|e| format!("tray menu append: {e}"))?;

        let show_id = show.id().clone();
        let hide_id = hide.id().clone();
        let quit_id = quit.id().clone();

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("罫線 — クリックで表示/非表示")
            .with_icon(icon)
            .build()
            .map_err(|e| format!("tray build: {e}"))?;

        Ok(Self {
            tray,
            show_id,
            hide_id,
            quit_id,
        })
    }
}

fn load_tray_icon() -> Result<Icon, String> {
    let image = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .map_err(|e| e.to_string())?
        .into_rgba8();
    // トレイは小さめが無難。高解像度でも動くが 32x32 に揃える。
    let resized = image::imageops::resize(&image, 32, 32, image::imageops::FilterType::Lanczos3);
    let (width, height) = resized.dimensions();
    Icon::from_rgba(resized.into_raw(), width, height).map_err(|e| e.to_string())
}
