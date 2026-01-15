use nih_plug::prelude::*;
use std::sync::Arc;

mod dsp {
    use nih_plug::prelude::util;
    use std::f32::consts::PI;

    const PRE_HP_HZ: f32 = 100.0;
    const PRE_EMPH_HZ: f32 = 600.0;
    const PRE_EQ_FREQ_HZ: f32 = 120.0;
    const PRE_EQ_Q: f32 = 0.6;
    const PRE_EQ_GAIN_DB: f32 = -1.0;
    const TONE_MIN_HZ: f32 = 700.0;
    const TONE_MAX_HZ: f32 = 5200.0;
    const DC_BLOCK_HZ: f32 = 18.0;
    const ASYM_BIAS: f32 = 0.08;
    const DRIVE_KNEE: f32 = 0.6;
    const DRIVE_STAGE1_DB: f32 = 30.0;
    const DRIVE_PRE_BOOST_DB: f32 = 6.0;
    const SECOND_STAGE_MIN_GAIN: f32 = 1.0;
    const SECOND_STAGE_MAX_GAIN: f32 = 2.0;
    const SECOND_STAGE_MIX_MAX: f32 = 0.75;
    const INTERSTAGE_LP_HZ: f32 = 7500.0;
    const POST_LPF_BASE_HZ: f32 = 11200.0;
    const POST_LPF_REDUCTION_HZ: f32 = 3800.0;
    const POST_LPF_MIX_MAX: f32 = 0.6;
    const VINTAGE_LPF_BASE_HZ: f32 = 9800.0;
    const VINTAGE_LPF_REDUCTION_HZ: f32 = 1800.0;
    const AA_CUTOFF_RATIO: f32 = 0.45;
    const PRE_SOFT_LP_HZ: f32 = 16000.0;
    const PRE_SOFTEN_MIX: f32 = 0.08;
    const DEHARSH_START: f32 = 0.5;
    const DEHARSH_CUTOFF_SCALE: f32 = 0.12;
    const ASYM_BIAS_REDUCTION: f32 = 0.65;
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

    #[derive(Clone, Copy, Default)]
    struct Biquad {
        z1: f32,
        z2: f32,
    }

    #[derive(Clone, Copy, Default)]
    struct BiquadCoeffs {
        b0: f32,
        b1: f32,
        b2: f32,
        a1: f32,
        a2: f32,
    }

    impl Biquad {
        fn reset(&mut self) {
            self.z1 = 0.0;
            self.z2 = 0.0;
        }

        fn process(&mut self, input: f32, c: BiquadCoeffs) -> f32 {
            let out = (c.b0 * input) + self.z1;
            self.z1 = (c.b1 * input) - (c.a1 * out) + self.z2;
            self.z2 = (c.b2 * input) - (c.a2 * out);
            out
        }
    }

    #[derive(Clone, Copy)]
    struct ChannelState {
        pre_eq: Biquad,
        pre_tighten: OnePole,
        pre_emphasis: OnePole,
        pre_soften: OnePole,
        interstage: OnePole,
        tone: OnePole,
        post_clip_lpf: OnePole,
        post_clip_lpf2: OnePole,
        vintage_lpf: OnePole,
        dc_block: OnePole,
        aa_stage1: OnePole,
        aa_stage2: OnePole,
        prev_input: f32,
    }

    impl Default for ChannelState {
        fn default() -> Self {
            Self {
                pre_eq: Biquad::default(),
                pre_tighten: OnePole::default(),
                pre_emphasis: OnePole::default(),
                pre_soften: OnePole::default(),
                interstage: OnePole::default(),
                tone: OnePole::default(),
                post_clip_lpf: OnePole::default(),
                post_clip_lpf2: OnePole::default(),
                vintage_lpf: OnePole::default(),
                dc_block: OnePole::default(),
                aa_stage1: OnePole::default(),
                aa_stage2: OnePole::default(),
                prev_input: 0.0,
            }
        }
    }

    impl ChannelState {
        fn reset(&mut self) {
            self.pre_eq.reset();
            self.pre_tighten.reset();
            self.pre_emphasis.reset();
            self.pre_soften.reset();
            self.interstage.reset();
            self.tone.reset();
            self.post_clip_lpf.reset();
            self.post_clip_lpf2.reset();
            self.vintage_lpf.reset();
            self.dc_block.reset();
            self.aa_stage1.reset();
            self.aa_stage2.reset();
            self.prev_input = 0.0;
        }
    }

    pub struct Dsp {
        sample_rate: f32,
        channels: Vec<ChannelState>,
        pre_eq_coeffs: BiquadCoeffs,
    }

    impl Default for Dsp {
        fn default() -> Self {
            Self {
                sample_rate: 44100.0,
                channels: Vec::new(),
                pre_eq_coeffs: BiquadCoeffs::default(),
            }
        }
    }

