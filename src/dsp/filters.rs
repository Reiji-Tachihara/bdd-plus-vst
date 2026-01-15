use std::f32::consts::PI;

#[derive(Clone, Copy)]
pub(super) struct OnePole {
    z1: f32,
}

impl Default for OnePole {
    fn default() -> Self {
        Self { z1: 0.0 }
    }
}

impl OnePole {
    pub(super) fn reset(&mut self) {
        self.z1 = 0.0;
    }

    pub(super) fn lowpass(&mut self, input: f32, coeff: f32) -> f32 {
        self.z1 += coeff * (input - self.z1);
        self.z1
    }

    pub(super) fn highpass(&mut self, input: f32, coeff: f32) -> f32 {
        input - self.lowpass(input, coeff)
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct Biquad {
    z1: f32,
    z2: f32,
}

#[derive(Clone, Copy, Default)]
pub(super) struct BiquadCoeffs {
    pub(super) b0: f32,
    pub(super) b1: f32,
    pub(super) b2: f32,
    pub(super) a1: f32,
    pub(super) a2: f32,
}

impl Biquad {
    pub(super) fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    pub(super) fn process(&mut self, input: f32, c: BiquadCoeffs) -> f32 {
        let out = (c.b0 * input) + self.z1;
        self.z1 = (c.b1 * input) - (c.a1 * out) + self.z2;
        self.z2 = (c.b2 * input) - (c.a2 * out);
        out
    }
}

pub(super) fn peaking_eq_coeffs(
    freq_hz: f32,
    q: f32,
    gain_db: f32,
    sample_rate: f32,
) -> BiquadCoeffs {
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

pub(super) fn one_pole_coeff(cutoff: f32, sample_rate: f32) -> f32 {
    let x = (-2.0 * PI * cutoff / sample_rate).exp();
    (1.0 - x).clamp(0.0, 1.0)
}
