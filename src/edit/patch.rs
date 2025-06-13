use std::error::Error;
use std::process::Command;
use std::path::Path;
use tempfile::NamedTempFile;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct PatchResult {
    pub success: bool,
    pub files_modified: Vec<String>,
    pub output: String,
    pub error: Option<String>,
}

pub struct CodePatcher {
    dry_run: bool,
    backup: bool,
}

impl CodePatcher {
    /// Create a new CodePatcher
    pub fn new() -> Self {
        CodePatcher {
            dry_run: false,
            backup: true,
        }
    }
    
    /// Create a CodePatcher with custom options
    pub fn with_options(dry_run: bool, backup: bool) -> Self {
        CodePatcher { dry_run, backup }
    }
    
    /// Apply a unified diff string to the filesystem
    pub fn apply_patch(&self, diff_content: &str) -> Result<PatchResult, Box<dyn Error>> {
        if diff_content.trim().is_empty() {
            return Ok(PatchResult {
                success: true,
                files_modified: vec![],
                output: "No changes to apply".to_string(),
                error: None,
            });
        }
        
        println!("🔧 Applying code patch...");
        if self.dry_run {
            println!("🏃 DRY RUN MODE - No actual changes will be made");
        }
        
        // Create temporary patch file
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(diff_content.as_bytes())?;
        let temp_path = temp_file.path();
        
        // Build patch command
        let mut cmd = Command::new("patch");
        cmd.arg("-p1"); // Remove one level of path prefix
        cmd.arg("--unified"); // Expect unified diff format
        cmd.arg("--input").arg(temp_path);
        
        if self.dry_run {
            cmd.arg("--dry-run");
        }
        
        if self.backup {
            cmd.arg("--backup");
        }
        
        // Add verbose output
        cmd.arg("--verbose");
        
        println!("🔧 Running patch command: {:?}", cmd);
        
        // Execute patch command
        let output = cmd.output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined_output = format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);
        
        let success = output.status.success();
        
        // Parse modified files from patch output
        let files_modified = self.parse_modified_files(&stdout);
        
        let result = PatchResult {
            success,
            files_modified: files_modified.clone(),
            output: combined_output.clone(),
            error: if success { None } else { Some(stderr.to_string()) },
        };
        
        if success {
            println!("✅ Patch applied successfully!");
            if !files_modified.is_empty() {
                println!("📝 Modified files:");
                for file in &files_modified {
                    println!("  • {}", file);
                }
            }
        } else {
            println!("❌ Patch failed!");
            println!("Error output: {}", stderr);
        }
        
        if self.dry_run {
            println!("🏃 DRY RUN - No actual changes were made");
        }
        
