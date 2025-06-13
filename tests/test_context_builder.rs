use project_x::orchestrator::context::{ContextBuilder, FileContext};
use tempfile::tempdir;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn create_mock_project(dir: &Path) {
    fs::create_dir_all(dir.join("src/utils")).unwrap();
    let mut file1 = File::create(dir.join("src/main.rs")).unwrap();
    file1.write_all(b"fn main() { let client = api::Client::new(); }").unwrap();
    let mut file2 = File::create(dir.join("src/utils/api.rs")).unwrap();
    file2.write_all(b"pub struct Client; impl Client { pub fn new() -> Self { Client } }").unwrap();
    let mut file3 = File::create(dir.join("docs/guide.md")).unwrap();
    file3.write_all(b"# Guide\n\nThis is a guide.").unwrap();
}

#[test]
fn test_build_context_from_prompt_surgically_selects_files() {
    let temp_dir = tempdir().unwrap();
    let project_root = temp_dir.path();
    create_mock_project(project_root);

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_root).unwrap();

    let context_builder = ContextBuilder::new();
    let prompt = "add an API key to the client in api.rs";
    let context = context_builder.build_context_from_prompt(prompt).unwrap();

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
    
    assert_eq!(context.files.len(), 1, "Should only find one relevant file");
    assert_eq!(context.files[0].path, "src/utils/api.rs");
    assert!(context.files[0].content.contains("pub struct Client"));
} 