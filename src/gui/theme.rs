use nih_plug_egui::egui::{Color32, Vec2};

// パネル全体のサイズ。
pub(super) const PANEL_SIZE: Vec2 = Vec2::new(420.0, 300.0);
// パネル内側の余白。
pub(super) const PANEL_PADDING: f32 = 18.0;
// 各列の幅。
pub(super) const COLUMN_WIDTH: f32 = 110.0;
// スライダーの幅。
pub(super) const SLIDER_WIDTH: f32 = 36.0;
// スライダーの高さ。
pub(super) const SLIDER_HEIGHT: f32 = 165.0;
// 値表示の高さ。
pub(super) const VALUE_HEIGHT: f32 = 26.0;
// 値表示の角丸半径。
pub(super) const VALUE_RADIUS: f32 = 4.0;
// ラベル文字サイズ。
pub(super) const LABEL_SIZE: f32 = 13.0;
// バイパスボタンのサイズ。
pub(super) const BYPASS_SIZE: Vec2 = Vec2::new(140.0, 28.0);
// バイパスラベル文字サイズ。
pub(super) const BYPASS_LABEL_SIZE: f32 = 11.0;

// 値表示の文字色。
pub(super) const TEXT_VALUE: Color32 = Color32::from_rgb(70, 90, 92);
// ラベルの文字色。
pub(super) const TEXT_LABEL: Color32 = Color32::from_rgb(104, 124, 92);
// バイパスラベル文字色。
pub(super) const TEXT_BYPASS: Color32 = Color32::from_rgb(120, 120, 110);

// 値表示の背景色。
pub(super) const VALUE_BG: Color32 = Color32::from_rgb(10, 12, 12);
// 値表示の内側背景色。
pub(super) const VALUE_BG_INNER: Color32 = Color32::from_rgb(6, 8, 8);

// スライダートラック色。
pub(super) const SLIDER_TRACK: Color32 = Color32::from_rgb(25, 25, 25);
// スライダーチック色。
pub(super) const SLIDER_TICK: Color32 = Color32::from_rgb(90, 90, 90);
// スライダーノブ色。
pub(super) const SLIDER_KNOB: Color32 = Color32::from_rgb(110, 110, 108);
// スライダーノブ縁色。
pub(super) const SLIDER_KNOB_EDGE: Color32 = Color32::from_rgb(70, 70, 68);

// バイパスON色。
pub(super) const BYPASS_ON: Color32 = Color32::from_rgb(140, 40, 34);
// バイパスOFF色。
pub(super) const BYPASS_OFF: Color32 = Color32::from_rgb(40, 40, 38);
