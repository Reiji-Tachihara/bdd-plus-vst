use nih_plug::prelude::util;

use super::constants::*;
use super::filters::one_pole_coeff;

/// ソフトクリップ関数。
pub(super) fn soft_clip(
    // 入力サンプル。
    input: f32,
) -> f32 {
    // 入力をソフトクリップする。
    input.tanh()
}
/// 第1段ドライブゲインを算出する。
pub(super) fn drive_stage1_gain(
    // ドライブ値。
    drive: f32,
) -> f32 {
    // ドライブ値を曲線で整形する。
    let shaped = drive_curve(drive, 2.2);
    // 第1段のゲインに変換する。
    util::db_to_gain(shaped * DRIVE_STAGE1_DB)
}
/// 事前ブーストゲインを算出する。
pub(super) fn drive_to_pre_boost(
    // ドライブ値。
    drive: f32,
) -> f32 {
    // 0..1に制限したドライブ値。
    let shaped = drive.clamp(0.0, 1.0);
    // 事前ブーストのゲインに変換する。
    util::db_to_gain(DRIVE_PRE_BOOST_DB * shaped)
}
/// 第2段ドライブのミックス比率を算出する。
pub(super) fn drive_stage2_gain(
    // ドライブ値。
    drive: f32,
) -> f32 {
    // 中域フォーカスの強さ。
    let mid = drive_mid_focus(drive);
    // 第2段ゲインを補間する。
    SECOND_STAGE_MIN_GAIN + (SECOND_STAGE_MAX_GAIN - SECOND_STAGE_MIN_GAIN) * mid
}
/// 第2段のミックス比率を算出する。
pub(super) fn drive_stage2_mix(
    // ドライブ値。
    drive: f32,
) -> f32 {
    // 中域フォーカスの強さ。
    let mid = drive_mid_focus(drive);
    // 第2段ブレンド比率を算出する。
    SECOND_STAGE_MIX_MAX * mid
}
/// ドライブ値をレベル補正ゲインに変換する。
pub(super) fn drive_to_compensation(
    // ドライブ値。
    drive: f32,
) -> f32 {
    // ドライブ値を補正曲線で整形する。
    let shaped = drive_curve(drive, 1.4);
    // レベル補正ゲインに変換する。
    util::db_to_gain(-5.0 * shaped)
}
/// レベル値をリニアゲインに変換する。
pub(super) fn level_to_gain(
    // レベル値。
    level: f32,
) -> f32 {
    // レベル値をdBに変換する。
    let db = -60.0 + (level.clamp(0.0, 1.0) * 66.0);
    // dBをリニアゲインに変換する。
    util::db_to_gain(db)
}

/// トーン値をローパスフィルタ係数に変換する。
pub(super) fn tone_to_coeff(
    // トーン値。
    tone: f32,
    // サンプルレート。
    sample_rate: f32,
    // デハーシュ量。
    deharsh: f32,
) -> f32 {
    // トーン値をなだらかに整形する。
    let shaped = tone.clamp(0.0, 1.0).powf(0.7);
    // トーンのカットオフ周波数。
    let cutoff = TONE_MIN_HZ + (TONE_MAX_HZ - TONE_MIN_HZ) * shaped;
    // デハーシュ分の補正を反映する。
    let adjusted = cutoff * (1.0 - (DEHARSH_CUTOFF_SCALE * deharsh));
    // 1次ローパス係数に変換する。
    one_pole_coeff(adjusted, sample_rate)
}
/// エンファシス用ローパス係数を計算する。
pub(super) fn pre_emphasis_coeff(
    // サンプルレート。
    sample_rate: f32,
) -> f32 {
    // エンファシス用ローパス係数を計算する。
    one_pole_coeff(PRE_EMPH_HZ, sample_rate)
}
/// タイトニング用ローパス係数を計算する。
pub(super) fn pre_tighten_coeff(
    // サンプルレート。
    sample_rate: f32,
) -> f32 {
    // 低域を締めるローパス係数を計算する。
    one_pole_coeff(PRE_HP_HZ, sample_rate)
}

