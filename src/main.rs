use clap::Parser;
use project_x::voice::capture::VoiceRecorder;
use project_x::voice::transcribe::Transcriber;
use project_x::llm::gemini_client::GeminiClient;
use std::time::Duration;
use tokio;

#[derive(Parser)]
#[clap(name = "project-x")]
#[clap(about = "AI-First Coding Assistant - Voice to Code Pipeline")]
struct Cli {
    /// Duration to record audio in seconds
    #[clap(short, long, default_value = "5")]
    duration: u64,
    
    /// Output path for temporary WAV file
    #[clap(short, long, default_value = "temp_audio.wav")]
    output: String,
    
    /// Show API usage statistics
    #[clap(long)]
    show_usage: bool,
    
    /// Test Gemini API with a simple prompt (bypasses voice recording)
    #[clap(long)]
    test_llm: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize Gemini client first (needed for all operations)
    let gemini_client = GeminiClient::new()?;
    
    // Handle test LLM functionality
    if let Some(test_prompt) = &cli.test_llm {
        println!("🤖 Testing Gemini API with prompt: \"{}\"", test_prompt);
        
        let response = gemini_client.generate_code_suggestion(test_prompt).await?;
        println!("✅ Gemini response:");
        println!("{}", response);
        
        let (used, max) = gemini_client.get_usage_stats();
        println!("📊 API Usage: {}/{} requests remaining", max - used, max);
        
        return Ok(());
    }
    
    // Handle usage stats display
    if cli.show_usage {
        let (used, max) = gemini_client.get_usage_stats();
        println!("📊 API Usage: {}/{} requests", used, max);
        return Ok(());
    }
    
    println!("🎤 Starting voice-to-code pipeline...");
    
    // Step 1: Record audio
    println!("📢 Recording audio for {} seconds...", cli.duration);
    let duration = Duration::from_secs(cli.duration);
    VoiceRecorder::record_audio(&cli.output, duration)?;
    println!("✅ Audio recorded to: {}", cli.output);
    
    // Step 2: Transcribe audio
    println!("🔤 Transcribing audio to text...");
    let mut transcriber = Transcriber::new()?;
    let transcribed_text = transcriber.transcribe_audio(&cli.output)?;
    println!("✅ Transcription: \"{}\"", transcribed_text);
    
    // Step 3: Get response from Gemini
    println!("🤖 Sending to Gemini for code suggestion...");
    
    let response = gemini_client.generate_code_suggestion(&transcribed_text).await?;
    println!("✅ Gemini response:");
    println!("{}", response);
    
    // Show usage after request
    let (used, max) = gemini_client.get_usage_stats();
    println!("📊 API Usage: {}/{} requests remaining", max - used, max);
    
    // Clean up temporary file
    if std::path::Path::new(&cli.output).exists() {
        std::fs::remove_file(&cli.output)?;
        println!("🧹 Cleaned up temporary audio file");
    }
    
    Ok(())
}