    impl Dsp {
        pub fn initialize(&mut self, sample_rate: f32, channels: usize) {
            self.sample_rate = sample_rate.max(1.0);
            self.channels.resize_with(channels, ChannelState::default);
            self.pre_eq_coeffs = peaking_eq_coeffs(
                PRE_EQ_FREQ_HZ,
                PRE_EQ_Q,
                PRE_EQ_GAIN_DB,
                self.sample_rate,
            );
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

        pub fn pre_tighten_coeff(&self) -> f32 {
            pre_tighten_coeff(self.sample_rate)
        }

        pub fn dc_block_coeff(&self) -> f32 {
            dc_block_coeff(self.sample_rate)
        }

        pub fn process_sample(
            &mut self,
            channel: usize,
            input: f32,
            drive: f32,
            tone: f32,
            level: f32,
            pre_coeff: f32,
            pre_tighten_coeff: f32,
            dc_block_coeff: f32,
        ) -> f32 {
            let state = &mut self.channels[channel];
            let drive_gain = drive_stage1_gain(drive) * drive_to_pre_boost(drive);
            let level_gain = level_to_gain(level);
            let comp_gain = drive_to_compensation(drive);
            let deharsh = deharsh_amount(drive);
            let tone_coeff = tone_to_coeff(tone, self.sample_rate, deharsh);
            let bias_reduction = bias_reduction(drive);
            let bias = ASYM_BIAS * drive * (1.0 - (ASYM_BIAS_REDUCTION * bias_reduction));
            let aa_coeff = aa_coeff(self.sample_rate);
            let interstage_coeff = interstage_coeff(self.sample_rate);
            let pre_soften_coeff = pre_soften_coeff(self.sample_rate);
            let post_lpf_coeff = post_lpf_coeff(self.sample_rate, deharsh);
            let post_lpf_mix = post_lpf_mix(deharsh);
            let vintage_coeff = vintage_lpf_coeff(self.sample_rate, drive);
            let stage2_gain = drive_stage2_gain(drive);
            let stage2_mix = drive_stage2_mix(drive);

            // Pre-filtering tightens lows before clipping.
            let eq = state.pre_eq.process(input + ANTI_DENORMAL, self.pre_eq_coeffs);
            let tightened = state
                .pre_tighten
                .highpass(eq, pre_tighten_coeff);
            let pre = state.pre_emphasis.highpass(tightened, pre_coeff);
            let softened = state.pre_soften.lowpass(pre, pre_soften_coeff);
            let pre = pre + (softened - pre) * PRE_SOFTEN_MIX;

            // 2x oversampling with linear interpolation and averaging on the way down.
            let up_a = (state.prev_input + pre) * 0.5;
            state.prev_input = pre;
            let y_a = soft_clip((up_a * drive_gain) + bias);
            let y_b = soft_clip((pre * drive_gain) + bias);
            let inter_a = state.interstage.lowpass(y_a, interstage_coeff);
            let inter_b = state.interstage.lowpass(y_b, interstage_coeff);
            let stage2_a = soft_clip(inter_a * stage2_gain);
            let stage2_b = soft_clip(inter_b * stage2_gain);
            let blend_a = inter_a + (stage2_a - inter_a) * stage2_mix;
            let blend_b = inter_b + (stage2_b - inter_b) * stage2_mix;
            let aa_a = state.aa_stage1.lowpass(blend_a, aa_coeff);
            let aa_a = state.aa_stage2.lowpass(aa_a, aa_coeff);
            let aa_b = state.aa_stage1.lowpass(blend_b, aa_coeff);
            let aa_b = state.aa_stage2.lowpass(aa_b, aa_coeff);
            let clipped = (aa_a + aa_b) * 0.5;

            // Remove DC offset introduced by asymmetry.
            let dc_free = state.dc_block.highpass(clipped, dc_block_coeff);

            // Post tone filter tames the top-end after clipping.
            let post_lpf = state.post_clip_lpf.lowpass(dc_free, post_lpf_coeff);
            let post_lpf = state.post_clip_lpf2.lowpass(post_lpf, post_lpf_coeff);
            let post_lpf = dc_free + (post_lpf - dc_free) * post_lpf_mix;
            let vintage = state.vintage_lpf.lowpass(post_lpf, vintage_coeff);
            let toned = state.tone.lowpass(vintage, tone_coeff);

            toned * comp_gain * level_gain
        }
    }

    fn soft_clip(input: f32) -> f32 {
        input.tanh()
    }

    fn drive_stage1_gain(drive: f32) -> f32 {
        let shaped = drive_curve(drive, 2.2);
        util::db_to_gain(shaped * DRIVE_STAGE1_DB)
    }

    fn drive_to_pre_boost(drive: f32) -> f32 {
        let shaped = drive.clamp(0.0, 1.0);
        util::db_to_gain(DRIVE_PRE_BOOST_DB * shaped)
    }

    fn drive_stage2_gain(drive: f32) -> f32 {
        let mid = drive_mid_focus(drive);
        SECOND_STAGE_MIN_GAIN + (SECOND_STAGE_MAX_GAIN - SECOND_STAGE_MIN_GAIN) * mid
    }

    fn drive_stage2_mix(drive: f32) -> f32 {
        let mid = drive_mid_focus(drive);
        SECOND_STAGE_MIX_MAX * mid
    }