pub(super) fn dc_block_coeff(
    // サンプルレート。
    sample_rate: f32,
) -> f32 {
    // DCブロッカ用係数を計算する。
    one_pole_coeff(DC_BLOCK_HZ, sample_rate)
}
/// 段間ローパス係数を計算する。
pub(super) fn interstage_coeff(
    // サンプルレート。
    sample_rate: f32,
) -> f32 {
    // 段間ローパス係数を計算する。
    one_pole_coeff(INTERSTAGE_LP_HZ, sample_rate * 2.0)
}
/// 事後ローパス係数を計算する。
pub(super) fn post_lpf_coeff(
    // サンプルレート。
    sample_rate: f32,
    // デハーシュ量。
    deharsh: f32,
) -> f32 {
    // デハーシュ分を加味したカットオフ。
    let cutoff = (POST_LPF_BASE_HZ - (POST_LPF_REDUCTION_HZ * deharsh)).max(40.0);
    // 事後ローパス係数を計算する。
    one_pole_coeff(cutoff, sample_rate)
}
/// 事後ローパスの混合量を算出する。
pub(super) fn post_lpf_mix(
    // デハーシュ量。
    deharsh: f32,
) -> f32 {
    // 事後ローパスの混合量。
    POST_LPF_MIX_MAX * deharsh
}
/// アンチエイリアス用ローパス係数を計算する。
pub(super) fn aa_coeff(
    // サンプルレート。
    sample_rate: f32,
) -> f32 {
    // 2x時のアンチエイリアス係数を計算する。
    one_pole_coeff(sample_rate * AA_CUTOFF_RATIO, sample_rate * 2.0)
}
/// 事前ソフト化ローパス係数を計算する。
pub(super) fn pre_soften_coeff(
    // サンプルレート。
    sample_rate: f32,
) -> f32 {
    // 事前ソフト化用ローパス係数。
    one_pole_coeff(PRE_SOFT_LP_HZ, sample_rate)
}
/// ビンテージローパス係数を計算する。
pub(super) fn vintage_lpf_coeff(
    // サンプルレート。
    sample_rate: f32,
    // ドライブ値。
    drive: f32,
) -> f32 {
    // ドライブ値を整形する。
    let shaped = drive_curve(drive, 1.2);
    // ビンテージローパスのカットオフ。
    let cutoff = VINTAGE_LPF_BASE_HZ - (VINTAGE_LPF_REDUCTION_HZ * shaped);
    // 係数へ変換する。
    one_pole_coeff(cutoff.max(40.0), sample_rate)
}
/// バイアス減少量を算出する。
pub(super) fn bias_reduction(
    // ドライブ値。
    drive: f32,
) -> f32 {
    // バイアス減少カーブ。
    smoothstep(DEHARSH_START, 1.0, drive)
}
/// デハーシュ量を算出する。
pub(super) fn deharsh_amount(
    // ドライブ値。
    drive: f32,
) -> f32 {
    // デハーシュの量を0..1に正規化。
    ((drive - DEHARSH_START) / (1.0 - DEHARSH_START)).clamp(0.0, 1.0)
}
/// 中域フォーカスの強さを算出する。
pub(super) fn drive_mid_focus(
    // ドライブ値。
    drive: f32,
) -> f32 {
    // 中域フォーカスの曲線。
    smoothstep(0.3, 0.7, drive)
}
/// ドライブ値を曲線で整形する。
fn drive_curve(
    // ドライブ値。
    drive: f32,
    // 曲線の指数。
    power: f32,
) -> f32 {
    // ニーで正規化したドライブ値。
    let shaped = (drive / DRIVE_KNEE).clamp(0.0, 1.0);
    // 曲線で強調する。
    shaped.powf(power)
}
/// スムースステップ補間を行う。
fn smoothstep(
    // 下限エッジ。
    edge0: f32,
    // 上限エッジ。
    edge1: f32,
    // 入力値。
    x: f32,
) -> f32 {
    // 0..1へ正規化した補間値。
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    // 3次で滑らかに補間する。
    t * t * (3.0 - 2.0 * t)
}
//soft_clip() が「有限な値を返し、極端に振り切れない」ことを確認
#[cfg(test)]
mod tests {
    use super::soft_clip;

    #[test]
    fn soft_clip_is_bounded() {
        for value in [-10.0, -1.0, 0.0, 1.0, 10.0] {
            // テスト入力値。
            // クリップ結果。
            let shaped = soft_clip(value);
            assert!(shaped.is_finite());
            assert!(shaped.abs() <= 1.2);
        }
    }
}
