use std::f32::consts::PI;

#[derive(Clone, Copy)]
pub(super) struct OnePole {
    // 1サンプル遅延の内部状態。
    z1: f32,
}

impl Default for OnePole {
    fn default() -> Self {
        // 内部状態をゼロ初期化する。
        Self { z1: 0.0 }
    }
}

impl OnePole {
    pub(super) fn reset(&mut self) {
        // フィルタ状態を初期化する。
        self.z1 = 0.0;
    }

    pub(super) fn lowpass(
        &mut self,
        // 入力サンプル。
        input: f32,
        // 1次フィルタ係数。
        coeff: f32,
    ) -> f32 {
        // 入力サンプル。
        self.z1 += coeff * (input - self.z1);
        // ローパス出力。
        self.z1
    }

    pub(super) fn highpass(
        &mut self,
        // 入力サンプル。
        input: f32,
        // 1次フィルタ係数。
        coeff: f32,
    ) -> f32 {
        // ローパスとの差分でハイパスを得る。
        input - self.lowpass(input, coeff)
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct Biquad {
    // 双一次変換の遅延状態1。
    z1: f32,
    // 双一次変換の遅延状態2。
    z2: f32,
}

#[derive(Clone, Copy, Default)]
pub(super) struct BiquadCoeffs {
    // フィードフォワード係数0。
    pub(super) b0: f32,
    // フィードフォワード係数1。
    pub(super) b1: f32,
    // フィードフォワード係数2。
    pub(super) b2: f32,
    // フィードバック係数1。
    pub(super) a1: f32,
    // フィードバック係数2。
    pub(super) a2: f32,
}

impl Biquad {
    pub(super) fn reset(&mut self) {
        // 双一次の遅延状態をクリアする。
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    pub(super) fn process(
        &mut self,
        // 入力サンプル。
        input: f32,
        // バイカッド係数。
        c: BiquadCoeffs,
    ) -> f32 {
        // 現在の出力サンプル。
        let out = (c.b0 * input) + self.z1;
        // 1つ目の遅延状態を更新する。
        self.z1 = (c.b1 * input) - (c.a1 * out) + self.z2;
        // 2つ目の遅延状態を更新する。
        self.z2 = (c.b2 * input) - (c.a2 * out);
        // フィルタ出力。
        out
    }
}

pub(super) fn peaking_eq_coeffs(
    // 中心周波数。
    freq_hz: f32,
    // Q値。
    q: f32,
    // ゲイン(dB)。
    gain_db: f32,
    // サンプルレート。
    sample_rate: f32,
) -> BiquadCoeffs {
    // 正規化角周波数。
    let omega = 2.0 * PI * (freq_hz / sample_rate);
    // サイン成分。
    let sin_omega = omega.sin();
    // コサイン成分。
    let cos_omega = omega.cos();
    // Q値から導く帯域幅係数。
    let alpha = sin_omega / (2.0 * q.max(0.001));
    // ゲインをリニアに変換。
    let a = 10.0_f32.powf(gain_db / 40.0);

    // フィードフォワード係数0。
    let b0 = 1.0 + alpha * a;
    // フィードフォワード係数1。
    let b1 = -2.0 * cos_omega;
    // フィードフォワード係数2。
    let b2 = 1.0 - alpha * a;
    // 正規化用のA0係数。
    let a0 = 1.0 + alpha / a;
    // フィードバック係数1。
    let a1 = -2.0 * cos_omega;
    // フィードバック係数2。
    let a2 = 1.0 - alpha / a;

    // A0で正規化するための逆数。
    let inv_a0 = 1.0 / a0;
    BiquadCoeffs {
        // 正規化済みb0。
        b0: b0 * inv_a0,
        // 正規化済みb1。
        b1: b1 * inv_a0,
        // 正規化済みb2。
        b2: b2 * inv_a0,
        // 正規化済みa1。
        a1: a1 * inv_a0,
        // 正規化済みa2。
        a2: a2 * inv_a0,
    }
}

pub(super) fn one_pole_coeff(
    // カットオフ周波数。
    cutoff: f32,
    // サンプルレート。
    sample_rate: f32,
) -> f32 {
    // 1次IIRの減衰係数。
    let x = (-2.0 * PI * cutoff / sample_rate).exp();
    // 0..1に収めた係数。
    (1.0 - x).clamp(0.0, 1.0)
}
