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
// 背景テクスチャを再読み込みする。
fn refresh_bg_texture(state: &mut UiState, ctx: &egui::Context) {
    let (texture, size) = load_bg_texture(ctx);
    state.bg_texture = Some(texture);
    state.bg_size_px = size;
    state.bg_tex_manager_id = tex_manager_id(ctx);
}
// ピクセル座標からRectを作成する。
fn rect_from_px(base: Pos2, scale: f32, px: (f32, f32, f32, f32)) -> Rect {
    Rect::from_min_size(
        Pos2::new(base.x + px.0 * scale, base.y + px.1 * scale),
        Vec2::new(px.2 * scale, px.3 * scale),
    )
}

// 背景画像をUI全体に対して縮小する比率。
const BG_SCALE: f32 = 0.6;

// 値表示窓の矩形 (背景画像のピクセル座標)。
const VALUE_WINDOWS_PX: [(f32, f32, f32, f32); 3] = [
    (120.0, 157.0, 205.0, 114.0),
    (392.0, 156.0, 206.0, 115.0),
    (667.0, 158.0, 208.0, 113.0),
];

// スライダー溝の矩形 (背景画像のピクセル座標)。
const SLIDER_SLOTS_PX: [(f32, f32, f32, f32); 3] = [
    (207.0, 325.0, 54.0, 624.0),
    (477.0, 325.0, 58.0, 624.0),
    (753.0, 325.0, 54.0, 624.0),
];

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
            refresh_bg_texture(state, ctx);
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
                refresh_bg_texture(state, ctx);
            }
            // 中央パネルにUIを描画する。
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
    if let Some(texture) = &state.bg_texture {
        // 背景テクスチャ。
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
    // DPIと背景スケールを反映した係数。
    let scale = BG_SCALE / pixels_per_point;
    // 背景矩形の原点。
    let base = rect.min;

    // スライダーの構成(パラメータとラベル)。
    let columns = [(&params.drive), (&params.tone), (&params.level)];

    for (idx, param) in columns.iter().enumerate() {
        let value_rect = rect_from_px(base, scale, VALUE_WINDOWS_PX[idx]);
        let slider_rect = rect_from_px(base, scale, SLIDER_SLOTS_PX[idx]);
        draw_column_at(ui, setter, param, idx, value_rect, slider_rect, scale);
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
    // 値表示矩形。
    value_rect: Rect,
    // スライダー矩形。
    slider_rect: Rect,
    // DPIスケール。
    scale: f32,
) {
    draw_value_window_at(ui, value_rect, param.value(), scale);

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
    let font_size = (rect.height() * 0.6).max(6.0);
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        format!("{value_text:>3}"),
        FontId::proportional(font_size),
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
        if let Some(pos) = response.interact_pointer_pos() {
            // マウス位置。
            // ドラッグ位置から新しい値を計算する。
            set_param_from_pos(setter, param, rect, pos);
        }
    }
    if response.drag_stopped() {
        setter.end_set_parameter(param);
    }
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            // マウス位置。
            // クリック位置から新しい値を計算する。
            setter.begin_set_parameter(param);
            set_param_from_pos(setter, param, rect, pos);
            setter.end_set_parameter(param);
        }
    }
}

fn set_param_from_pos(
    setter: &ParamSetter,
    param: &nih_plug::params::FloatParam,
    rect: Rect,
    pos: Pos2,
) {
    let new_value = ((rect.bottom() - pos.y) / rect.height()).clamp(0.0, 1.0);
    setter.set_parameter(param, new_value);
}

// 背景画像のピクセルサイズを返す。
fn bg_size_px() -> [usize; 2] {
    // 背景画像バイト列。
    const BG_BYTES: &[u8] = include_bytes!("../../assets/bg.png");
    let img = image::load_from_memory(BG_BYTES).expect("背景画像の読み込みに失敗しました");
    [img.width() as usize, img.height() as usize]
}

// 背景画像のテクスチャを読み込む。
fn load_bg_texture(
    // eguiのコンテキスト。
    ctx: &egui::Context,
) -> (egui::TextureHandle, [usize; 2]) {
    // 背景画像バイト列。
    const BG_BYTES: &[u8] = include_bytes!("../../assets/bg.png");
    // 画像をRGBAに変換する。
    let image = image::load_from_memory(BG_BYTES)
        .expect("背景画像の読み込みに失敗しました")
        .to_rgba8();

    // 背景画像のサイズ。
    let size = [image.width() as usize, image.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());

    // eguiにテクスチャとして登録する。
    (
        ctx.load_texture("bdd_plus_bg", color_image, egui::TextureOptions::LINEAR),
        size,
    )
}
