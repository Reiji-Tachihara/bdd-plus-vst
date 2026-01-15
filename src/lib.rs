use nih_plug::prelude::*;
use std::sync::Arc;

mod dsp;
mod params;

use params::BddPlusParams;

struct BddPlus {
    params: Arc<BddPlusParams>,
    dsp: dsp::Dsp,
}

impl Default for BddPlus {
    fn default() -> Self {
        Self {
            params: Arc::new(BddPlusParams::default()),
            dsp: dsp::Dsp::default(),
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
