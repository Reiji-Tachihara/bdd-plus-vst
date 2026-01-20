mod constants;
mod filters;
mod waveshaper;

use constants::*;
use filters::{peaking_eq_coeffs, Biquad, BiquadCoeffs, OnePole};
use waveshaper::*;

#[derive(Clone, Copy)]
struct ChannelState {
    // 事前EQ用のバイカッド。
    pre_eq: Biquad,
    // 低域タイト化用の1次フィルタ。
    pre_tighten: OnePole,
    // 事前エンファシス用の1次フィルタ。
    pre_emphasis: OnePole,
    // 事前ソフト化用の1次フィルタ。
    pre_soften: OnePole,
    // 段間ローパス用の1次フィルタ。
    interstage: OnePole,
    // トーン用の1次フィルタ。
    tone: OnePole,
    // 事後ローパス用の1次フィルタ(段1)。
    post_clip_lpf: OnePole,
    // 事後ローパス用の1次フィルタ(段2)。
    post_clip_lpf2: OnePole,
    // ビンテージ質感用のローパス。
    vintage_lpf: OnePole,
    // DC除去用の1次フィルタ。
    dc_block: OnePole,
    // アンチエイリアス用の1次フィルタ(段1)。
    aa_stage1: OnePole,
    // アンチエイリアス用の1次フィルタ(段2)。
    aa_stage2: OnePole,
    // 前回入力サンプル(2x補間用)。
    prev_input: f32,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            // 各フィルタ状態を初期化する。
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
        // チャンネル内の全フィルタ状態をリセットする。
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
    // 現在のサンプルレート。
    sample_rate: f32,
    // チャンネルごとの状態バッファ。
    channels: Vec<ChannelState>,
    // 事前EQの係数。
    pre_eq_coeffs: BiquadCoeffs,
}

impl Default for Dsp {
    fn default() -> Self {
        Self {
            // 初期サンプルレート。
            sample_rate: 44100.0,
            // チャンネル数は初期化時に確定。
            channels: Vec::new(),
            // EQ係数の初期値。
            pre_eq_coeffs: BiquadCoeffs::default(),
        }
    }
}

impl Dsp {
    pub fn initialize(
        &mut self,
        // サンプルレート。
        sample_rate: f32,
        // チャンネル数。
        channels: usize,
    ) {
        // サンプルレートを安全域に制限する。
        self.sample_rate = sample_rate.max(1.0);
        // チャンネル数に応じて状態を確保する。
        self.channels.resize_with(channels, ChannelState::default);
        // 事前EQ係数を更新する。
        self.pre_eq_coeffs = peaking_eq_coeffs(
            PRE_EQ_FREQ_HZ,
            PRE_EQ_Q,
            PRE_EQ_GAIN_DB,
            self.sample_rate,
        );
        // 内部状態をリセットする。
        self.reset();
    }

    pub fn reset(&mut self) {
        // 全チャンネルの状態を初期化する。
        for state in &mut self.channels {
            state.reset();
        }
    }

    pub fn channel_count(&self) -> usize {
        // 現在のチャンネル数を返す。
        self.channels.len()
    }

    pub fn pre_emphasis_coeff(&self) -> f32 {
        // エンファシス係数を返す。
        pre_emphasis_coeff(self.sample_rate)
    }

    pub fn pre_tighten_coeff(&self) -> f32 {
        // 低域タイト化の係数を返す。
        pre_tighten_coeff(self.sample_rate)
    }

    pub fn dc_block_coeff(&self) -> f32 {
        // DCブロッカ係数を返す。
        dc_block_coeff(self.sample_rate)
    }

