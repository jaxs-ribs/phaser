use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug)]
pub struct GeminiClient {
    api_key: String,
    http_client: Client,
}

impl GeminiClient {
    /// Create a new `GeminiClient` using the `GEMINI_API_KEY` environment variable.
    pub fn new() -> Result<Self, env::VarError> {
        let api_key = env::var("GEMINI_API_KEY")?;
        Ok(Self {
            api_key,
            http_client: Client::new(),
        })
    }

    /// Send a prompt to the Gemini API and return the generated text.
    pub async fn generate(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}",
            self.api_key
        );

        let request_body = GenerateRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
        };

        let resp = self
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(format!("Request failed with status: {}", resp.status()).into());
        }

        let response_body: GenerateResponse = resp.json().await?;
        let text = response_body
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .and_then(|p| p.text.clone())
            .unwrap_or_default();

        Ok(text)
    }
}

#[derive(Serialize)]
struct GenerateRequest {
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
struct GenerateResponse {
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
    text: Option<String>,
}
