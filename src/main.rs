use clap::Parser;
use project_x::voice::capture::VoiceRecorder;
use project_x::voice::transcribe::Transcriber;
use project_x::llm::gemini_client::GeminiClient;
use project_x::orchestrator::context::ContextBuilder;
use project_x::orchestrator::memory::{MemoryManager, MessageRole};
use project_x::edit::patch::CodePatcher;
use project_x::hooks::autotest::TestExecutor;
use std::time::Duration;
use tokio;
use std::env;
use tempfile::{tempdir, TempDir};
use std::path::PathBuf;
use fs_extra;

#[derive(Parser)]
#[clap(name = "project-x")]
#[clap(about = "AI-First Coding Assistant - Autonomous TDD Loop")]
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
    
    /// Autonomous coding prompt - the AI will edit code, test, and fix issues
    #[clap(long)]
    prompt: Option<String>,
    
    /// Maximum number of retry attempts for failed tests
    #[clap(long, default_value = "3")]
    max_retries: usize,
    
    /// Dry run mode - show what would be done without applying changes
    #[clap(long)]
    dry_run: bool,
    
    /// Skip tests - apply changes without running test suite
    #[clap(long)]
    skip_tests: bool,

    /// Run in a temporary sandbox directory to avoid modifying project files
    #[clap(long)]
    sandbox: bool,

    /// If using --sandbox, keep the directory after completion for inspection
    #[clap(long)]
    keep_sandbox: bool,
}

struct Sandbox {
    _temp_dir: TempDir,
    path: PathBuf,
}

impl Sandbox {
    fn new() -> Result<Self, Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let path = temp_dir.path().to_path_buf();
        println!("✨ Created sandbox directory at: {}", path.display());

        let mut copy_options = fs_extra::dir::CopyOptions::new();
        copy_options.copy_inside = true;
        fs_extra::dir::copy(".", &path, &copy_options)?;
        println!("🖨️  Copied project to sandbox.");

