//! 罫線入力フローティングアプリ
//! 常時最前面のパレットから、アクティブなエディタへ罫線文字を送る。
//! タスクトレイに常駐し、× は終了ではなく非表示にする。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod chars;
mod debug_log;
mod input;
mod pump_keepalive;
mod single_instance;
mod tray;
mod win_ctl;

use std::sync::Arc;

use chars::SECTIONS;
use eframe::egui::{self, ViewportCommand};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use single_instance::WINDOW_TITLE;
use tray::TrayHandle;
use win_ctl::WinCtl;

fn main() -> eframe::Result<()> {
    debug_log::init();
    debug_log::log("main", "start");

    if !single_instance::try_acquire() {
        debug_log::log("main", "another instance running → exit");
        // 2 つ目のプロセス: 既存を前面に出して静かに終了
        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([300.0, 380.0])
            .with_min_inner_size([240.0, 260.0])
            .with_decorations(false) // 枠なしフロート
            .with_taskbar(false) // タスクバーに出さない
            .with_always_on_top()
            .with_transparent(false)
            .with_resizable(true)
            .with_title(WINDOW_TITLE)
            .with_icon(app_icon()),
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            configure_style(&cc.egui_ctx);

            let win = WinCtl::new();
            let keepalive = pump_keepalive::start(win.clone());

            let ctx = cc.egui_ctx.clone();
            let tray = TrayHandle::create(win.clone(), move || {
                ctx.request_repaint();
            })
            .unwrap_or_else(|e| {
                eprintln!("トレイ作成に失敗しました: {e}");
                panic!("system tray is required: {e}");
            });

            debug_log::log(
                "main",
                format!("ui ready log={}", debug_log::path_string()),
            );

            #[cfg(debug_assertions)]
            let last_status = format!("log: {}", debug_log::path_string());
            #[cfg(not(debug_assertions))]
            let last_status = String::new();

            Ok(Box::new(KeisenApp {
                tray: Some(tray),
                win,
                last_status,
                _keepalive: keepalive,
            }))
        }),
    )
}

/// ダークグレーのキートップに白い「田」
fn app_icon() -> egui::IconData {
    let image = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("embedded app icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

/// 罫線表示用のフォントファミリー名
const FONT_KEISEN: &str = "keisen";

/// 埋め込みフォントは使わず、Windows のシステムフォントだけを実行時ロードする。
fn configure_fonts(ctx: &egui::Context) {
    // default_fonts feature 無効時は empty 相当
    let mut fonts = egui::FontDefinitions::empty();

    // 罫線（Box Drawing）を含む可能性が高いもの。TTF を優先（TTC は顔選択で欠けることがある）。
    let box_drawing_fonts = [
        ("cascadia_mono", r"C:\Windows\Fonts\CascadiaMono.ttf"),
        ("cascadia_code", r"C:\Windows\Fonts\CascadiaCode.ttf"),
        ("consolas", r"C:\Windows\Fonts\consola.ttf"),
        ("segoe_ui_symbol", r"C:\Windows\Fonts\seguisym.ttf"),
        ("segoe_ui", r"C:\Windows\Fonts\segoeui.ttf"),
    ];
    // UI ラベル（日本語）用
    let jp_fonts = [
        ("noto_sans_jp", r"C:\Windows\Fonts\NotoSansJP-VF.ttf"),
        ("yu_gothic", r"C:\Windows\Fonts\YuGothM.ttc"),
        ("meiryo", r"C:\Windows\Fonts\meiryo.ttc"),
        ("ms_gothic", r"C:\Windows\Fonts\msgothic.ttc"),
        ("biz_ud_gothic", r"C:\Windows\Fonts\BIZ-UDGothicR.ttc"),
    ];

    let mut box_names: Vec<String> = Vec::new();
    let mut jp_names: Vec<String> = Vec::new();

    for (name, path) in box_drawing_fonts {
        if let Ok(data) = std::fs::read(path) {
            fonts
                .font_data
                .insert(name.to_owned(), egui::FontData::from_owned(data).into());
            box_names.push(name.to_owned());
        }
    }
    for (name, path) in jp_fonts {
        if let Ok(data) = std::fs::read(path) {
            fonts
                .font_data
                .insert(name.to_owned(), egui::FontData::from_owned(data).into());
            jp_names.push(name.to_owned());
        }
    }

    if box_names.is_empty() && jp_names.is_empty() {
        panic!(
            "システムフォントを読み込めませんでした。C:\\Windows\\Fonts に Cascadia / 游ゴシック / メイリオ等があるか確認してください。"
        );
    }

    // Proportional: 日本語 → 罫線系
    let mut proportional = jp_names.clone();
    for n in &box_names {
        if !proportional.contains(n) {
            proportional.push(n.clone());
        }
    }

    // Monospace / 罫線ボタン: 罫線系 → 日本語
    let mut monospace = box_names.clone();
    for n in &jp_names {
        if !monospace.contains(n) {
            monospace.push(n.clone());
        }
    }

    fonts
        .families
        .insert(egui::FontFamily::Proportional, proportional.clone());
    fonts
        .families
        .insert(egui::FontFamily::Monospace, monospace.clone());
    fonts
        .families
        .insert(egui::FontFamily::Name(FONT_KEISEN.into()), monospace);

    ctx.set_fonts(fonts);
}

fn configure_style(ctx: &egui::Context) {
    // ライトテーマの白ボタン + 薄い文字で消えるのを防ぐため、強制ダーク
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(32, 34, 38);
    visuals.extreme_bg_color = egui::Color32::from_rgb(24, 26, 30);
    visuals.override_text_color = Some(egui::Color32::from_rgb(230, 235, 240));
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(55, 60, 68);
    visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(230, 235, 240));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 80, 95);
    visuals.widgets.hovered.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(90, 110, 140);
    visuals.widgets.active.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));
    visuals.selection.bg_fill = egui::Color32::from_rgb(70, 100, 150);

    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(4.0, 4.0);
    style.spacing.button_padding = egui::vec2(6.0, 4.0);
    ctx.set_style(style);
}

