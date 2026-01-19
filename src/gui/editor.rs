use nih_plug::prelude::{Editor, ParamSetter};
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2},
    EguiState,
};
use std::sync::Arc;

use super::theme;
use crate::params::BddPlusParams;

// GUIで使う状態（背景テクスチャなど）を保持する
struct UiState {
    bg_texture: Option<egui::TextureHandle>,
    bg_size_px: [usize; 2],
}

const BG_SCALE: f32 = 0.6;

// GUIエディタのエントリポイント
pub(super) fn create_editor(params: Arc<BddPlusParams>) -> Option<Box<dyn Editor>> {
    let bg_size_px = bg_size_px();
    let egui_state = EguiState::from_size(
        (bg_size_px[0] as f32 * BG_SCALE) as u32,
        (bg_size_px[1] as f32 * BG_SCALE) as u32,
    );
    create_egui_editor(
        egui_state,
        UiState {
            bg_texture: None,
            bg_size_px,
        },
        |ctx, state| {
            // 初回のみ背景テクスチャを生成・キャッシュする
            if state.bg_texture.is_none() {
                let (texture, size) = load_bg_texture(ctx);
                state.bg_texture = Some(texture);
                state.bg_size_px = size;
            }
        },
        move |ctx, setter, state| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // DPI補正: points <-> pixels の換算に必ず使用する
                let pixels_per_point = ui.ctx().pixels_per_point();
                let bg_size_points = Vec2::new(
                    state.bg_size_px[0] as f32 / pixels_per_point,
                    state.bg_size_px[1] as f32 / pixels_per_point,
                ) * BG_SCALE;
                ui.set_min_size(bg_size_points);
                let panel_rect = ui.available_rect_before_wrap();
                let bg_rect = Rect::from_min_size(panel_rect.min, bg_size_points);

                // 背景は固定サイズ・固定位置で描画する
                draw_background(ui, state, bg_rect);

                // UI配置は背景の矩形を基準に行う
                ui.allocate_ui_at_rect(bg_rect, |ui| {
                    draw_fixed_layout(ui, setter, &params, bg_rect, pixels_per_point);
                });
            });
        },
    )
}

// 背景画像（またはフォールバック）を描画する
fn draw_background(ui: &egui::Ui, state: &UiState, rect: Rect) {
    let painter = ui.painter();
    if let Some(texture) = &state.bg_texture {
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.rect_filled(rect, 0.0, Color32::from_rgb(45, 42, 38));
    }
}

// 背景基準の固定座標でレイアウトを組み立てる
fn draw_fixed_layout(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    params: &BddPlusParams,
    rect: Rect,
    pixels_per_point: f32,
) {
    // すべての寸法は pixels_per_point で補正して扱う
    let scale = 1.0 / pixels_per_point;
    let padding = theme::PANEL_PADDING * scale;
    let column_width = theme::COLUMN_WIDTH * scale;
    let column_gap = 12.0 * scale;
    let value_height = theme::VALUE_HEIGHT * scale;
    let slider_height = theme::SLIDER_HEIGHT * scale;
    let slider_width = theme::SLIDER_WIDTH * scale;
    let value_gap = 6.0 * scale;
    let column_height = value_height + value_gap + slider_height;

    let total_width = (column_width * 3.0) + (column_gap * 2.0);
    let start_x = (rect.width() - total_width) * 0.5;
    let start_y = padding;
    let base = rect.min;

    // スライダー列の並び順
    let columns = [
        (&params.drive, "DRIVE"),
        (&params.tone, "TONE"),
        (&params.level, "LEVEL"),
    ];

    for (idx, (param, label)) in columns.iter().enumerate() {
        let x = start_x + (idx as f32 * (column_width + column_gap));
        let column_rect = Rect::from_min_size(
            Pos2::new(base.x + x, base.y + start_y),
            Vec2::new(column_width, column_height),
        );
        draw_column_at(
            ui,
            setter,
            param,
            label,
            column_rect,
            slider_width,
            value_height,
            value_gap,
            scale,
        );
    }
}

