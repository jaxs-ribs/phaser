use std::error::Error;

pub struct GeminiClient;

impl GeminiClient {
    pub fn new() -> Self {
        GeminiClient
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        // Placeholder implementation for task 1.4 demo
        println!("Sending prompt to Gemini: {} (stub)", prompt);
        Ok("Gemini response".to_string())
    }
}
