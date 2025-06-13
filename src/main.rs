use clap::Parser;

mod voice;
mod llm;

use voice::capture::VoiceRecorder;
use voice::transcribe::Transcriber;
use llm::gemini_client::GeminiClient;

#[derive(Parser, Debug)]
#[command(author, version, about = "Voice to code demo")] 
struct Args {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _args = Args::parse();

    let temp_wav = "temp.wav";

    VoiceRecorder::record_audio(temp_wav, 5)?;

    let transcriber = Transcriber::new();
    let transcript = transcriber.transcribe_audio(temp_wav)?;

    let client = GeminiClient::new();
    let response = client.generate(&transcript).await?;

    println!("{}", response);

    Ok(())
}
