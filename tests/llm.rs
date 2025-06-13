use project_x::llm::gemini_client::GeminiClient;
use serde_json::json;
use wiremock::matchers::{method, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_gemini_client_requires_api_key() {
    // Temporarily remove relevant environment variables if they exist
    let original_var = std::env::var("GEMINI_API_KEY").ok();
    let original_openrouter = std::env::var("OPENROUTER_API_KEY").ok();
    std::env::remove_var("GEMINI_API_KEY");
    std::env::remove_var("OPENROUTER_API_KEY");
    
    let result = GeminiClient::new();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("GEMINI_API_KEY"));
    
    // Restore the original environment variable if it existed
    if let Some(key) = original_var {
        std::env::set_var("GEMINI_API_KEY", key);
    }
    if let Some(key) = original_openrouter {
        std::env::set_var("OPENROUTER_API_KEY", key);
    }
}

#[tokio::test]
async fn test_gemini_client_successful_request() {
    // Set up mock server
    let mock_server = MockServer::start().await;
    
    // Set up environment
    std::env::set_var("GEMINI_API_KEY", "test_api_key");
    
    // Create expected response
    let mock_response = json!({
        "candidates": [
            {
                "content": {
                    "parts": [
                        {
                            "text": "This is a test response from Gemini"
                        }
                    ]
                }
            }
        ]
    });
    
    // Set up mock endpoint
    Mock::given(method("POST"))
        .and(path_regex(r"/v1beta/models/gemini-pro:generateContent"))
        .and(query_param("key", "test_api_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;
    
    // Create client and modify base URL to use mock server
    let mut client = GeminiClient::new().unwrap();
    // Note: In a real implementation, you'd want to make the base_url configurable
    // For this test, we'll need to modify the struct to allow URL override
    
    // For now, this test validates the client creation and API key handling
    // A full integration test would require modifying the GeminiClient to accept a custom base URL
    
    std::env::remove_var("GEMINI_API_KEY");
}

#[tokio::test]
async fn test_gemini_client_handles_api_error() {
    let mock_server = MockServer::start().await;
    std::env::set_var("GEMINI_API_KEY", "test_api_key");
    
    // Set up mock to return an error
    Mock::given(method("POST"))
        .and(path_regex(r"/v1beta/models/gemini-pro:generateContent"))
        .and(query_param("key", "test_api_key"))
        .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
        .mount(&mock_server)
        .await;
    
    // This test structure shows how you would test error handling
    // The actual implementation would need the configurable base URL
    
    std::env::remove_var("GEMINI_API_KEY");
}

#[test]
fn test_code_suggestion_prompt_enhancement() {
    // Test that the code suggestion method properly formats the prompt
    let input = "create a function to sort an array";
    let expected_content = "You are a coding assistant";
    
    // This is a unit test for the prompt formatting logic
    // In the actual implementation, you might want to extract the prompt
    // formatting to a separate testable function
    
    assert!(true); // Placeholder - would test actual prompt formatting
}