    fn drive_to_compensation(drive: f32) -> f32 {
        let shaped = drive_curve(drive, 1.4);
        util::db_to_gain(-5.0 * shaped)
    }

    fn level_to_gain(level: f32) -> f32 {
        let db = -60.0 + (level.clamp(0.0, 1.0) * 66.0);
        util::db_to_gain(db)
    }

    fn tone_to_coeff(tone: f32, sample_rate: f32, deharsh: f32) -> f32 {
        let shaped = tone.clamp(0.0, 1.0).powf(0.7);
        let cutoff = TONE_MIN_HZ + (TONE_MAX_HZ - TONE_MIN_HZ) * shaped;
        let adjusted = cutoff * (1.0 - (DEHARSH_CUTOFF_SCALE * deharsh));
        one_pole_coeff(adjusted, sample_rate)
    }

    fn pre_emphasis_coeff(sample_rate: f32) -> f32 {
        one_pole_coeff(PRE_EMPH_HZ, sample_rate)
    }

    fn pre_tighten_coeff(sample_rate: f32) -> f32 {
        one_pole_coeff(PRE_HP_HZ, sample_rate)
    }

    fn dc_block_coeff(sample_rate: f32) -> f32 {
        one_pole_coeff(DC_BLOCK_HZ, sample_rate)
    }

    fn interstage_coeff(sample_rate: f32) -> f32 {
        one_pole_coeff(INTERSTAGE_LP_HZ, sample_rate * 2.0)
    }

    fn post_lpf_coeff(sample_rate: f32, deharsh: f32) -> f32 {
        let cutoff = (POST_LPF_BASE_HZ - (POST_LPF_REDUCTION_HZ * deharsh)).max(40.0);
        one_pole_coeff(cutoff, sample_rate)
    }

    fn post_lpf_mix(deharsh: f32) -> f32 {
        POST_LPF_MIX_MAX * deharsh
    }

    fn aa_coeff(sample_rate: f32) -> f32 {
        one_pole_coeff(sample_rate * AA_CUTOFF_RATIO, sample_rate * 2.0)
    }

    fn pre_soften_coeff(sample_rate: f32) -> f32 {
        one_pole_coeff(PRE_SOFT_LP_HZ, sample_rate)
    }

    fn vintage_lpf_coeff(sample_rate: f32, drive: f32) -> f32 {
        let shaped = drive_curve(drive, 1.2);
        let cutoff = VINTAGE_LPF_BASE_HZ - (VINTAGE_LPF_REDUCTION_HZ * shaped);
        one_pole_coeff(cutoff.max(40.0), sample_rate)
    }

    fn bias_reduction(drive: f32) -> f32 {
        smoothstep(DEHARSH_START, 1.0, drive)
    }

    fn deharsh_amount(drive: f32) -> f32 {
        ((drive - DEHARSH_START) / (1.0 - DEHARSH_START)).clamp(0.0, 1.0)
    }

    fn drive_mid_focus(drive: f32) -> f32 {
        smoothstep(0.3, 0.7, drive)
    }

    fn drive_curve(drive: f32, power: f32) -> f32 {
        let shaped = (drive / DRIVE_KNEE).clamp(0.0, 1.0);
        shaped.powf(power)
    }

    fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    fn peaking_eq_coeffs(freq_hz: f32, q: f32, gain_db: f32, sample_rate: f32) -> BiquadCoeffs {
        let omega = 2.0 * PI * (freq_hz / sample_rate);
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q.max(0.001));
        let a = 10.0_f32.powf(gain_db / 40.0);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / a;

        let inv_a0 = 1.0 / a0;
        BiquadCoeffs {
            b0: b0 * inv_a0,
            b1: b1 * inv_a0,
            b2: b2 * inv_a0,
            a1: a1 * inv_a0,
            a2: a2 * inv_a0,
        }
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
    #[id = "bypass"]
    pub bypass: BoolParam,
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
            bypass: BoolParam::new("Bypass", false)
                .make_bypass()
                .hide_in_generic_ui(),
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
        if self.params.bypass.value() {
            return ProcessStatus::Normal;
        }

        if buffer.is_empty() || buffer.channels() == 0 {
            return ProcessStatus::Normal;
        }

        if buffer.channels() > self.dsp.channel_count() {
            return ProcessStatus::Normal;
        }

        let pre_coeff = self.dsp.pre_emphasis_coeff();
        let pre_tighten_coeff = self.dsp.pre_tighten_coeff();
        let dc_block_coeff = self.dsp.dc_block_coeff();

        for mut channels in buffer.iter_samples() {
            let drive = self.params.drive.smoothed.next();
            let tone = self.params.tone.smoothed.next();
            let level = self.params.level.smoothed.next();

            for channel_idx in 0..channels.len() {
                // SAFETY: channel_idx is within channels.len().
                let sample = unsafe { channels.get_unchecked_mut(channel_idx) };
                let processed = self.dsp.process_sample(
                    channel_idx,
                    *sample,
                    drive,
                    tone,
                    level,
                    pre_coeff,
                    pre_tighten_coeff,
                    dc_block_coeff,
                );
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
