use nih_plug::prelude::{Editor, ParamSetter};
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2},
    EguiState,
};
use std::sync::Arc;

use super::theme;
use crate::params::BddPlusParams;

// 現在のテクスチャマネージャを識別する。
fn tex_manager_id(ctx: &egui::Context) -> usize {
    Arc::as_ptr(&ctx.tex_manager()) as usize
}

// GUIの描画で使う一時状態。
struct UiState {
    // 背景テクスチャのハンドル。
    bg_texture: Option<egui::TextureHandle>,
    // 背景画像のピクセルサイズ。
    bg_size_px: [usize; 2],
    // テクスチャマネージャの識別子。
    bg_tex_manager_id: usize,
}

// 背景画像をUI全体に対して縮小する比率。
const BG_SCALE: f32 = 0.6;

// GUIエディタのエントリポイント。
pub(super) fn create_editor(
    // プラグインのパラメータ。
    params: Arc<BddPlusParams>,
) -> Option<Box<dyn Editor>> {
    // 背景画像のピクセルサイズ。
    let bg_size_px = bg_size_px();
    // 初期のウィンドウサイズを決める状態。
    let egui_state = EguiState::from_size(
        (bg_size_px[0] as f32 * BG_SCALE) as u32,
        (bg_size_px[1] as f32 * BG_SCALE) as u32,
    );
    create_egui_editor(
        egui_state,
        UiState {
            // まだテクスチャは読み込まれていない。
            bg_texture: None,
            // 背景サイズは先に設定する。
            bg_size_px,
            // 初期は未設定。
            bg_tex_manager_id: 0,
        },
        |ctx, state| {
            // ウィンドウ生成のたびに背景テクスチャを読み込む。
            let (texture, size) = load_bg_texture(ctx);
            state.bg_texture = Some(texture);
            state.bg_size_px = size;
            state.bg_tex_manager_id = tex_manager_id(ctx);
        },
        move |ctx, setter, state| {
            let tex_manager_id = tex_manager_id(ctx);
            // テクスチャが破棄されていたら再読み込みする。
            let needs_reload = state.bg_tex_manager_id != tex_manager_id
                || match &state.bg_texture {
                    Some(texture) => texture.size() == [0, 0],
                    None => true,
                };
            if needs_reload {
                let (texture, size) = load_bg_texture(ctx);
                state.bg_texture = Some(texture);
                state.bg_size_px = size;
                state.bg_tex_manager_id = tex_manager_id;
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                // DPI変換用の係数。
                let pixels_per_point = ui.ctx().pixels_per_point();
                // 背景サイズ(ポイント単位)。
                let bg_size_points = Vec2::new(
                    state.bg_size_px[0] as f32 / pixels_per_point,
                    state.bg_size_px[1] as f32 / pixels_per_point,
                ) * BG_SCALE;
                // 最小サイズを背景に合わせる。
                ui.set_min_size(bg_size_points);
                // パネルの領域。
                let panel_rect = ui.available_rect_before_wrap();
                // 背景の描画領域。
                let bg_rect = Rect::from_min_size(panel_rect.min, bg_size_points);

                // 背景はパネル全体に描画する。
                draw_background(ui, state, bg_rect);

                // UI本体は背景の矩形に合わせて配置する。
                ui.allocate_ui_at_rect(bg_rect, |ui| {
                    draw_fixed_layout(ui, setter, &params, bg_rect, pixels_per_point);
                });
            });
        },
    )
}

// 背景画像を描画する。
fn draw_background(
    // 描画対象のUI。
    ui: &egui::Ui,
    // 背景状態。
    state: &UiState,
    // 背景を描く矩形。
    rect: Rect,
) {
    // 描画用ペインタ。
    let painter = ui.painter();
    if let Some(texture) = &state.bg_texture { // 背景テクスチャ。
        // 読み込んだ背景画像を描画する。
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        // 背景がない場合は単色で埋める。
        painter.rect_filled(rect, 0.0, Color32::from_rgb(45, 42, 38));
    }
}

