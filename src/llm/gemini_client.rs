use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use serde_json::json;

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
    api_key: Option<String>,
    base_url: String,
    use_openrouter: bool,
    openrouter_key: Option<String>,
    openrouter_model: String,
    request_count: Arc<AtomicU32>,
    max_requests_per_session: u32,
    max_prompt_length: usize,
}

impl GeminiClient {
    /// Initialize the GeminiClient with API key from GEMINI_API_KEY environment variable
    /// Includes built-in spending protection limits
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let api_key = env::var("GEMINI_API_KEY").ok();

        let client = Client::new();

        // Allow model override via env, default to Gemini 2.5 Pro
        let model_name = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-pro".to_string());

        let base_url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model_name);

        // Spending protection: Limit requests per session and prompt length
        let max_requests_per_session = env::var("GEMINI_MAX_REQUESTS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .unwrap_or(10);

        let max_prompt_length = env::var("GEMINI_MAX_PROMPT_LENGTH")
            .unwrap_or_else(|_| "2000".to_string())
            .parse::<usize>()
            .unwrap_or(2000);

        // Detect OpenRouter usage
        let openrouter_key = env::var("OPENROUTER_API_KEY").ok();
        let use_openrouter = openrouter_key.is_some();
        let openrouter_model = env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "google/gemini-2.5-pro-preview-06-05".to_string());

        Ok(GeminiClient {
            client,
            api_key,
            base_url,
            use_openrouter,
            openrouter_key,
            openrouter_model,
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
        
        let response_text = if self.use_openrouter {
            // Build OpenAI style JSON
            let payload = json!({
                "model": self.openrouter_model,
                "messages": [{"role": "user", "content": prompt}],
            });

            let resp = self.client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", self.openrouter_key.as_ref().unwrap()))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err = resp.text().await.unwrap_or_default();
                return Err(format!("OpenRouter error {}: {}", status, err).into());
            }

            let val: serde_json::Value = resp.json().await?;
            val["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()
        } else {
            if self.api_key.as_deref().unwrap_or("").is_empty() {
                return Err("GEMINI_API_KEY not set".into());
            }

            let request_body = GeminiRequest {
                contents: vec![Content {
                    parts: vec![Part {
                        text: prompt.to_string(),
                    }],
                }],
            };

            let resp = self
                .client
                .post(&self.base_url)
                .query(&[("key", self.api_key.as_ref().unwrap())])
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let error_text = resp.text().await.unwrap_or_default();
                return Err(format!("Gemini API error {}: {}", status, error_text).into());
            }

            let gemini_response: GeminiResponse = resp.json().await?;

            if let Some(candidate) = gemini_response.candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    part.text.clone()
                } else { "".to_string() }
            } else { "".to_string() }
        };

        if response_text.is_empty() {
            Err("Empty response text".into())
        } else {
            Ok(response_text)
        }
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