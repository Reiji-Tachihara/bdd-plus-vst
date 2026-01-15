use nih_plug::prelude::*;

const GUI_STEP_SIZE: f32 = 1.0 / 23.0;

#[derive(Params)]
pub(crate) struct BddPlusParams {
    #[id = "drive"]
    pub drive: FloatParam,
    #[id = "tone"]
    pub tone: FloatParam,
    #[id = "level"]
    pub level: FloatParam,
    #[id = "bypass"]
    pub bypass: BoolParam,
}

impl Default for BddPlusParams {
    fn default() -> Self {
        Self {
            drive: FloatParam::new("Drive", 0.35, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_step_size(GUI_STEP_SIZE)
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            tone: FloatParam::new("Tone", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_step_size(GUI_STEP_SIZE)
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            level: FloatParam::new("Level", 0.8, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_step_size(GUI_STEP_SIZE)
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            bypass: BoolParam::new("Bypass", false)
                .make_bypass()
                .hide_in_generic_ui(),
        }
    }
}
