use nih_plug::prelude::*;
use std::sync::Arc;

mod dsp {
    use nih_plug::prelude::util;
    use std::f32::consts::PI;

    const PRE_EMPH_HZ: f32 = 140.0;
    const TONE_MIN_HZ: f32 = 800.0;
    const TONE_MAX_HZ: f32 = 6500.0;
    const ASYM_BIAS: f32 = 0.08;
    const ANTI_DENORMAL: f32 = 1.0e-24;

    #[derive(Clone, Copy)]
    struct OnePole {
        z1: f32,
    }

    impl Default for OnePole {
        fn default() -> Self {
            Self { z1: 0.0 }
        }
    }

    impl OnePole {
        fn reset(&mut self) {
            self.z1 = 0.0;
        }

        fn lowpass(&mut self, input: f32, coeff: f32) -> f32 {
            self.z1 += coeff * (input - self.z1);
            self.z1
        }

        fn highpass(&mut self, input: f32, coeff: f32) -> f32 {
            input - self.lowpass(input, coeff)
        }
    }

    #[derive(Clone, Copy)]
    struct ChannelState {
        pre_emphasis: OnePole,
        tone: OnePole,
        prev_input: f32,
    }

    impl Default for ChannelState {
        fn default() -> Self {
            Self {
                pre_emphasis: OnePole::default(),
                tone: OnePole::default(),
                prev_input: 0.0,
            }
        }
    }

    impl ChannelState {
        fn reset(&mut self) {
            self.pre_emphasis.reset();
            self.tone.reset();
            self.prev_input = 0.0;
        }
    }

    pub struct Dsp {
        sample_rate: f32,
        channels: Vec<ChannelState>,
    }

    impl Default for Dsp {
        fn default() -> Self {
            Self {
                sample_rate: 44100.0,
                channels: Vec::new(),
            }
        }
    }

    impl Dsp {
        pub fn initialize(&mut self, sample_rate: f32, channels: usize) {
            self.sample_rate = sample_rate.max(1.0);
            self.channels.resize_with(channels, ChannelState::default);
            self.reset();
        }

        pub fn reset(&mut self) {
            for state in &mut self.channels {
                state.reset();
            }
        }

        pub fn channel_count(&self) -> usize {
            self.channels.len()
        }

        pub fn pre_emphasis_coeff(&self) -> f32 {
            pre_emphasis_coeff(self.sample_rate)
        }

        pub fn process_sample(
            &mut self,
            channel: usize,
            input: f32,
            drive: f32,
            tone: f32,
            level: f32,
            pre_coeff: f32,
        ) -> f32 {
            let state = &mut self.channels[channel];
            let drive_gain = drive_to_gain(drive);
            let level_gain = level_to_gain(level);
            let tone_coeff = tone_to_coeff(tone, self.sample_rate);

            // Pre-emphasis trims lows to keep the clipping tight.
            let pre = state
                .pre_emphasis
                .highpass(input + ANTI_DENORMAL, pre_coeff);

            // 2x oversampling with linear interpolation and averaging on the way down.
            let up_a = (state.prev_input + pre) * 0.5;
            state.prev_input = pre;
            let y_a = soft_clip(up_a * drive_gain);
            let y_b = soft_clip(pre * drive_gain);
            let clipped = (y_a + y_b) * 0.5;

            // Post tone filter tames the top-end after clipping.
            let toned = state.tone.lowpass(clipped, tone_coeff);

            toned * level_gain
        }
    }

    fn soft_clip(input: f32) -> f32 {
        let biased = input + ASYM_BIAS;
        biased.tanh() - ASYM_BIAS.tanh()
    }

    fn drive_to_gain(drive: f32) -> f32 {
        let shaped = drive.clamp(0.0, 1.0).powf(2.2);
        util::db_to_gain(shaped * 24.0)
    }

    fn level_to_gain(level: f32) -> f32 {
        let db = -60.0 + (level.clamp(0.0, 1.0) * 66.0);
        util::db_to_gain(db)
    }

    fn tone_to_coeff(tone: f32, sample_rate: f32) -> f32 {
        let shaped = tone.clamp(0.0, 1.0).powf(0.7);
        let cutoff = TONE_MIN_HZ + (TONE_MAX_HZ - TONE_MIN_HZ) * shaped;
        one_pole_coeff(cutoff, sample_rate)
    }

    fn pre_emphasis_coeff(sample_rate: f32) -> f32 {
        one_pole_coeff(PRE_EMPH_HZ, sample_rate)
    }

    fn one_pole_coeff(cutoff: f32, sample_rate: f32) -> f32 {
        let x = (-2.0 * PI * cutoff / sample_rate).exp();
        (1.0 - x).clamp(0.0, 1.0)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn soft_clip_is_bounded() {
            for value in [-10.0, -1.0, 0.0, 1.0, 10.0] {
                let shaped = soft_clip(value);
                assert!(shaped.is_finite());
                assert!(shaped.abs() <= 1.2);
            }
        }
    }
}

struct BddPlus {
    params: Arc<BddPlusParams>,
    dsp: dsp::Dsp,
}

const GUI_STEP_SIZE: f32 = 1.0 / 23.0;

#[derive(Params)]
struct BddPlusParams {
    #[id = "drive"]
    pub drive: FloatParam,
    #[id = "tone"]
    pub tone: FloatParam,
    #[id = "level"]
    pub level: FloatParam,
}

impl Default for BddPlus {
    fn default() -> Self {
        Self {
            params: Arc::new(BddPlusParams::default()),
            dsp: dsp::Dsp::default(),
        }
    }
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
        }
    }
}

impl Plugin for BddPlus {
    const NAME: &'static str = "bdd_plus";
    const VENDOR: &'static str = "Reiji Tachihara";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "re.sg.102041@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let channels = audio_io_layout
            .main_output_channels
            .map(|count| count.get() as usize)
            .unwrap_or(0);
        self.dsp.initialize(buffer_config.sample_rate, channels);
        true
    }

    fn reset(&mut self) {
        self.dsp.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if buffer.is_empty() || buffer.channels() == 0 {
            return ProcessStatus::Normal;
        }

        if buffer.channels() > self.dsp.channel_count() {
            return ProcessStatus::Normal;
        }

        let pre_coeff = self.dsp.pre_emphasis_coeff();

        for mut channels in buffer.iter_samples() {
            let drive = self.params.drive.smoothed.next();
            let tone = self.params.tone.smoothed.next();
            let level = self.params.level.smoothed.next();

            for channel_idx in 0..channels.len() {
                // SAFETY: channel_idx is within channels.len().
                let sample = unsafe { channels.get_unchecked_mut(channel_idx) };
                let processed =
                    self.dsp
                        .process_sample(channel_idx, *sample, drive, tone, level, pre_coeff);
                *sample = processed;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for BddPlus {
    const CLAP_ID: &'static str = "jp.reiji.bddplus";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("My favorite effecter");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for BddPlus {
    const VST3_CLASS_ID: [u8; 16] = *b"JPREIJIBDDPLUS01";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nih_export_clap!(BddPlus);
nih_export_vst3!(BddPlus);