struct KeisenApp {
    /// ドロップするとトレイが消えるので保持する
    tray: Option<TrayHandle>,
    /// トレイハンドラと共有する表示制御（Win32 ShowWindow のみで隠す）
    win: Arc<WinCtl>,
    last_status: String,
    /// メッセージポンプ起こし（Drop で停止）
    _keepalive: pump_keepalive::KeepaliveHandle,
}

impl KeisenApp {
    fn capture_hwnd(&self, frame: &eframe::Frame) {
        if self.win.hwnd().is_some() {
            return;
        }
        let Ok(handle) = frame.window_handle() else {
            return;
        };
        if let RawWindowHandle::Win32(h) = handle.as_raw() {
            self.win.set_hwnd(h.hwnd.get());
        }
    }

    fn hide_to_tray(&self) {
        // ViewportCommand::Visible(false) は使わない（eframe ループ停止の原因）
        debug_log::log("ui", "hide_to_tray (× or close)");
        self.win.hide();
    }
}

impl eframe::App for KeisenApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // 枠なし窓の背景（暗いパレット）
        egui::Rgba::from_srgba_unmultiplied(32, 34, 38, 255).to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        debug_log::note_update();
        // 心拍は debug_log 内 tick。詳細は 2 秒に 1 回だけ
        static LAST_HB: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let tick = debug_log::update_tick();
        let last = LAST_HB.load(std::sync::atomic::Ordering::Relaxed);
        if tick.saturating_sub(last) >= 120 {
            // ~2s if ~60fps; もっと粗くてもよい
            LAST_HB.store(tick, std::sync::atomic::Ordering::Relaxed);
            debug_log::log(
                "update",
                format!(
                    "heartbeat want_vis={} really_vis={} fg={}",
                    self.win.want_visible(),
                    self.win.is_really_visible(),
                    self.win.is_really_foreground()
                ),
            );
        }

        self.capture_hwnd(frame);
        input::track_foreground();

        // 終了要求（トレイから）。トレイアイコンを落として Close を通す
        if self.win.should_quit() {
            if self.tray.take().is_some() {
                debug_log::log("ui", "should_quit → Close + drop tray");
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }

        // × / Alt+F4 等はトレイへ格納（「終了」メニュー時のみ本当に閉じる）
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.win.should_quit() {
                debug_log::log("ui", "close_requested (quit)");
                self.tray.take();
            } else {
                debug_log::log("ui", "close_requested → CancelClose + hide");
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                self.hide_to_tray();
            }
        }

        // 非表示中も update を回す保険（keepalive の PostMessage と併用）
        ctx.request_repaint_after(std::time::Duration::from_millis(200));

        // フロート用タイトルバー（ドラッグ移動 + トレイへ隠す）
        egui::TopBottomPanel::top("float_title")
            .exact_height(32.0)
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(24, 26, 30))
                    .inner_margin(egui::Margin::symmetric(8, 4)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("罫線")
                            .strong()
                            .size(13.0)
                            .color(egui::Color32::from_rgb(200, 210, 220)),
                    );

                    // ドラッグ領域（余白をつかんで移動）
                    let drag_size = egui::vec2(ui.available_width() - 28.0, ui.available_height());
                    let drag = ui.allocate_response(drag_size, egui::Sense::click_and_drag());
                    if drag.drag_started_by(egui::PointerButton::Primary) {
                        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                    }

                    let close = ui.add_sized(
                        [24.0, 22.0],
                        egui::Button::new(
                            egui::RichText::new("×")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(200, 200, 200)),
                        )
                        .frame(false),
                    );
                    if close.clicked() {
                        self.hide_to_tray();
                    }
                    if close.hovered() {
                        ui.painter().rect_filled(
                            close.rect.expand(2.0),
                            4.0,
                            egui::Color32::from_rgb(180, 60, 60),
                        );
                        ui.painter().text(
                            close.rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "×",
                            egui::FontId::proportional(14.0),
                            egui::Color32::WHITE,
                        );
                    }
                });
            });

        // 細い枠線（フロート感）
        let screen = ctx.content_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("float_border"),
        ));
        painter.rect_stroke(
            screen.shrink(0.5),
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 78, 90)),
            egui::StrokeKind::Inside,
        );

        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(24, 26, 30))
                    .inner_margin(egui::Margin::symmetric(8, 4)),
            )
            .show(ctx, |ui| {
                if self.last_status.is_empty() {
                    ui.label(
                        egui::RichText::new("クリックで入力 · トレイに常駐 · × で隠す")
                            .small()
                            .color(egui::Color32::from_rgb(180, 190, 200)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(&self.last_status)
                            .small()
                            .color(egui::Color32::from_rgb(200, 220, 200)),
                    );
                }
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(32, 34, 38))
                    .inner_margin(egui::Margin::same(8)),
            )
            .show(ctx, |ui| {
                let button_size = egui::vec2(34.0, 34.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (section_i, section) in SECTIONS.iter().enumerate() {
                            if section_i > 0 {
                                ui.add_space(8.0);
                            }

                            ui.label(
                                egui::RichText::new(section.title)
                                    .small()
                                    .strong()
                                    .color(egui::Color32::from_rgb(210, 220, 235)),
                            );
                            ui.add_space(2.0);

                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                                for &ch in section.chars {
                                    let label = egui::RichText::new(ch.to_string())
                                        .family(egui::FontFamily::Name(FONT_KEISEN.into()))
                                        .size(20.0)
                                        .color(egui::Color32::from_rgb(245, 248, 250));
                                    let response = ui.add_sized(
                                        button_size,
                                        egui::Button::new(label)
                                            .corner_radius(6.0)
                                            .fill(egui::Color32::from_rgb(55, 60, 68)),
                                    );

                                    let tip = format!("{ch}  U+{:04X}", ch as u32);
                                    let response = response.on_hover_text(tip);

                                    if response.clicked() {
                                        let ok = input::type_char(ch);
                                        self.last_status = if ok {
                                            format!("入力: {ch}")
                                        } else {
                                            format!("送信失敗: {ch}")
                                        };
                                    }
                                }
                            });
                        }

                        // スクロール末尾に少し余白（リサイズグリップと被らないように）
                        ui.add_space(16.0);
                    });

                // 端のリサイズハンドル（枠なしでもサイズ変更できるように）
                draw_resize_handles(ui, ctx);
            });

        // トレイトグル用: クリックでフォーカスが奪われる前の「前面だったか」
        self.win
            .set_was_foreground(self.win.is_really_foreground());
    }
}