// 背景上に3本スライダーの固定レイアウトを描画する。
fn draw_fixed_layout(
    // 描画対象のUI。
    ui: &mut egui::Ui,
    // パラメータ変更用のセッター。
    setter: &ParamSetter,
    // プラグインパラメータ。
    params: &BddPlusParams,
    // 背景矩形。
    rect: Rect,
    // DPI変換係数。
    pixels_per_point: f32,
) {
    // DPIに応じたスケール。
    let scale = 1.0 / pixels_per_point;
    // パネル内側の余白。
    let padding = theme::PANEL_PADDING * scale;
    // 1列の幅。
    let column_width = theme::COLUMN_WIDTH * scale;
    // 列間の隙間。
    let column_gap = 12.0 * scale;
    // 値表示の高さ。
    let value_height = theme::VALUE_HEIGHT * scale;
    // スライダーの高さ。
    let slider_height = theme::SLIDER_HEIGHT * scale;
    // スライダーの幅。
    let slider_width = theme::SLIDER_WIDTH * scale;
    // 値表示とスライダーの間隔。
    let value_gap = 6.0 * scale;
    // 1列の総高さ。
    let column_height = value_height + value_gap + slider_height;

    // 3列分の総幅。
    let total_width = (column_width * 3.0) + (column_gap * 2.0);
    // 左端の開始位置。
    let start_x = (rect.width() - total_width) * 0.5;
    // 上端の開始位置。
    let start_y = padding;
    // 背景矩形の原点。
    let base = rect.min;

    // スライダーの構成(パラメータとラベル)。
    let columns = [(&params.drive), (&params.tone), (&params.level)];

    for (idx, param) in columns.iter().enumerate() {
        // 列のX座標。
        let x = start_x + (idx as f32 * (column_width + column_gap));
        // 列の描画矩形。
        let column_rect = Rect::from_min_size(
            Pos2::new(base.x + x, base.y + start_y),
            Vec2::new(column_width, column_height),
        );
        draw_column_at(
            ui,
            setter,
            param,
            idx,
            column_rect,
            slider_width,
            value_height,
            value_gap,
            scale,
        );
    }
}

// 1列分の値表示とスライダーを描画する。
fn draw_column_at(
    // 描画対象のUI。
    ui: &mut egui::Ui,
    // パラメータ変更用のセッター。
    setter: &ParamSetter,
    // 対象パラメータ。
    param: &nih_plug::params::FloatParam,
    // eguiのIDに使う列番号。
    column_idx: usize,
    // 列の描画矩形。
    rect: Rect,
    // スライダー幅。
    slider_width: f32,
    // 値表示高さ。
    value_height: f32,
    // 値表示の隙間。
    value_gap: f32,
    // DPIスケール。
    scale: f32,
) {
    // 値表示の矩形。
    let value_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), value_height));
    draw_value_window_at(ui, value_rect, param.value(), scale);

    // スライダーの矩形。
    let slider_rect = Rect::from_min_size(
        Pos2::new(
            rect.center().x - (slider_width * 0.5),
            value_rect.bottom() + value_gap,
        ),
        Vec2::new(slider_width, theme::SLIDER_HEIGHT * scale),
    );
    draw_slider_at(ui, setter, param, slider_rect, scale, column_idx);
}

// パラメータ値の小窓を描画する。
fn draw_value_window_at(
    // 描画対象のUI。
    ui: &egui::Ui,
    // 値表示矩形。
    rect: Rect,
    // パラメータ値。
    value: f32,
    // DPIスケール。
    scale: f32,
) {
    // 描画用ペインタ。
    let painter = ui.painter();
    painter.rect_filled(rect, theme::VALUE_RADIUS * scale, theme::VALUE_BG);
    painter.rect_filled(
        rect.shrink(2.0 * scale),
        theme::VALUE_RADIUS * scale,
        theme::VALUE_BG_INNER,
    );

    // 0..100の表示用値。
    let value_text = (value.clamp(0.0, 1.0) * 100.0).round() as i32;
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        format!("{value_text:>3}"),
        FontId::proportional(14.0 * scale),
        theme::TEXT_VALUE,
    );
}

