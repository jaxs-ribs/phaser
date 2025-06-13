use project_x::orchestrator::context::ContextBuilder;
use project_x::llm::gemini_client::GeminiClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🤖 TDD Demo - Testing LLM Diff Generation");
    
    let gemini_client = GeminiClient::new()?;
    let context_builder = ContextBuilder::new();
    
    let user_request = "Add a simple hello_world function to src/lib.rs";
    
    // Build context
    let context = context_builder.build_smart_context(user_request, ".", 2)?;
    
    let lib_rs_content = context
        .files
        .iter()
            .find(|f| f.path == "src/lib.rs")
        .map(|f| f.content.as_str())
        .unwrap_or("File not found");

    let llm_prompt = format!(
        r#"
You are an expert Rust programmer working as a command-line tool.
Your SOLE purpose is to generate a valid, standard unified diff.
Do NOT provide any explanation, preamble, or narrative.
Do NOT wrap the diff in markdown code blocks.
The user wants to add a simple hello_world function to src/lib.rs.
The current content of the file is provided below.

---
{lib_rs_content}
---

Generate the unified diff to add the 'hello_world' function."#
    );

    println!("\n--- Sending this prompt to Gemini ---");
    println!("{}", llm_prompt);
    
    println!("📝 Sending focused prompt to LLM...");
    let response = gemini_client.generate(&llm_prompt).await?;
    
    println!("\n📄 LLM Response:");
    println!("--- START RESPONSE ---");
    println!("{}", response);
    println!("--- END RESPONSE ---");
    
    // Test if it's a valid diff
    if response.contains("--- a/") && response.contains("+++ b/") {
        println!("\n✅ Response appears to be a valid unified diff");
    } else {
        println!("\n❌ Response is not a valid unified diff format");
    }
    
    Ok(())
}