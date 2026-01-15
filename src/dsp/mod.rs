mod constants;
mod filters;
mod waveshaper;

use constants::*;
use filters::{peaking_eq_coeffs, Biquad, BiquadCoeffs, OnePole};
use waveshaper::*;

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
        let tightened = state.pre_tighten.highpass(eq, pre_tighten_coeff);
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
