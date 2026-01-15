use nih_plug::prelude::util;

use super::constants::*;
use super::filters::one_pole_coeff;

pub(super) fn soft_clip(input: f32) -> f32 {
    input.tanh()
}

pub(super) fn drive_stage1_gain(drive: f32) -> f32 {
    let shaped = drive_curve(drive, 2.2);
    util::db_to_gain(shaped * DRIVE_STAGE1_DB)
}

pub(super) fn drive_to_pre_boost(drive: f32) -> f32 {
    let shaped = drive.clamp(0.0, 1.0);
    util::db_to_gain(DRIVE_PRE_BOOST_DB * shaped)
}

pub(super) fn drive_stage2_gain(drive: f32) -> f32 {
    let mid = drive_mid_focus(drive);
    SECOND_STAGE_MIN_GAIN + (SECOND_STAGE_MAX_GAIN - SECOND_STAGE_MIN_GAIN) * mid
}

pub(super) fn drive_stage2_mix(drive: f32) -> f32 {
    let mid = drive_mid_focus(drive);
    SECOND_STAGE_MIX_MAX * mid
}

pub(super) fn drive_to_compensation(drive: f32) -> f32 {
    let shaped = drive_curve(drive, 1.4);
    util::db_to_gain(-5.0 * shaped)
}

pub(super) fn level_to_gain(level: f32) -> f32 {
    let db = -60.0 + (level.clamp(0.0, 1.0) * 66.0);
    util::db_to_gain(db)
}

pub(super) fn tone_to_coeff(tone: f32, sample_rate: f32, deharsh: f32) -> f32 {
    let shaped = tone.clamp(0.0, 1.0).powf(0.7);
    let cutoff = TONE_MIN_HZ + (TONE_MAX_HZ - TONE_MIN_HZ) * shaped;
    let adjusted = cutoff * (1.0 - (DEHARSH_CUTOFF_SCALE * deharsh));
    one_pole_coeff(adjusted, sample_rate)
}

pub(super) fn pre_emphasis_coeff(sample_rate: f32) -> f32 {
    one_pole_coeff(PRE_EMPH_HZ, sample_rate)
}

pub(super) fn pre_tighten_coeff(sample_rate: f32) -> f32 {
    one_pole_coeff(PRE_HP_HZ, sample_rate)
}

pub(super) fn dc_block_coeff(sample_rate: f32) -> f32 {
    one_pole_coeff(DC_BLOCK_HZ, sample_rate)
}

pub(super) fn interstage_coeff(sample_rate: f32) -> f32 {
    one_pole_coeff(INTERSTAGE_LP_HZ, sample_rate * 2.0)
}

pub(super) fn post_lpf_coeff(sample_rate: f32, deharsh: f32) -> f32 {
    let cutoff = (POST_LPF_BASE_HZ - (POST_LPF_REDUCTION_HZ * deharsh)).max(40.0);
    one_pole_coeff(cutoff, sample_rate)
}

pub(super) fn post_lpf_mix(deharsh: f32) -> f32 {
    POST_LPF_MIX_MAX * deharsh
}

pub(super) fn aa_coeff(sample_rate: f32) -> f32 {
    one_pole_coeff(sample_rate * AA_CUTOFF_RATIO, sample_rate * 2.0)
}

pub(super) fn pre_soften_coeff(sample_rate: f32) -> f32 {
    one_pole_coeff(PRE_SOFT_LP_HZ, sample_rate)
}

pub(super) fn vintage_lpf_coeff(sample_rate: f32, drive: f32) -> f32 {
    let shaped = drive_curve(drive, 1.2);
    let cutoff = VINTAGE_LPF_BASE_HZ - (VINTAGE_LPF_REDUCTION_HZ * shaped);
    one_pole_coeff(cutoff.max(40.0), sample_rate)
}

pub(super) fn bias_reduction(drive: f32) -> f32 {
    smoothstep(DEHARSH_START, 1.0, drive)
}

pub(super) fn deharsh_amount(drive: f32) -> f32 {
    ((drive - DEHARSH_START) / (1.0 - DEHARSH_START)).clamp(0.0, 1.0)
}

pub(super) fn drive_mid_focus(drive: f32) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::soft_clip;

    #[test]
    fn soft_clip_is_bounded() {
        for value in [-10.0, -1.0, 0.0, 1.0, 10.0] {
            let shaped = soft_clip(value);
            assert!(shaped.is_finite());
            assert!(shaped.abs() <= 1.2);
        }
    }
}
