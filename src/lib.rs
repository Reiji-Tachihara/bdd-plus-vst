use nih_plug::prelude::*;
use std::sync::Arc;

mod dsp;
mod gui;
mod params;

use params::BddPlusParams;

struct BddPlus {
    // プラグインのパラメータ一式。
    params: Arc<BddPlusParams>,
    // DSP処理の本体。
    dsp: dsp::Dsp,
}

impl Default for BddPlus {
    fn default() -> Self {
        Self {
            // 既定のパラメータ。
            params: Arc::new(BddPlusParams::default()),
            // DSP状態を初期化する。
            dsp: dsp::Dsp::default(),
        }
    }
}

impl Plugin for BddPlus {
    // プラグイン名。
    const NAME: &'static str = "bdd_plus";
    // ベンダー名。
    const VENDOR: &'static str = "Reiji Tachihara";
    // プロジェクトURL。
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    // 連絡先メール。
    const EMAIL: &'static str = "re.sg.102041@gmail.com";

    // バージョン文字列。
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // 入出力レイアウト定義。
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    // サンプル精度のオートメーションを有効化。
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        // 共有パラメータを返す。
        self.params.clone()
    }

    fn editor(
        &mut self,
        // 非同期実行用のハンドラ(未使用)。
        _async_executor: AsyncExecutor<Self>,
    ) -> Option<Box<dyn Editor>> {
        // GUIエディタを生成する。
        gui::create_editor(self.params.clone())
    }

    fn initialize(
        &mut self,
        // 入出力レイアウト。
        audio_io_layout: &AudioIOLayout,
        // バッファ設定。
        buffer_config: &BufferConfig,
        // 初期化コンテキスト(未使用)。
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // 出力チャンネル数を取得する。
        let channels = audio_io_layout
            .main_output_channels
            .map(|count| count.get() as usize)
            .unwrap_or(0);
        // DSPを初期化する。
        self.dsp.initialize(buffer_config.sample_rate, channels);
        true
    }

    fn reset(&mut self) {
        // DSP状態をリセットする。
        self.dsp.reset();
    }

    fn process(
        &mut self,
        // 処理対象バッファ。
        buffer: &mut Buffer,
        // 補助バッファ(未使用)。
        _aux: &mut AuxiliaryBuffers,
        // プロセスコンテキスト(未使用)。
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

        // 事前エンファシス係数。
        let pre_coeff = self.dsp.pre_emphasis_coeff();
        // 低域タイト化係数。
        let pre_tighten_coeff = self.dsp.pre_tighten_coeff();
        // DCブロック係数。
        let dc_block_coeff = self.dsp.dc_block_coeff();

        // チャンネル単位でサンプルを処理する。
        for mut channels in buffer.iter_samples() {
            // ドライブ値(スムージング済み)。
            let drive = self.params.drive.smoothed.next();
            // トーン値(スムージング済み)。
            let tone = self.params.tone.smoothed.next();
            // レベル値(スムージング済み)。
            let level = self.params.level.smoothed.next();

            // チャンネル内の各サンプルを処理する。
            for channel_idx in 0..channels.len() { // チャンネルインデックス。
                // SAFETY: channel_idx is within channels.len().
                // チャンネルのサンプル参照。
                let sample = unsafe { channels.get_unchecked_mut(channel_idx) };
                // DSP処理済みサンプル。
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
                // 出力を上書きする。
                *sample = processed;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for BddPlus {
    // CLAPのプラグインID。
    const CLAP_ID: &'static str = "jp.reiji.bddplus";
    // CLAP用の説明文。
    const CLAP_DESCRIPTION: Option<&'static str> = Some("My favorite effecter");
    // マニュアルURL。
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    // サポートURLは未設定。
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // CLAPの機能タグ。
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for BddPlus {
    // VST3クラスID。
    const VST3_CLASS_ID: [u8; 16] = *b"JPREIJIBDDPLUS01";
    // VST3カテゴリ。
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

// CLAPエクスポート。
nih_export_clap!(BddPlus);
// VST3エクスポート。
nih_export_vst3!(BddPlus);