// 縦型スライダーを描画してドラッグ操作を処理する。
fn draw_slider_at(
    // 描画対象のUI。
    ui: &mut egui::Ui,
    // パラメータ変更用のセッター。
    setter: &ParamSetter,
    // 対象パラメータ。
    param: &nih_plug::params::FloatParam,
    // スライダー矩形。
    rect: Rect,
    // DPIスケール。
    scale: f32,
    // eguiのIDに使う列番号。
    column_idx: usize,
) {
    // マウス操作のレスポンス。
    let response = ui.interact(rect, ui.id().with(column_idx), Sense::click_and_drag());
    // 描画用ペインタ。
    let painter = ui.painter();

    // スライダートラックの矩形。
    let track_rect = Rect::from_center_size(rect.center(), Vec2::new(6.0, rect.height()));
    painter.rect_filled(track_rect, 3.0, theme::SLIDER_TRACK);

    // 現在のパラメータ値。
    let value = param.value().clamp(0.0, 1.0);
    // ノブのY座標。
    let knob_y = rect.bottom() - (value * rect.height());
    // ノブの矩形。
    let knob_rect = Rect::from_center_size(
        Pos2::new(rect.center().x, knob_y),
        Vec2::new(rect.width(), 14.0 * scale),
    );
    painter.rect_filled(knob_rect, 7.0, theme::SLIDER_KNOB);

    if response.drag_started() {
        setter.begin_set_parameter(param);
    }
    if response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() { // マウス位置。
            // ドラッグ位置から新しい値を計算する。
            let new_value = ((rect.bottom() - pos.y) / rect.height()).clamp(0.0, 1.0);
            setter.set_parameter(param, new_value);
        }
    }
    if response.drag_stopped() {
        setter.end_set_parameter(param);
    }
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() { // マウス位置。
            // クリック位置から新しい値を計算する。
            let new_value = ((rect.bottom() - pos.y) / rect.height()).clamp(0.0, 1.0);
            setter.begin_set_parameter(param);
            setter.set_parameter(param, new_value);
            setter.end_set_parameter(param);
        }
    }
}

// 背景画像のピクセルサイズを返す。
fn bg_size_px() -> [usize; 2] {
    // 背景画像バイト列。
    const BG_BYTES: &[u8] = include_bytes!("../../assets/bg.png");
    image::load_from_memory(BG_BYTES)
        .ok()
        .map(|img| [img.width() as usize, img.height() as usize])
        .unwrap_or([256, 256])
}

// 背景画像のテクスチャを読み込み、ない場合は代替画像を作る。
fn load_bg_texture(
    // eguiのコンテキスト。
    ctx: &egui::Context,
) -> (egui::TextureHandle, [usize; 2]) {
    // 背景画像バイト列。
    const BG_BYTES: &[u8] = include_bytes!("../../assets/bg.png");
    // 画像をRGBAに変換する。
    let image = image::load_from_memory(BG_BYTES)
        .ok()
        .map(|img| img.to_rgba8());

    // eguiで使う画像とサイズ。
    let (color_image, size) = if let Some(image) = image { // 読み込んだ画像。
        // 背景画像のサイズ。
        let size = [image.width() as usize, image.height() as usize];
        (
            egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw()),
            size,
        )
    } else {
        // 代替画像のサイズ。
        let size = [256, 256];
        // 代替画像のピクセル配列。
        let mut pixels = Vec::with_capacity(size[0] * size[1]);
        for y in 0..size[1] { // Y方向の走査。
            for x in 0..size[0] { // X方向の走査。
                // 擬似的なノイズ値。
                let n = ((x * 13 + y * 17) & 0x1f) as u8;
                // 赤成分。
                let r = 60 + n;
                // 緑成分。
                let g = 54 + (n / 2);
                // 青成分。
                let b = 48 + (n / 3);
                pixels.push(Color32::from_rgb(r, g, b));
            }
        }
        (egui::ColorImage { size, pixels }, size)
    };

    // eguiにテクスチャとして登録する。
    (
        ctx.load_texture("bdd_plus_bg", color_image, egui::TextureOptions::LINEAR),
        size,
    )
}
