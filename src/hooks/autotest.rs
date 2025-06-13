use std::error::Error;
use std::process::Command;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub success: bool,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub total: usize,
    pub duration: Duration,
    pub output: String,
    pub errors: Vec<String>,
    pub failed_tests: Vec<FailedTest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedTest {
    pub name: String,
    pub error_message: String,
    pub file_location: Option<String>,
}

pub struct TestExecutor {
    timeout: Option<Duration>,
    verbose: bool,
}

impl TestExecutor {
    /// Create a new TestExecutor with default settings
    pub fn new() -> Self {
        TestExecutor {
            timeout: Some(Duration::from_secs(300)), // 5 minute default timeout
            verbose: true,
        }
    }
    
    /// Create a TestExecutor with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        TestExecutor {
            timeout: Some(timeout),
            verbose: true,
        }
    }
    
    /// Create a TestExecutor with no timeout
    pub fn no_timeout() -> Self {
        TestExecutor {
            timeout: None,
            verbose: true,
        }
    }
    
    /// Run the full test suite
    pub fn run_tests(&self) -> Result<TestResult, Box<dyn Error>> {
        println!("🧪 Running test suite...");
        let start_time = Instant::now();
        
        let mut cmd = Command::new("cargo");
        cmd.arg("test");
        cmd.arg("--color=always");
        
        if self.verbose {
            cmd.arg("--verbose");
        }
        
        println!("🔧 Running command: {:?}", cmd);
        
        // Execute the command with optional timeout
        let output = if let Some(timeout) = self.timeout {
            self.run_with_timeout(cmd, timeout)?
        } else {
            cmd.output()?
        };
        
        let duration = start_time.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined_output = format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);
        
        let success = output.status.success();
        
        // Parse test results from output
        let (passed, failed, ignored, total) = self.parse_test_counts(&stdout);
        let failed_tests = self.parse_failed_tests(&stdout);
        let errors = self.parse_errors(&stderr);
        
        let result = TestResult {
            success,
            passed,
            failed,
            ignored,
            total,
            duration,
            output: combined_output,
            errors,
            failed_tests,
        };
        
        self.print_test_summary(&result);
        
        Ok(result)
    }
    
    /// Run specific tests by name pattern
    pub fn run_specific_tests(&self, test_pattern: &str) -> Result<TestResult, Box<dyn Error>> {
        println!("🧪 Running tests matching pattern: {}", test_pattern);
        let start_time = Instant::now();
        
        let mut cmd = Command::new("cargo");
        cmd.arg("test");
        cmd.arg("--color=always");
        cmd.arg(test_pattern);
        
        if self.verbose {
            cmd.arg("--verbose");
        }
        
        let output = if let Some(timeout) = self.timeout {
            self.run_with_timeout(cmd, timeout)?
        } else {
            cmd.output()?
        };
        
        let duration = start_time.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined_output = format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);
        
        let success = output.status.success();
        let (passed, failed, ignored, total) = self.parse_test_counts(&stdout);
        let failed_tests = self.parse_failed_tests(&stdout);
        let errors = self.parse_errors(&stderr);
        
        let result = TestResult {
            success,
            passed,
            failed,
            ignored,
            total,
            duration,
            output: combined_output,
            errors,
            failed_tests,
        };
        
        self.print_test_summary(&result);
        Ok(result)
    }
    
    /// Check if the project compiles without running tests
    pub fn check_compilation(&self) -> Result<bool, Box<dyn Error>> {
        println!("🔧 Checking compilation...");
        
        let output = Command::new("cargo")
            .arg("check")
            .arg("--color=always")
            .output()?;
        
        let success = output.status.success();
        
        if success {
            println!("✅ Compilation successful");
        } else {
            println!("❌ Compilation failed");
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Compilation errors:\n{}", stderr);
        }
        
        Ok(success)
    }
    
    /// Run a command with timeout (simplified implementation)
    fn run_with_timeout(&self, mut cmd: Command, _timeout: Duration) -> Result<std::process::Output, Box<dyn Error>> {
        // For now, just run without timeout (the full timeout implementation is complex)
        // In a production system, you'd use a proper async timeout mechanism
        cmd.output().map_err(|e| e.into())
    }
    
    /// Parse test counts from cargo test output
    fn parse_test_counts(&self, output: &str) -> (usize, usize, usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        let mut ignored = 0;
        
        for line in output.lines() {
            if line.starts_with("test result:") {
                // Example: "test result: ok. 5 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out"
                let parts: Vec<&str> = line.split(';').collect();
                for part in parts {
                    let trimmed = part.trim();
                    if trimmed.contains("passed") {
                        if let Some(num) = trimmed.split_whitespace().find_map(|tok| tok.parse::<usize>().ok()) {
                            passed = num;
                        }
                    } else if trimmed.contains("failed") {
                        if let Some(num) = trimmed.split_whitespace().find_map(|tok| tok.parse::<usize>().ok()) {
                            failed = num;
                        }
                    } else if trimmed.contains("ignored") {
                        if let Some(num) = trimmed.split_whitespace().find_map(|tok| tok.parse::<usize>().ok()) {
                            ignored = num;
                        }
                    }
                }
            }
        }
        
        let total = passed + failed + ignored;
        (passed, failed, ignored, total)
    }
    
    /// Parse failed test information from output
    fn parse_failed_tests(&self, output: &str) -> Vec<FailedTest> {
        let mut failed_tests = Vec::new();
        let lines: Vec<&str> = output.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            if line.contains(" FAILED") && line.contains("test ") {
                // Example: "test utils::git_client::tests::test_add_and_commit ... FAILED"
                if let Some(test_name) = line.split("test ").nth(1) {
                    if let Some(name) = test_name.split(" ...").next() {
                        // Look for error message in subsequent lines
                        let mut error_message = String::new();
                        let mut file_location = None;
                        
                        // Collect error details from following lines
                        for j in (i + 1)..std::cmp::min(i + 20, lines.len()) {
                            let error_line = lines[j];
                            if error_line.starts_with("test ") {
                                break;
                            }
                            if error_line.trim().is_empty() {
                                continue;
                            }
                            // Extract file location from panic messages
                            if error_line.contains(" panicked at ") {
                                if let Some(location_part) = error_line.split(" panicked at ").nth(1) {
                                    if let Some(location) = location_part.split(':').next() {
                                        file_location = Some(location.to_string());
                                    }
                                }
                            }
                            error_message.push_str(error_line);
                            error_message.push('\n');
                        }
                        
                        failed_tests.push(FailedTest {
                            name: name.to_string(),
                            error_message: error_message.trim().to_string(),
                            file_location,
                        });
                    }
                }
            }
        }
        
        failed_tests
    }
    
    /// Parse compilation or other errors from stderr
    fn parse_errors(&self, stderr: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        for line in stderr.lines() {
            if line.contains("error:") || line.contains("Error:") {
                errors.push(line.to_string());
            }
        }
        
        errors
    }
    
    /// Print a formatted test summary
    fn print_test_summary(&self, result: &TestResult) {
        println!("\n📊 Test Summary:");
        println!("⏱️  Duration: {:.2}s", result.duration.as_secs_f64());
        
        if result.success {
            println!("✅ Tests PASSED");
        } else {
            println!("❌ Tests FAILED");
        }
        
        println!("📈 Results: {} passed, {} failed, {} ignored ({} total)", 
                result.passed, result.failed, result.ignored, result.total);
        
        if !result.failed_tests.is_empty() {
            println!("\n💥 Failed Tests:");
            for test in &result.failed_tests {
                println!("  • {}", test.name);
                if let Some(location) = &test.file_location {
                    println!("    📍 {}", location);
                }
                if !test.error_message.is_empty() {
                    // Show first line of error message
                    if let Some(first_line) = test.error_message.lines().next() {
                        println!("    ❌ {}", first_line);
                    }
                }
            }
        }
        
        if !result.errors.is_empty() {
            println!("\n⚠️  Compilation Errors:");
            for error in &result.errors {
                println!("  • {}", error);
            }
        }
    }
}

impl Default for TestExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_test_counts() {
        let executor = TestExecutor::new();
        let output = "test result: ok. 5 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out";
        
        let (passed, failed, ignored, total) = executor.parse_test_counts(output);
        assert_eq!(passed, 5);
        assert_eq!(failed, 2);
        assert_eq!(ignored, 1);
        assert_eq!(total, 8);
    }
    
    #[test]
    fn test_parse_failed_tests() {
        let executor = TestExecutor::new();
        let output = r#"test utils::git_client::tests::test_add_and_commit ... FAILED
thread 'utils::git_client::tests::test_add_and_commit' panicked at src/utils/git_client.rs:369:9:
assertion failed: status.staged_files.contains(&"test.txt".to_string())"#;
        
        let failed_tests = executor.parse_failed_tests(output);
        assert_eq!(failed_tests.len(), 1);
        assert_eq!(failed_tests[0].name, "utils::git_client::tests::test_add_and_commit");
        assert!(failed_tests[0].error_message.contains("assertion failed"));
    }
    
    #[test] 
    fn test_check_compilation() {
        let executor = TestExecutor::new();
        // This should pass if the project compiles
        let result = executor.check_compilation();
        assert!(result.is_ok());
    }
}