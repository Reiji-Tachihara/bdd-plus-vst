use nih_plug::prelude::*;

// GUI用のステップ幅(24段階相当)。
const GUI_STEP_SIZE: f32 = 1.0 / 23.0;

#[derive(Params)]
pub(crate) struct BddPlusParams {
    #[id = "drive"]
    // ドライブ量。
    pub drive: FloatParam,
    #[id = "tone"]
    // トーン量。
    pub tone: FloatParam,
    #[id = "level"]
    // 出力レベル。
    pub level: FloatParam,
    #[id = "bypass"]
    // バイパスの有効/無効。
    pub bypass: BoolParam,
}

fn make_param(name: &str, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max: 1.0 })
        .with_step_size(GUI_STEP_SIZE)
        .with_smoother(SmoothingStyle::Linear(50.0))
        .with_unit(" %")
        .with_value_to_string(formatters::v2s_f32_percentage(0))
        .with_string_to_value(formatters::s2v_f32_percentage())
}

impl Default for BddPlusParams {
    fn default() -> Self {
        Self {
            // ドライブパラメータ。
            drive: make_param("Drive", 0.5),
            // トーンパラメータ。
            tone: make_param("Tone", 0.5),
            // レベルパラメータ。
            level: make_param("Level", 0.5),
            // バイパスパラメータ。
            bypass: BoolParam::new("Bypass", false)
                .make_bypass()
                .hide_in_generic_ui(),
        }
    }
}
