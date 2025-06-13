use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs};
use walkdir::WalkDir;

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
pub struct ContextBuilder;

impl ContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a `ProjectContext` by searching the repository for files that match keywords
    /// extracted from the user prompt. The search is performed with the `rg` (ripgrep)
    /// command-line tool, which must be available in the user's `PATH`.
    pub fn build_context_from_prompt(&self, prompt: &str) -> Result<ProjectContext> {
        let keywords = self.extract_keywords(prompt);
        if keywords.is_empty() {
            return Ok(ProjectContext::new());
        }

        let mut files: HashSet<String> = HashSet::new();

        // Scan the workspace recursively. If a keyword contains a dot (.) we treat it as a possible
        // filename; otherwise we look for it inside the file content (best-effort, small files only).
        for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
            let path_str = entry.path().to_string_lossy();

            for kw in &keywords {
                if kw.contains('.') {
                    // Filename match (exact)
                    if let Some(name) = entry.file_name().to_str() {
                        if name == kw {
                            if let Ok(rel) = entry.path().strip_prefix("./") {
                                files.insert(rel.to_string_lossy().to_string());
                            } else {
                                files.insert(path_str.to_string());
                            }
                        }
                    }
                } else {
                    // Simple substring search inside file content (skip large files > 50kB)
                    if let Ok(meta) = entry.metadata() {
                        if meta.len() <= 50_000 {
                            if let Ok(content) = fs::read_to_string(entry.path()) {
                                if content.contains(kw) {
                                    if let Ok(rel) = entry.path().strip_prefix("./") {
                                        files.insert(rel.to_string_lossy().to_string());
                                    } else {
                                        files.insert(path_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Build FileContext objects.
        let mut file_contexts = Vec::new();
        for file_path in files {
            match fs::read_to_string(&file_path) {
                Ok(content) => file_contexts.push(FileContext {
                    path: file_path,
                    line_count: content.lines().count(),
                    char_count: content.chars().count(),
                    content,
                    exists: true,
                }),
                Err(_) => file_contexts.push(FileContext {
                    path: file_path,
                    content: String::new(),
                    line_count: 0,
                    char_count: 0,
                    exists: false,
                }),
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

    /// A very small heuristic keyword extractor: keep tokens that either contain a `.`
    /// (likely a file name) or have uppercase letters (likely a Rust type).
    fn extract_keywords(&self, prompt: &str) -> Vec<String> {
        prompt
            .split_whitespace()
            .filter(|s| s.len() > 3 && (s.contains('.') || s.chars().any(|c| c.is_ascii_uppercase())))
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric() && c != '.').to_string())
            .collect()
    }

    /// Build a context using a quick heuristic: run `build_context_from_prompt` and then
    /// keep at most `max_files` files. This is a pared-down replacement for the more
    /// sophisticated logic that existed previously, but it is sufficient for typical
    /// use-cases and keeps the API stable for callers.
    pub fn build_smart_context(&self, user_request: &str, _project_root: &str, max_files: usize) -> Result<ProjectContext> {
        let mut ctx = self.build_context_from_prompt(user_request)?;
        if ctx.files.len() > max_files {
            ctx.files.truncate(max_files);
        }
        // Recalculate totals after truncation
        ctx.total_line_count = ctx.files.iter().map(|f| f.line_count).sum();
        ctx.total_char_count = ctx.files.iter().map(|f| f.char_count).sum();
        Ok(ctx)
    }

    /// Format a `ProjectContext` for inclusion in an LLM prompt. The format is a simple
    /// plain-text listing that mirrors the previous implementation: it shows each file
    /// path followed by its content, separated with markers.
    pub fn format_for_llm(&self, ctx: &ProjectContext, user_request: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("User Request: {}\n\n", user_request));
        out.push_str(&format!(
            "Context: {} files, {} lines, {} chars\n\n",
            ctx.files.len(), ctx.total_line_count, ctx.total_char_count
        ));

        for file in &ctx.files {
            out.push_str(&format!("=== File: {} ===\n", file.path));
            if file.exists {
                out.push_str(&file.content);
            } else {
                out.push_str("(file missing)\n");
            }
            out.push_str("\n\n");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::{fs::File, io::Write};

    #[test]
    fn test_build_context_from_prompt_selects_correct_files() {
        let tmp = tempdir().unwrap();
        let project_root = tmp.path();

        // Create a small fake project layout.
        std::fs::create_dir_all(project_root.join("src/utils")).unwrap();
        let mut f_main = File::create(project_root.join("src/main.rs")).unwrap();
        f_main.write_all(b"fn main() { let client = api::Client::new(); }").unwrap();

        let mut f_api = File::create(project_root.join("src/utils/api.rs")).unwrap();
        f_api.write_all(b"pub struct Client; impl Client { pub fn new() -> Self { Client } }").unwrap();

        // Change into that dir so ripgrep sees the files.
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(project_root).unwrap();

        let builder = ContextBuilder::new();
        let ctx = builder
            .build_context_from_prompt("update the client struct in api.rs")
            .unwrap();

        std::env::set_current_dir(original_dir).unwrap();

        assert_eq!(ctx.files.len(), 1);
        assert_eq!(ctx.files[0].path, "src/utils/api.rs");
        assert!(ctx.files[0].content.contains("pub struct Client"));
    }
} 