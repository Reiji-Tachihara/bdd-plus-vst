use nih_plug_egui::egui::{Color32, Vec2};

pub(super) const PANEL_SIZE: Vec2 = Vec2::new(420.0, 300.0);
pub(super) const PANEL_PADDING: f32 = 18.0;
pub(super) const COLUMN_WIDTH: f32 = 110.0;
pub(super) const SLIDER_WIDTH: f32 = 36.0;
pub(super) const SLIDER_HEIGHT: f32 = 165.0;
pub(super) const VALUE_HEIGHT: f32 = 26.0;
pub(super) const VALUE_RADIUS: f32 = 4.0;
pub(super) const LABEL_SIZE: f32 = 13.0;
pub(super) const BYPASS_SIZE: Vec2 = Vec2::new(140.0, 28.0);
pub(super) const BYPASS_LABEL_SIZE: f32 = 11.0;

pub(super) const TEXT_VALUE: Color32 = Color32::from_rgb(70, 90, 92);
pub(super) const TEXT_LABEL: Color32 = Color32::from_rgb(104, 124, 92);
pub(super) const TEXT_BYPASS: Color32 = Color32::from_rgb(120, 120, 110);

pub(super) const VALUE_BG: Color32 = Color32::from_rgb(10, 12, 12);
pub(super) const VALUE_BG_INNER: Color32 = Color32::from_rgb(6, 8, 8);

pub(super) const SLIDER_TRACK: Color32 = Color32::from_rgb(25, 25, 25);
pub(super) const SLIDER_TICK: Color32 = Color32::from_rgb(90, 90, 90);
pub(super) const SLIDER_KNOB: Color32 = Color32::from_rgb(110, 110, 108);
pub(super) const SLIDER_KNOB_EDGE: Color32 = Color32::from_rgb(70, 70, 68);

pub(super) const BYPASS_ON: Color32 = Color32::from_rgb(140, 40, 34);
pub(super) const BYPASS_OFF: Color32 = Color32::from_rgb(40, 40, 38);
