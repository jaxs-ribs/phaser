use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

#[derive(Debug)]
pub struct GeminiClient {
    client: Client,
    api_key: String,
    base_url: String,
    request_count: Arc<AtomicU32>,
    max_requests_per_session: u32,
    max_prompt_length: usize,
}

impl GeminiClient {
    /// Initialize the GeminiClient with API key from GEMINI_API_KEY environment variable
    /// Includes built-in spending protection limits
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| "GEMINI_API_KEY environment variable not set")?;

        let client = Client::new();
        let base_url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent".to_string();

        // Spending protection: Limit requests per session and prompt length
        let max_requests_per_session = env::var("GEMINI_MAX_REQUESTS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .unwrap_or(10);

        let max_prompt_length = env::var("GEMINI_MAX_PROMPT_LENGTH")
            .unwrap_or_else(|_| "2000".to_string())
            .parse::<usize>()
            .unwrap_or(2000);

        Ok(GeminiClient {
            client,
            api_key,
            base_url,
            request_count: Arc::new(AtomicU32::new(0)),
            max_requests_per_session,
            max_prompt_length,
        })
    }

    /// Generate a response from Gemini API for the given prompt with spending controls
    pub async fn generate(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        // Spending protection: Check request count limit
        let current_count = self.request_count.fetch_add(1, Ordering::SeqCst);
        if current_count >= self.max_requests_per_session {
            return Err(format!(
                "Request limit exceeded: {}/{} requests used in this session. Restart to reset.",
                current_count + 1,
                self.max_requests_per_session
            ).into());
        }

        // Spending protection: Check prompt length limit
        if prompt.len() > self.max_prompt_length {
            return Err(format!(
                "Prompt too long: {} characters (max: {}). Truncate your input.",
                prompt.len(),
                self.max_prompt_length
            ).into());
        }

        println!("🔒 API Request {}/{} | Prompt length: {}/{}", 
                current_count + 1, 
                self.max_requests_per_session,
                prompt.len(),
                self.max_prompt_length);
        
        let request_body = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
        };

        let response = self
            .client
            .post(&self.base_url)
            .query(&[("key", &self.api_key)])
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Gemini API error {}: {}", status, error_text).into());
        }

        let gemini_response: GeminiResponse = response.json().await?;

        // Extract the response text
        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }

        Err("No response content found in Gemini API response".into())
    }

    /// Generate a code-focused response by adding context to the prompt
    pub async fn generate_code_suggestion(&self, transcribed_text: &str) -> Result<String, Box<dyn Error>> {
        // Truncate transcribed text if it's too long to prevent runaway costs
        let truncated_text = if transcribed_text.len() > 500 {
            format!("{}... [truncated]", &transcribed_text[..500])
        } else {
            transcribed_text.to_string()
        };

        let enhanced_prompt = format!(
            "You are a coding assistant. The user said: \"{}\"\n\nPlease provide a concise, helpful code-related response or solution (max 200 words). If the input is not code-related, politely explain that you focus on coding assistance.",
            truncated_text
        );

        self.generate(&enhanced_prompt).await
    }

    /// Get current usage statistics
    pub fn get_usage_stats(&self) -> (u32, u32) {
        let current = self.request_count.load(Ordering::SeqCst);
        (current, self.max_requests_per_session)
    }

    /// Reset request counter (for testing or manual reset)
    pub fn reset_usage(&self) {
        self.request_count.store(0, Ordering::SeqCst);
        println!("🔄 Usage counter reset");
    }
}