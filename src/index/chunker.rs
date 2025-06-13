use std::error::Error;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChunkType {
    Function,
    Struct,
    Enum,
    Impl,
    Module,
    Const,
    Static,
    Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChunk {
    pub chunk_type: ChunkType,
    pub name: String,
    pub code: String,
    pub start_line: usize,
    pub end_line: usize,
    pub file_path: String,
}

pub struct CodeChunker;

impl CodeChunker {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Simplified regex-based implementation for demonstration
        // TODO: Replace with proper tree-sitter parsing when linking issues are resolved
        Ok(CodeChunker)
    }

    pub fn chunk_file(&self, file_path: &str) -> Result<Vec<CodeChunk>, Box<dyn Error>> {
        let source_code = fs::read_to_string(file_path)?;
        self.chunk_code(&source_code, file_path)
    }

    pub fn chunk_code(&self, source_code: &str, file_path: &str) -> Result<Vec<CodeChunk>, Box<dyn Error>> {
        // Simplified regex-based chunking for demonstration
        // In production, this would use proper tree-sitter parsing
        let mut chunks = Vec::new();
        
        self.extract_functions_regex(source_code, file_path, &mut chunks);
        self.extract_structs_regex(source_code, file_path, &mut chunks);
        self.extract_enums_regex(source_code, file_path, &mut chunks);
        self.extract_impls_regex(source_code, file_path, &mut chunks);
        
        Ok(chunks)
    }

    fn extract_functions_regex(&self, source_code: &str, file_path: &str, chunks: &mut Vec<CodeChunk>) {
        // Simple regex to match function definitions
        let lines: Vec<&str> = source_code.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("fn ") || line.starts_with("pub fn ") || line.starts_with("async fn ") || line.starts_with("pub async fn ") {
                if let Some((name, start_line, end_line, code)) = self.extract_function_block(&lines, i) {
                    chunks.push(CodeChunk {
                        chunk_type: ChunkType::Function,
                        name,
                        code,
                        start_line,
                        end_line,
                        file_path: file_path.to_string(),
                    });
                    i = end_line;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }

    fn extract_structs_regex(&self, source_code: &str, file_path: &str, chunks: &mut Vec<CodeChunk>) {
        let lines: Vec<&str> = source_code.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("struct ") || line.starts_with("pub struct ") {
                if let Some((name, start_line, end_line, code)) = self.extract_block(&lines, i, "struct") {
                    chunks.push(CodeChunk {
                        chunk_type: ChunkType::Struct,
                        name,
                        code,
                        start_line,
                        end_line,
                        file_path: file_path.to_string(),
                    });
                    i = end_line;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }

    fn extract_enums_regex(&self, source_code: &str, file_path: &str, chunks: &mut Vec<CodeChunk>) {
        let lines: Vec<&str> = source_code.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("enum ") || line.starts_with("pub enum ") {
                if let Some((name, start_line, end_line, code)) = self.extract_block(&lines, i, "enum") {
                    chunks.push(CodeChunk {
                        chunk_type: ChunkType::Enum,
                        name,
                        code,
                        start_line,
                        end_line,
                        file_path: file_path.to_string(),
                    });
                    i = end_line;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }

    fn extract_impls_regex(&self, source_code: &str, file_path: &str, chunks: &mut Vec<CodeChunk>) {
        let lines: Vec<&str> = source_code.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("impl ") {
                if let Some((name, start_line, end_line, code)) = self.extract_block(&lines, i, "impl") {
                    chunks.push(CodeChunk {
                        chunk_type: ChunkType::Impl,
                        name,
                        code,
                        start_line,
                        end_line,
                        file_path: file_path.to_string(),
                    });
                    i = end_line;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }

    fn extract_function_block(&self, lines: &[&str], start_idx: usize) -> Option<(String, usize, usize, String)> {
        let start_line = start_idx + 1;
        let first_line = lines[start_idx].trim();
        
        // Extract function name
        let name = if let Some(fn_pos) = first_line.find("fn ") {
            let after_fn = &first_line[fn_pos + 3..];
            if let Some(paren_pos) = after_fn.find('(') {
                after_fn[..paren_pos].trim().to_string()
            } else {
                "unnamed".to_string()
            }
        } else {
            "unnamed".to_string()
        };

        // Find the end of the function
        let mut brace_count = 0;
        let mut found_opening_brace = false;
        let end_idx;
        
        for (i, line) in lines.iter().enumerate().skip(start_idx) {
            for ch in line.chars() {
                match ch {
                    '{' => {
                        brace_count += 1;
                        found_opening_brace = true;
                    }
                    '}' => {
                        brace_count -= 1;
                        if found_opening_brace && brace_count == 0 {
                            end_idx = i;
                            let code = lines[start_idx..=end_idx].join("\n");
                            return Some((name, start_line, end_idx + 1, code));
                        }
                    }
                    _ => {}
                }
            }
        }
        
        None
    }

    fn extract_block(&self, lines: &[&str], start_idx: usize, block_type: &str) -> Option<(String, usize, usize, String)> {
        let start_line = start_idx + 1;
        let first_line = lines[start_idx].trim();
        
        // Extract name
        let name = if let Some(type_pos) = first_line.find(&format!("{} ", block_type)) {
            let after_type = &first_line[type_pos + block_type.len() + 1..];
            if let Some(space_or_brace) = after_type.find(|c: char| c == ' ' || c == '{' || c == '<') {
                after_type[..space_or_brace].trim().to_string()
            } else {
                after_type.trim().to_string()
            }
        } else {
            "unnamed".to_string()
        };

        // Find the end of the block
        let mut brace_count = 0;
        let mut found_opening_brace = false;
        let end_idx;
        
        for (i, line) in lines.iter().enumerate().skip(start_idx) {
            for ch in line.chars() {
                match ch {
                    '{' => {
                        brace_count += 1;
                        found_opening_brace = true;
                    }
                    '}' => {
                        brace_count -= 1;
                        if found_opening_brace && brace_count == 0 {
                            end_idx = i;
                            let code = lines[start_idx..=end_idx].join("\n");
                            return Some((name, start_line, end_idx + 1, code));
                        }
                    }
                    _ => {}
                }
            }
        }
        
        None
    }
}