//! 罫線入力フローティングアプリ
//! 常時最前面のパレットから、アクティブなエディタへ罫線文字を送る。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod chars;
mod input;

use chars::SECTIONS;
use eframe::egui::{self, ViewportCommand};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([300.0, 380.0])
            .with_min_inner_size([240.0, 260.0])
            .with_decorations(false) // 枠なしフロート
            .with_taskbar(false) // タスクバーに出さない
            .with_always_on_top()
            .with_transparent(false)
            .with_resizable(true)
            .with_title("罫線"),
        ..Default::default()
    };

    eframe::run_native(
        "罫線",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            configure_style(&cc.egui_ctx);
            Ok(Box::new(KeisenApp::default()))
        }),
    )
}

/// 罫線表示用のフォントファミリー名
const FONT_KEISEN: &str = "keisen";

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 罫線（Box Drawing）を確実に含むフォントを優先して読み込む。
    // TTC は顔の選択で欠けることがあるので、単体 TTF を先に試す。
    let box_drawing_fonts = [
        r"C:\Windows\Fonts\CascadiaMono.ttf",
        r"C:\Windows\Fonts\CascadiaCode.ttf",
        r"C:\Windows\Fonts\consola.ttf",
        r"C:\Windows\Fonts\seguisym.ttf", // Segoe UI Symbol
        r"C:\Windows\Fonts\segoeui.ttf",
    ];
    let jp_fonts = [
        r"C:\Windows\Fonts\NotoSansJP-VF.ttf",
        r"C:\Windows\Fonts\YuGothM.ttc",
        r"C:\Windows\Fonts\meiryo.ttc",
        r"C:\Windows\Fonts\msgothic.ttc",
        r"C:\Windows\Fonts\BIZ-UDGothicR.ttc",
    ];

    let mut loaded_names: Vec<String> = Vec::new();

    for (i, path) in box_drawing_fonts.iter().enumerate() {
        if let Ok(data) = std::fs::read(path) {
            let name = format!("box{i}");
            fonts
                .font_data
                .insert(name.clone(), egui::FontData::from_owned(data).into());
            loaded_names.push(name);
            break; // 1つあれば十分（フォールバックは jp 側）
        }
    }

    for (i, path) in jp_fonts.iter().enumerate() {
        if let Ok(data) = std::fs::read(path) {
            let name = format!("jp{i}");
            fonts
                .font_data
                .insert(name.clone(), egui::FontData::from_owned(data).into());
            loaded_names.push(name);
            break;
        }
    }

    // カスタムファミリー: 罫線フォント → 日本語 → egui デフォルト
    let mut keisen_family = loaded_names.clone();
    // デフォルトの比例フォントも後ろに残す
    if let Some(default_prop) = fonts.families.get(&egui::FontFamily::Proportional) {
        for f in default_prop {
            if !keisen_family.contains(f) {
                keisen_family.push(f.clone());
            }
        }
    }
    fonts
        .families
        .insert(egui::FontFamily::Name(FONT_KEISEN.into()), keisen_family);

    // UI 全体でも日本語・罫線が使えるように先頭へ
    for family in [
        egui::FontFamily::Proportional,
        egui::FontFamily::Monospace,
    ] {
        let list = fonts.families.entry(family).or_default();
        for name in loaded_names.iter().rev() {
            list.insert(0, name.clone());
        }
    }

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

#[derive(Default)]
struct KeisenApp {
    last_status: String,
}

impl eframe::App for KeisenApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // 枠なし窓の背景（暗いパレット）
        egui::Rgba::from_srgba_unmultiplied(32, 34, 38, 255).to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        input::track_foreground();
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // フロート用タイトルバー（ドラッグ移動 + 閉じる）
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
                        ctx.send_viewport_cmd(ViewportCommand::Close);
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
                        egui::RichText::new("クリックで入力 · 下にスクロールで全部")
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
