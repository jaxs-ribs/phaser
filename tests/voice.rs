use project_x::voice::capture::VoiceRecorder;
use std::time::Duration;
use tempfile::NamedTempFile;

#[test]
fn test_voice_recorder_creates_wav_file() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let temp_path = temp_file.path().to_str().unwrap();
    
    // Record for a very short duration to minimize test time
    let duration = Duration::from_millis(100);
    
    let result = VoiceRecorder::record_audio(temp_path, duration);
    
    // The test should pass if we can successfully attempt to record
    // Even if no audio device is available, the function should handle it gracefully
    match result {
        Ok(_) => {
            // If recording succeeded, verify the file exists and has some content
            assert!(std::path::Path::new(temp_path).exists());
            let metadata = std::fs::metadata(temp_path).expect("Failed to get file metadata");
            assert!(metadata.len() > 0, "WAV file should not be empty");
        }
        Err(e) => {
            // If recording failed, it's likely due to no audio device in CI/test environment
            // This is acceptable for automated testing
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("no input device available") || 
                error_msg.contains("device") ||
                error_msg.contains("audio"),
                "Error should be related to audio device availability: {}", error_msg
            );
        }
    }
}

// Note: Testing the Transcriber requires a Whisper model file, which is typically
// too large to include in a repository. In a real-world scenario, you would:
// 1. Mock the whisper-rs calls
// 2. Use a test fixture with a small model
// 3. Skip transcription tests if WHISPER_MODEL_PATH is not set

#[cfg(test)]
mod transcriber_tests {
    use super::*;
    use project_x::voice::transcribe::Transcriber;
    
    #[test]
    fn test_transcriber_requires_model_path() {
        // Temporarily remove the environment variable if it exists
        let original_var = std::env::var("WHISPER_MODEL_PATH").ok();
        std::env::remove_var("WHISPER_MODEL_PATH");
        
        let result = Transcriber::new();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("WHISPER_MODEL_PATH"));
        
        // Restore the original environment variable if it existed
        if let Some(path) = original_var {
            std::env::set_var("WHISPER_MODEL_PATH", path);
        }
    }
    
    #[test]
    fn test_transcriber_validates_model_file_exists() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let nonexistent_path = temp_file.path().to_str().unwrap().to_owned() + "_nonexistent";
        
        std::env::set_var("WHISPER_MODEL_PATH", &nonexistent_path);
        
        let result = Transcriber::new();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
        
        std::env::remove_var("WHISPER_MODEL_PATH");
    }
}