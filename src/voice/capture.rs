use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use hound::{WavWriter, WavSpec, SampleFormat as HoundSampleFormat};

pub struct VoiceRecorder;

impl VoiceRecorder {
    /// Record audio from the default input device for the given duration and
    /// save it to `output_filename` in WAV format (16-bit, 16kHz, mono).
    pub fn record_audio(
        output_filename: &str,
        duration: Duration,
    ) -> Result<(), Box<dyn Error>> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("no input device available")?;

        let default_config = device.default_input_config()?;
        let sample_format = default_config.sample_format();

        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(16_000),
            buffer_size: cpal::BufferSize::Default,
        };

        let spec = WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 16,
            sample_format: HoundSampleFormat::Int,
        };
        let writer = WavWriter::create(output_filename, spec)?;
        let writer = Arc::new(Mutex::new(writer));

        let writer_clone = writer.clone();
        let err_fn = |err| eprintln!("stream error: {}", err);

        let stream = match sample_format {
            SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _| write_input_data(data, &writer_clone),
                err_fn,
                None,
            )?,
            SampleFormat::U16 => device.build_input_stream(
                &config,
                move |data: &[u16], _| write_input_data(data, &writer_clone),
                err_fn,
                None,
            )?,
            SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _| write_input_data(data, &writer_clone),
                err_fn,
                None,
            )?,
            _ => return Err("unsupported sample format".into()),
        };

        stream.play()?;
        let start = Instant::now();
        while start.elapsed() < duration {
            std::thread::sleep(Duration::from_millis(50));
        }

        drop(stream);
        writer.lock().unwrap().finalize()?;

        Ok(())
    }
}

fn write_input_data<T>(input: &[T], writer: &Arc<Mutex<WavWriter<std::io::BufWriter<std::fs::File>>>>)
where
    T: cpal::Sample,
{
    let mut writer = writer.lock().unwrap();
    for &sample in input {
        let sample: i16 = cpal::Sample::to_i16(&sample);
        writer.write_sample(sample).ok();
    }
}