// 1列分の値表示・スライダー・ラベルを描画する
fn draw_column_at(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &nih_plug::params::FloatParam,
    label: &str,
    rect: Rect,
    slider_width: f32,
    value_height: f32,
    value_gap: f32,
    scale: f32,
) {
    let value_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), value_height));
    draw_value_window_at(ui, value_rect, param.value(), scale);

    let slider_rect = Rect::from_min_size(
        Pos2::new(
            rect.center().x - (slider_width * 0.5),
            value_rect.bottom() + value_gap,
        ),
        Vec2::new(slider_width, theme::SLIDER_HEIGHT * scale),
    );
    draw_slider_at(ui, setter, param, slider_rect, scale, label);
}

// 数値表示の“黒い窓”を描画する
fn draw_value_window_at(ui: &egui::Ui, rect: Rect, value: f32, scale: f32) {
    let painter = ui.painter();
    painter.rect_filled(rect, theme::VALUE_RADIUS * scale, theme::VALUE_BG);
    painter.rect_filled(
        rect.shrink(2.0 * scale),
        theme::VALUE_RADIUS * scale,
        theme::VALUE_BG_INNER,
    );

    let value_text = (value.clamp(0.0, 1.0) * 100.0).round() as i32;
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        format!("{value_text:>3}"),
        FontId::proportional(14.0 * scale),
        theme::TEXT_VALUE,
    );
}

// 縦スライダーを固定座標で描画し、ドラッグ操作を処理する
fn draw_slider_at(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &nih_plug::params::FloatParam,
    rect: Rect,
    scale: f32,
    id_source: &str,
) {
    let response = ui.interact(rect, ui.id().with(id_source), Sense::click_and_drag());
    let painter = ui.painter();

    let track_rect = Rect::from_center_size(rect.center(), Vec2::new(6.0, rect.height()));
    painter.rect_filled(track_rect, 3.0, theme::SLIDER_TRACK);

    let value = param.value().clamp(0.0, 1.0);
    let knob_y = rect.bottom() - (value * rect.height());
    let knob_rect = Rect::from_center_size(
        Pos2::new(rect.center().x, knob_y),
        Vec2::new(rect.width(), 14.0 * scale),
    );
    painter.rect_filled(knob_rect, 7.0, theme::SLIDER_KNOB);

    if response.drag_started() {
        setter.begin_set_parameter(param);
    }
    if response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() {
            let new_value = ((rect.bottom() - pos.y) / rect.height()).clamp(0.0, 1.0);
            setter.set_parameter(param, new_value);
        }
    }
    if response.drag_stopped() {
        setter.end_set_parameter(param);
    }
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let new_value = ((rect.bottom() - pos.y) / rect.height()).clamp(0.0, 1.0);
            setter.begin_set_parameter(param);
            setter.set_parameter(param, new_value);
            setter.end_set_parameter(param);
        }
    }
}

// 埋め込み画像のサイズ（px）を取得する
fn bg_size_px() -> [usize; 2] {
    const BG_BYTES: &[u8] = include_bytes!("../../assets/bg.png");
    image::load_from_memory(BG_BYTES)
        .ok()
        .map(|img| [img.width() as usize, img.height() as usize])
        .unwrap_or([256, 256])
}

// 背景テクスチャを生成し、サイズと一緒に返す
fn load_bg_texture(ctx: &egui::Context) -> (egui::TextureHandle, [usize; 2]) {
    const BG_BYTES: &[u8] = include_bytes!("../../assets/bg.png");
    let image = image::load_from_memory(BG_BYTES)
        .ok()
        .map(|img| img.to_rgba8());

    let (color_image, size) = if let Some(image) = image {
        let size = [image.width() as usize, image.height() as usize];
        (
            egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw()),
            size,
        )
    } else {
        let size = [256, 256];
        let mut pixels = Vec::with_capacity(size[0] * size[1]);
        for y in 0..size[1] {
            for x in 0..size[0] {
                let n = ((x * 13 + y * 17) & 0x1f) as u8;
                let r = 60 + n;
                let g = 54 + (n / 2);
                let b = 48 + (n / 3);
                pixels.push(Color32::from_rgb(r, g, b));
            }
        }
        (egui::ColorImage { size, pixels }, size)
    };

    (
        ctx.load_texture("bdd_plus_bg", color_image, egui::TextureOptions::LINEAR),
        size,
    )
}