/// 右下コーナーでリサイズできるようにする
fn draw_resize_handles(ui: &mut egui::Ui, ctx: &egui::Context) {
    let rect = ui.max_rect();
    let grip = 14.0;
    let corner = egui::Rect::from_min_size(
        egui::pos2(rect.right() - grip, rect.bottom() - grip),
        egui::vec2(grip, grip),
    );

    let response = ui.interact(
        corner,
        ui.id().with("resize_se"),
        egui::Sense::click_and_drag(),
    );
    if response.hovered() || response.dragged() {
        ui.painter().line_segment(
            [
                egui::pos2(corner.left() + 3.0, corner.bottom() - 2.0),
                egui::pos2(corner.right() - 2.0, corner.top() + 3.0),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 110, 120)),
        );
        ui.painter().line_segment(
            [
                egui::pos2(corner.left() + 7.0, corner.bottom() - 2.0),
                egui::pos2(corner.right() - 2.0, corner.top() + 7.0),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 110, 120)),
        );
        ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
    }

    if response.dragged() {
        let delta = response.drag_delta();
        if delta != egui::Vec2::ZERO {
            let size = ctx.input(|i| i.content_rect().size());
            let new_size = (size + delta).max(egui::vec2(240.0, 260.0));
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(new_size));
        }
    }
}
