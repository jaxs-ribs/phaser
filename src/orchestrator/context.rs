use std::error::Error;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use grep::regex::RegexMatcher;
use grep::searcher::{Searcher, Sink, SinkMatch};
use std::collections::HashSet;
use ignore::WalkBuilder;
use std::io::Cursor;
use std::process::Command;

use super::{FileContext, ProjectContext};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectContext {
    pub files: Vec<FileContext>,
    pub total_line_count: usize,
    pub total_char_count: usize,
}

impl ProjectContext {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    pub path: String,
    pub content: String,
    pub line_count: usize,
    pub char_count: usize,
    pub exists: bool,
}

#[derive(Debug, Default)]
pub struct ContextBuilder {}

impl ContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_context_from_prompt(&self, prompt: &str) -> Result<ProjectContext> {
        let keywords = self.extract_keywords(prompt);
        if keywords.is_empty() {
            return Ok(ProjectContext::new());
        }

        let pattern = keywords.join("|");
        let output = Command::new("rg")
            .arg("--files-with-matches")
            .arg("--iglob")
            .arg("!.git")
            .arg("-e")
            .arg(&pattern)
            .arg(".")
            .output()?;

        if !output.status.success() {
            if output.status.code() != Some(1) {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("ripgrep failed: {}", stderr));
            }
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: HashSet<String> = stdout.lines().map(String::from).collect();

        let mut file_contexts = Vec::new();
        for file_path in files {
            if let Ok(content) = fs::read_to_string(&file_path) {
                file_contexts.push(FileContext {
                    path: file_path,
                    line_count: content.lines().count(),
                    char_count: content.chars().count(),
                    content,
                    exists: true,
                });
            }
        }

        let total_line_count = file_contexts.iter().map(|f| f.line_count).sum();
        let total_char_count = file_contexts.iter().map(|f| f.char_count).sum();

        Ok(ProjectContext {
            files: file_contexts,
            total_line_count,
            total_char_count,
        })
    }

    fn extract_keywords(&self, prompt: &str) -> Vec<String> {
        prompt
            .split_whitespace()
            .filter(|s| s.len() > 3 && (s.contains('.') || s.chars().any(|c| c.is_ascii_uppercase())))
            .map(|s| {
                s.trim_matches(|p: char| !p.is_alphanumeric() && p != '.')
                    .to_string()
            })
            .collect()
    }

    /// Build context from a list of file paths
    pub fn build_context(&self, file_paths: &[&str]) -> Result<BuildContext, Box<dyn Error>> {
        println!("🔍 Building context from {} files...", file_paths.len());
        
        let mut files = Vec::new();
        let mut total_lines = 0;
        let mut total_chars = 0;
        let mut successful_files = 0;
        
        for &file_path in file_paths {
            match self.read_file_context(file_path) {
                Ok(file_context) => {
                    total_lines += file_context.line_count;
                    total_chars += file_context.char_count;
                    if file_context.exists {
                        successful_files += 1;
                    }
                    files.push(file_context);
                    println!("  ✅ {}: {} lines", file_path, files.last().unwrap().line_count);
                }
                Err(e) => {
                    println!("  ❌ {}: {}", file_path, e);
                    // Add a failed file entry
                    files.push(FileContext {
                        path: file_path.to_string(),
                        content: format!("Error reading file: {}", e),
                        line_count: 0,
                        char_count: 0,
                        exists: false,
                    });
                }
            }
        }
        
        let context_summary = format!(
            "Context includes {} files ({} successful) with {} total lines and {} characters",
            file_paths.len(), successful_files, total_lines, total_chars
        );
        
        println!("📊 {}", context_summary);
        
        Ok(BuildContext {
            files,
            total_lines,
            total_chars,
            total_files: file_paths.len(),
            context_summary,
        })
    }
    
    /// Read a single file and create FileContext
    fn read_file_context(&self, file_path: &str) -> Result<FileContext, Box<dyn Error>> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Ok(FileContext {
                path: file_path.to_string(),
                content: format!("File does not exist: {}", file_path),
                line_count: 0,
                char_count: 0,
                exists: false,
            });
        }
        
        let metadata = fs::metadata(path)?;
        if metadata.len() > 50_000 as u64 {
            return Ok(FileContext {
                path: file_path.to_string(),
                content: format!("File too large ({} bytes, max 50KB)", metadata.len()),
                line_count: 0,
                char_count: 0,
                exists: true,
            });
        }
        
        let content = fs::read_to_string(path)?;
        let line_count = content.lines().count();
        let char_count = content.len();
        
        Ok(FileContext {
            path: file_path.to_string(),
            content,
            line_count,
            char_count,
            exists: true,
        })
    }
    
    /// Format the context for LLM consumption
    pub fn format_for_llm(&self, context: &BuildContext, user_request: &str) -> String {
        let mut formatted = String::new();
        
        formatted.push_str(&format!("User Request: {}\n\n", user_request));
        formatted.push_str(&format!("Context Summary: {}\n\n", context.context_summary));
        
        for file in &context.files {
            formatted.push_str(&format!("=== File: {} ===\n", file.path));
            if file.exists {
                formatted.push_str(&file.content);
            } else {
                formatted.push_str(&format!("ERROR: {}", file.content));
            }
            formatted.push_str("\n\n");
        }
        
        formatted
    }
    
    /// Suggest relevant files based on the user request
    pub fn suggest_relevant_files(&self, user_request: &str, project_root: &str) -> Vec<String> {
        let mut suggested_files = Vec::new();
        let request_lower = user_request.to_lowercase();
        
        // Core files that are often relevant
        let core_files = vec![
            "src/main.rs",
            "src/lib.rs",
            "Cargo.toml",
        ];
        
        for file in core_files {
            let full_path = format!("{}/{}", project_root, file);
            if Path::new(&full_path).exists() {
                suggested_files.push(file.to_string());
            }
        }
        
        // Suggest files based on keywords in the request
        if request_lower.contains("test") {
            self.find_and_add_files(&mut suggested_files, project_root, "tests", ".rs");
        }
        
        if request_lower.contains("voice") || request_lower.contains("audio") {
            self.find_and_add_files(&mut suggested_files, project_root, "src/voice", ".rs");
        }
        
        if request_lower.contains("llm") || request_lower.contains("gemini") || request_lower.contains("ai") {
            self.find_and_add_files(&mut suggested_files, project_root, "src/llm", ".rs");
        }
        
        if request_lower.contains("index") || request_lower.contains("search") || request_lower.contains("rag") {
            self.find_and_add_files(&mut suggested_files, project_root, "src/index", ".rs");
        }
        
        if request_lower.contains("memory") || request_lower.contains("conversation") {
            self.find_and_add_files(&mut suggested_files, project_root, "src/orchestrator", ".rs");
        }
        
        if request_lower.contains("git") || request_lower.contains("commit") || request_lower.contains("patch") {
            self.find_and_add_files(&mut suggested_files, project_root, "src/utils", ".rs");
            self.find_and_add_files(&mut suggested_files, project_root, "src/edit", ".rs");
        }
        
        // Remove duplicates
        suggested_files.sort();
        suggested_files.dedup();
        
        suggested_files
    }
    
    /// Helper to find and add files matching a pattern
    fn find_and_add_files(&self, file_list: &mut Vec<String>, root: &str, subdir: &str, extension: &str) {
        let search_path = format!("{}/{}", root, subdir);
        if let Ok(entries) = fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let file_name = entry.file_name();
                        let file_name_str = file_name.to_string_lossy();
                        if file_name_str.ends_with(extension) {
                            let relative_path = format!("{}/{}", subdir, file_name_str);
                            file_list.push(relative_path);
                        }
                    }
                }
            }
        }
    }
    
    /// Build context with smart file selection based on user request
    pub fn build_smart_context(&self, user_request: &str, project_root: &str, max_files: usize) -> Result<BuildContext, Box<dyn Error>> {
        let suggested_files = self.suggest_relevant_files(user_request, project_root);
        
        println!("🧠 Smart context suggestion for request: \"{}\"", user_request);
        println!("📂 Suggested files: {:?}", suggested_files);
        
        // Limit to max_files
        let files_to_include: Vec<&str> = suggested_files
            .iter()
            .take(max_files)
            .map(|s| s.as_str())
            .collect();
        
        self.build_context(&files_to_include)
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct FilePathSink<'a> {
    files: &'a mut HashSet<String>,
}