    pub fn process_sample(
        &mut self,
        // チャンネル番号。
        channel: usize,
        // 入力サンプル。
        input: f32,
        // ドライブ値。
        drive: f32,
        // トーン値。
        tone: f32,
        // レベル値。
        level: f32,
        // 事前エンファシス係数。
        pre_coeff: f32,
        // 低域タイト化係数。
        pre_tighten_coeff: f32,
        // DCブロック係数。
        dc_block_coeff: f32,
    ) -> f32 {
        // 対象チャンネルの状態を取得する。
        let state = &mut self.channels[channel];
        // ドライブに応じた第1段ゲイン。
        let drive_gain = drive_stage1_gain(drive) * drive_to_pre_boost(drive);
        // 出力レベルのゲイン。
        let level_gain = level_to_gain(level);
        // ドライブ補正のゲイン。
        let comp_gain = drive_to_compensation(drive);
        // デハーシュ量。
        let deharsh = deharsh_amount(drive);
        // トーン用係数。
        let tone_coeff = tone_to_coeff(tone, self.sample_rate, deharsh);
        // バイアス低減量。
        let bias_reduction = bias_reduction(drive);
        // 非対称バイアス量。
        let bias = ASYM_BIAS * drive * (1.0 - (ASYM_BIAS_REDUCTION * bias_reduction));
        // アンチエイリアス係数。
        let aa_coeff = aa_coeff(self.sample_rate);
        // 段間ローパス係数。
        let interstage_coeff = interstage_coeff(self.sample_rate);
        // 事前ソフト化係数。
        let pre_soften_coeff = pre_soften_coeff(self.sample_rate);
        // 事後ローパス係数。
        let post_lpf_coeff = post_lpf_coeff(self.sample_rate, deharsh);
        // 事後ローパス混合量。
        let post_lpf_mix = post_lpf_mix(deharsh);
        // ビンテージローパス係数。
        let vintage_coeff = vintage_lpf_coeff(self.sample_rate, drive);
        // 第2段ゲイン。
        let stage2_gain = drive_stage2_gain(drive);
        // 第2段ブレンド比率。
        let stage2_mix = drive_stage2_mix(drive);

        // クリップ前に低域を締める。
        let eq = state.pre_eq.process(input + ANTI_DENORMAL, self.pre_eq_coeffs);
        // 低域タイト化後の信号。
        let tightened = state.pre_tighten.highpass(eq, pre_tighten_coeff);
        // エンファシス後の信号。
        let pre = state.pre_emphasis.highpass(tightened, pre_coeff);
        // 事前ソフト化後の信号。
        let softened = state.pre_soften.lowpass(pre, pre_soften_coeff);
        // ソフト化を混ぜた事前信号。
        let pre = pre + (softened - pre) * PRE_SOFTEN_MIX;

        // 2xオーバーサンプル用の線形補間値。
        let up_a = (state.prev_input + pre) * 0.5;
        // 前回入力を更新する。
        state.prev_input = pre;
        // 補間サンプルのクリップ結果。
        let y_a = soft_clip((up_a * drive_gain) + bias);
        // 現在サンプルのクリップ結果。
        let y_b = soft_clip((pre * drive_gain) + bias);
        // 段間ローパス後の補間サンプル。
        let inter_a = state.interstage.lowpass(y_a, interstage_coeff);
        // 段間ローパス後の現サンプル。
        let inter_b = state.interstage.lowpass(y_b, interstage_coeff);
        // 第2段クリップ(補間サンプル)。
        let stage2_a = soft_clip(inter_a * stage2_gain);
        // 第2段クリップ(現サンプル)。
        let stage2_b = soft_clip(inter_b * stage2_gain);
        // 第2段ブレンド(補間サンプル)。
        let blend_a = inter_a + (stage2_a - inter_a) * stage2_mix;
        // 第2段ブレンド(現サンプル)。
        let blend_b = inter_b + (stage2_b - inter_b) * stage2_mix;
        // アンチエイリアス1段目(補間サンプル)。
        let aa_a = state.aa_stage1.lowpass(blend_a, aa_coeff);
        // アンチエイリアス2段目(補間サンプル)。
        let aa_a = state.aa_stage2.lowpass(aa_a, aa_coeff);
        // アンチエイリアス1段目(現サンプル)。
        let aa_b = state.aa_stage1.lowpass(blend_b, aa_coeff);
        // アンチエイリアス2段目(現サンプル)。
        let aa_b = state.aa_stage2.lowpass(aa_b, aa_coeff);
        // 2xを戻すため平均する。
        let clipped = (aa_a + aa_b) * 0.5;

        // 非対称で生じたDCを除去する。
        let dc_free = state.dc_block.highpass(clipped, dc_block_coeff);

        // クリップ後の高域を抑える。
        let post_lpf = state.post_clip_lpf.lowpass(dc_free, post_lpf_coeff);
        // 2段目のローパス。
        let post_lpf = state.post_clip_lpf2.lowpass(post_lpf, post_lpf_coeff);
        // ローパスを混ぜて自然さを残す。
        let post_lpf = dc_free + (post_lpf - dc_free) * post_lpf_mix;
        // ビンテージ感のローパス。
        let vintage = state.vintage_lpf.lowpass(post_lpf, vintage_coeff);
        // トーン用ローパス。
        let toned = state.tone.lowpass(vintage, tone_coeff);

        // 補正とレベルを適用して出力する。
        toned * comp_gain * level_gain
    }
}
