use std::env;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

/// Handles transcription of WAV audio files using a Whisper model.
pub struct Transcriber {
    ctx: WhisperContext,
}

impl Transcriber {
    /// Create a new `Transcriber` using the `WHISPER_MODEL_PATH` environment variable.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let model_path = env::var("WHISPER_MODEL_PATH")
            .map_err(|_| "WHISPER_MODEL_PATH not set")?;
        let ctx = WhisperContext::new(&model_path)?;
        Ok(Self { ctx })
    }

    /// Transcribe the given WAV file and return the resulting text.
    pub fn transcribe_audio<P: AsRef<Path>>(
        &self,
        input_filename: P,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Read samples from the WAV file
        let mut reader = hound::WavReader::open(&input_filename)?;
        let samples: Vec<f32> = reader
            .samples::<i16>()
            .map(|s| s.unwrap() as f32 / i16::MAX as f32)
            .collect();

        let mut state = self.ctx.create_state()?;
        let mut params = FullParams::new(SamplingStrategy::default());
        state.full(&mut params, &samples)?;

        let mut transcript = String::new();
        let num_segments = state.full_n_segments();
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i)?;
            transcript.push_str(&segment);
        }
        Ok(transcript)
    }
}