        Ok(Sandbox {
            _temp_dir: temp_dir,
            path,
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize Gemini client first (needed for all operations)
    let gemini_client = GeminiClient::new()?;
    
    // Handle autonomous coding prompt - THE CORE SELF-EDITING LOOP
    if let Some(user_prompt) = &cli.prompt {
        return run_autonomous_tdd_loop(user_prompt, &cli, &gemini_client).await;
    }
    
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

/// THE AUTONOMOUS TDD LOOP - The core self-editing capability
async fn run_autonomous_tdd_loop(
    user_prompt: &str, 
    cli: &Cli,
    gemini_client: &GeminiClient
) -> Result<(), Box<dyn std::error::Error>> {
    if cli.sandbox {
        let sandbox = Sandbox::new()?;
        let original_dir = env::current_dir()?;
        env::set_current_dir(sandbox.path())?;

        // Run the actual loop inside the sandbox
        let result = run_loop_logic(user_prompt, cli, gemini_client).await;

        // Return to original directory
        env::set_current_dir(original_dir)?;

        if cli.keep_sandbox {
            println!("✅ Sandbox retained at: {}", sandbox.path().display());
            // To prevent the TempDir from being dropped and deleting the directory
            std::mem::forget(sandbox);
        }
        return result;
    }

    // If not in sandbox, run directly
    run_loop_logic(user_prompt, cli, gemini_client).await
}

/// The core logic of the autonomous loop (refactored to be called from sandbox or directly)
async fn run_loop_logic(
    user_prompt: &str,
    cli: &Cli,
    gemini_client: &GeminiClient,
) -> Result<(), Box<dyn Error>> {
    println!("🤖 AUTONOMOUS TDD LOOP ACTIVATED");
    println!("📝 User Request: \"{}\"", user_prompt);
    println!("🔧 Max Retries: {}", cli.max_retries);
    if cli.dry_run {
        println!("🏃 DRY RUN MODE - No actual changes will be made");
    }
    
    // Initialize components
    let context_builder = ContextBuilder::new();
    let code_patcher = CodePatcher::with_options(cli.dry_run, true);
    let test_executor = TestExecutor::new();
    let mut memory = MemoryManager::new(Some("autonomous_session.sqlite"))?;
    
    // Save the user request to memory
    memory.add_message(MessageRole::User, user_prompt)?;
    
    // Get project context
    let _project_root = std::env::current_dir()?.to_string_lossy().to_string();
    
    for attempt in 1..=cli.max_retries {
        println!("\n🔄 === ATTEMPT {} of {} ===", attempt, cli.max_retries);
        
        // Step 1: Build Context - Read relevant files
        println!("📚 Step 1: Building context...");
        let context = context_builder.build_smart_context(user_prompt, ".", 3)?;
        
        // Step 2: Generate LLM Prompt
        println!("🧠 Step 2: Generating LLM prompt...");

        let context_str = context_builder.format_for_llm(&context, user_prompt);
        let history_str = if attempt > 1 {
            format!(
                "Your previous attempt failed. Here is the conversation history for context:\n---\n{}\n---",
                memory.format_context(10)?
            )
        } else {
            String::new()
        };
        
        let llm_prompt = format!(
            r#"
You are an expert Rust programmer working as a command-line tool.
Your SOLE purpose is to generate a valid, standard unified diff for the user's request.
Do NOT provide any explanation, preamble, or narrative.
Do NOT wrap the diff in markdown code blocks.

USER REQUEST: "{user_prompt}"

{history_str}

Here are the relevant files and their contents:
---
{context_str}
---

Generate the unified diff to accomplish the user request."#
        );
        
        // Step 3: Get LLM Response
        println!("🤖 Step 3: Getting LLM response...");
        let llm_response = gemini_client.generate(&llm_prompt).await?;
        
        // Extract diff from markdown/code fences if present (robust)
        let diff_content = {
            // Try to find the first opening triple-back-tick fence
            if let Some(mut start) = llm_response.find("```") {
                start += 3; // skip the opening fence
                // If the fence is ```diff, skip the word "diff" (and an optional newline)
                let mut slice = &llm_response[start..];
                if slice.starts_with("diff") {
                    slice = &slice[4..];
                }
                // Trim a single leading newline if present
                if slice.starts_with('\n') {
                    slice = &slice[1..];
                }
                // Find the closing fence
                if let Some(end_rel) = slice.find("```") {
                    slice[..end_rel].trim().to_string()
                } else {
                    // No closing fence – return remainder
                    slice.trim().to_string()
                }
            } else {
                // No fence at all – return as-is
                llm_response.trim().to_string()
            }
        };
        
        // Clean up diff lines that contain leading line-number pipes (e.g. "   7 | pub mod hooks;")
        let cleaned_diff: String = diff_content
            .lines()
            .map(|line| {
                // remove patterns like "   7 | " at the start of the line
                if let Some(pos) = line.find('|') {
                    // only treat as metadata if everything before the pipe is whitespace or digits
                    let (left, right) = line.split_at(pos);
                    if left.trim().chars().all(|c| c.is_ascii_digit()) {
                        return right[1..].trim_start().to_string(); // skip the '|' and any following space
                    }
                }
                line.to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Keep only lines that look like a proper unified diff line
        let sanitised_diff: String = cleaned_diff
            .lines()
            .filter(|line| {
                line.starts_with("--- ")
                    || line.starts_with("+++ ")
                    || line.starts_with("@@")
                    || line.starts_with('+')
                    || line.starts_with('-')
                    || line.starts_with(' ')
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Save assistant response to memory
        memory.add_message(MessageRole::Assistant, &sanitised_diff)?;

        println!("📝 LLM generated diff ({} chars)", sanitised_diff.len());

        // Validate that we got a proper diff
        if !CodePatcher::validate_diff(&sanitised_diff) {
            println!("❌ Sanitised diff is not valid unified diff");
            if cli.dry_run {
                println!("📄 Sanitised diff:\n{}", sanitised_diff);
            }
            continue;
        }
        
        // Step 4: Apply the Patch
        println!("🔧 Step 4: Applying code changes...");
        let patch_result = code_patcher.apply_and_verify(&sanitised_diff)?;
        
        if !patch_result.success {
            println!("❌ Failed to apply patch: {:?}", patch_result.error);
            memory.add_message(MessageRole::System, &format!("Patch application failed: {:?}", patch_result.error))?;
            continue;
        }
        
        println!("✅ Patch applied successfully!");
        if !patch_result.files_modified.is_empty() {
            println!("📝 Modified files: {:?}", patch_result.files_modified);
        }
        
        if cli.dry_run {
            println!("🏃 DRY RUN: Skipping test execution");
            println!("✅ Autonomous loop completed successfully (dry run)");
            return Ok(());
        }
        
        // Step 5: Run Tests (unless skipped)
        if cli.skip_tests {
            println!("⏭️  Skipping tests as requested");
            println!("✅ Autonomous loop completed successfully!");
            return Ok(());
        }
        
        println!("🧪 Step 5: Running test suite...");
        let test_result = test_executor.run_tests()?;
        
        if test_result.success {
            println!("🎉 ALL TESTS PASSED!");
            println!("✅ Autonomous TDD loop completed successfully!");
            
            // Save success to memory
            memory.add_message(MessageRole::System, "Tests passed! Task completed successfully.")?;
            
            // Show final summary
            println!("\n📊 Final Summary:");
            println!("  • Request: {}", user_prompt);
            println!("  • Attempts: {}", attempt);
            println!("  • Files modified: {:?}", patch_result.files_modified);
            println!("  • Tests: {} passed, {} failed", test_result.passed, test_result.failed);
            
            return Ok(());
        } else {
            println!("❌ Tests failed! ({} passed, {} failed)", test_result.passed, test_result.failed);
            
            // Save test failure details to memory for next iteration
            let failure_summary = format!(
                "Tests failed: {} passed, {} failed. Errors: {}",
                test_result.passed,
                test_result.failed,
                test_result.failed_tests.iter()
                    .map(|t| format!("{}: {}", t.name, t.error_message))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
            memory.add_message(MessageRole::System, &failure_summary)?;
            
            if attempt == cli.max_retries {
                println!("💥 Maximum retries reached. Autonomous loop failed.");
                return Err("TDD loop failed after maximum retries".into());
            } else {
                println!("🔄 Retrying with test failure context...");
            }
        }
    }
    
    Err("TDD loop failed to complete successfully".into())
}