        Ok(result)
    }
    
    /// Parse the list of modified files from patch output
    fn parse_modified_files(&self, patch_output: &str) -> Vec<String> {
        let mut files = Vec::new();
        
        for line in patch_output.lines() {
            // Look for lines like "patching file src/main.rs"
            if line.starts_with("patching file ") {
                if let Some(file_path) = line.strip_prefix("patching file ") {
                    files.push(file_path.to_string());
                }
            }
        }
        
        files
    }
    
    /// Apply a patch and verify the result by checking if files exist
    pub fn apply_and_verify(&self, diff_content: &str) -> Result<PatchResult, Box<dyn Error>> {
        let result = self.apply_patch(diff_content)?;
        
        if result.success && !self.dry_run {
            // Verify that modified files actually exist
            let mut verified_files = Vec::new();
            for file_path in &result.files_modified {
                if Path::new(file_path).exists() {
                    verified_files.push(file_path.clone());
                    println!("✅ Verified: {}", file_path);
                } else {
                    println!("⚠️  Warning: {} was reported as modified but doesn't exist", file_path);
                }
            }
            
            Ok(PatchResult {
                success: result.success,
                files_modified: verified_files,
                output: result.output,
                error: result.error,
            })
        } else {
            Ok(result)
        }
    }
    
    /// Create a patch from the differences between two file contents
    pub fn create_patch(old_content: &str, new_content: &str, file_path: &str) -> Result<String, Box<dyn Error>> {
        // Create temporary files for diff
        let mut old_file = NamedTempFile::new()?;
        let mut new_file = NamedTempFile::new()?;
        
        old_file.write_all(old_content.as_bytes())?;
        new_file.write_all(new_content.as_bytes())?;
        
        // Run diff command to generate unified diff
        let output = Command::new("diff")
            .arg("-u")
            .arg("--label").arg(format!("a/{}", file_path))
            .arg("--label").arg(format!("b/{}", file_path))
            .arg(old_file.path())
            .arg(new_file.path())
            .output()?;
        
        let diff_output = String::from_utf8_lossy(&output.stdout);
        
        // diff returns exit code 1 when files differ, which is expected
        if output.status.code() == Some(0) {
            // Files are identical
            Ok(String::new())
        } else if output.status.code() == Some(1) {
            // Files differ - this is what we want
            Ok(diff_output.to_string())
        } else {
            // Actual error
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("diff command failed: {}", stderr).into())
        }
    }
    
    /// Test if a patch would apply cleanly without making changes
    pub fn test_patch(&self, diff_content: &str) -> Result<bool, Box<dyn Error>> {
        let dry_run_patcher = CodePatcher::with_options(true, false);
        let result = dry_run_patcher.apply_patch(diff_content)?;
        Ok(result.success)
    }
    
    /// Validate that a diff string looks like a proper unified diff
    pub fn validate_diff(diff_content: &str) -> bool {
        let lines: Vec<&str> = diff_content.lines().collect();
        
        if lines.is_empty() {
            return true; // Empty diff is valid (no changes)
        }
        
        // Look for unified diff headers
        let has_diff_header = lines.iter().any(|line| line.starts_with("--- ") || line.starts_with("+++ "));
        let has_hunk_header = lines.iter().any(|line| line.starts_with("@@"));
        
        // Basic validation - should have file headers and hunk headers
        has_diff_header && has_hunk_header
    }
}

impl Default for CodePatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
    #[test]
    fn test_validate_diff() {
        let valid_diff = r#"--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-old line
+new line
 line 3"#;
        
        assert!(CodePatcher::validate_diff(valid_diff));
        assert!(CodePatcher::validate_diff("")); // Empty diff is valid
        assert!(!CodePatcher::validate_diff("not a diff"));
    }
    
    #[test]
    fn test_create_patch() {
        let old_content = "line 1\nold line\nline 3\n";
        let new_content = "line 1\nnew line\nline 3\n";
        
        let patch = CodePatcher::create_patch(old_content, new_content, "test.txt").unwrap();
        
        assert!(!patch.is_empty());
        assert!(patch.contains("--- a/test.txt"));
        assert!(patch.contains("+++ b/test.txt"));
        assert!(patch.contains("-old line"));
        assert!(patch.contains("+new line"));
    }
    
    #[test]
    fn test_parse_modified_files() {
        let patcher = CodePatcher::new();
        let patch_output = r#"patching file src/main.rs
patching file src/lib.rs
Hunk #1 succeeded at 10 (offset 2 lines)."#;
        
        let files = patcher.parse_modified_files(patch_output);
        assert_eq!(files, vec!["src/main.rs", "src/lib.rs"]);
    }
    
    #[test]
    fn test_dry_run_patch() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "original content\n").unwrap();
        
        // Change to temp directory for the test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let diff = r#"--- a/test.txt
+++ b/test.txt
@@ -1 +1 @@
-original content
+modified content"#;
        
        let patcher = CodePatcher::with_options(true, false); // dry run
        let result = patcher.apply_patch(diff).unwrap();
        
        // File should not actually be modified in dry run
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "original content\n");
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}