impl<'a> Sink for FilePathSink<'a> {
    type Error = std::io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch) -> Result<bool, Self::Error> {
        let path_str = std::str::from_utf8(mat.path()).unwrap_or_default().to_string();
        if !path_str.is_empty() {
            self.files.insert(path_str);
        }
        Ok(true) // Continue searching
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_build_context_with_existing_file() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {\n    println!(\"Hello\");\n}").unwrap();
        
        let builder = ContextBuilder::new();
        let file_path = test_file.to_str().unwrap();
        let context = builder.build_context(&[file_path]).unwrap();
        
        assert_eq!(context.files.len(), 1);
        assert!(context.files[0].exists);
        assert_eq!(context.files[0].line_count, 3);
        assert!(context.files[0].content.contains("fn main()"));
    }
    
    #[test]
    fn test_build_context_with_nonexistent_file() {
        let builder = ContextBuilder::new();
        let context = builder.build_context(&["nonexistent.rs"]).unwrap();
        
        assert_eq!(context.files.len(), 1);
        assert!(!context.files[0].exists);
        assert!(context.files[0].content.contains("does not exist"));
    }
    
    #[test]
    fn test_suggest_relevant_files() {
        let builder = ContextBuilder::new();
        
        let test_files = builder.suggest_relevant_files("add voice recording feature", ".");
        assert!(test_files.iter().any(|f| f.contains("voice")));
        
        let llm_files = builder.suggest_relevant_files("fix gemini api issue", ".");
        assert!(llm_files.iter().any(|f| f.contains("llm")));
    }
    
    #[test]
    fn test_format_for_llm() {
        let builder = ContextBuilder::new();
        let file_context = FileContext {
            path: "test.rs".to_string(),
            content: "fn test() {}".to_string(),
            line_count: 1,
            char_count: 12,
            exists: true,
        };
        
        let context = BuildContext {
            files: vec![file_context],
            total_lines: 1,
            total_chars: 12,
            total_files: 1,
            context_summary: "Test context".to_string(),
        };
        
        let formatted = builder.format_for_llm(&context, "test request");
        assert!(formatted.contains("User Request: test request"));
        assert!(formatted.contains("=== File: test.rs ==="));
        assert!(formatted.contains("fn test() {}"));
    }
}