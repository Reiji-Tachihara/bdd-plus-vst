// 事前ハイパスのカットオフ周波数。
pub(super) const PRE_HP_HZ: f32 = 100.0;
// 事前エンファシスのカットオフ周波数。
pub(super) const PRE_EMPH_HZ: f32 = 600.0;
// 事前EQの中心周波数。
pub(super) const PRE_EQ_FREQ_HZ: f32 = 120.0;
// 事前EQのQ値。
pub(super) const PRE_EQ_Q: f32 = 0.6;
// 事前EQのゲイン(dB)。
pub(super) const PRE_EQ_GAIN_DB: f32 = -1.0;
// トーン最小カットオフ周波数。
pub(super) const TONE_MIN_HZ: f32 = 700.0;
// トーン最大カットオフ周波数。
pub(super) const TONE_MAX_HZ: f32 = 5200.0;
// DCブロッカのカットオフ周波数。
pub(super) const DC_BLOCK_HZ: f32 = 18.0;
// 非対称クリップのバイアス量。
pub(super) const ASYM_BIAS: f32 = 0.08;
// ドライブ曲線のニー位置。
pub(super) const DRIVE_KNEE: f32 = 0.6;
// 第1段クリップの基準ゲイン(dB)。
pub(super) const DRIVE_STAGE1_DB: f32 = 30.0;
// 事前ブースト量(dB)。
pub(super) const DRIVE_PRE_BOOST_DB: f32 = 6.0;
// 第2段ゲインの最小値。
pub(super) const SECOND_STAGE_MIN_GAIN: f32 = 1.0;
// 第2段ゲインの最大値。
pub(super) const SECOND_STAGE_MAX_GAIN: f32 = 2.0;
// 第2段ブレンドの最大値。
pub(super) const SECOND_STAGE_MIX_MAX: f32 = 0.75;
// 段間ローパスのカットオフ周波数。
pub(super) const INTERSTAGE_LP_HZ: f32 = 7500.0;
// 事後ローパスの基準カットオフ周波数。
pub(super) const POST_LPF_BASE_HZ: f32 = 11200.0;
// 事後ローパスの減衰量(Hz)。
pub(super) const POST_LPF_REDUCTION_HZ: f32 = 3800.0;
// 事後ローパスのミックス上限。
pub(super) const POST_LPF_MIX_MAX: f32 = 0.6;
// ビンテージローパスの基準カットオフ周波数。
pub(super) const VINTAGE_LPF_BASE_HZ: f32 = 9800.0;
// ビンテージローパスの減衰量(Hz)。
pub(super) const VINTAGE_LPF_REDUCTION_HZ: f32 = 1800.0;
// アンチエイリアスのカットオフ比率。
pub(super) const AA_CUTOFF_RATIO: f32 = 0.45;
// 事前ソフト化ローパスのカットオフ周波数。
pub(super) const PRE_SOFT_LP_HZ: f32 = 16000.0;
// 事前ソフト化の混合比率。
pub(super) const PRE_SOFTEN_MIX: f32 = 0.08;
// デハーシュ開始ドライブ値。
pub(super) const DEHARSH_START: f32 = 0.5;
// デハーシュ用カットオフ調整比。
pub(super) const DEHARSH_CUTOFF_SCALE: f32 = 0.12;
// バイアス低減の比率。
pub(super) const ASYM_BIAS_REDUCTION: f32 = 0.65;
// デノーマル回避用の微小値。
pub(super) const ANTI_DENORMAL: f32 = 1.0e-24;
