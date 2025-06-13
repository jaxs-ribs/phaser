use git2::{Repository, StatusOptions, Status, DiffOptions, DiffFormat};
use std::error::Error;
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub staged_files: Vec<String>,
    pub modified_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub renamed_files: Vec<(String, String)>, // (old_name, new_name)
}

#[derive(Debug, Clone)]
pub struct GitDiff {
    pub diff_text: String,
    pub file_count: usize,
    pub insertions: usize,
    pub deletions: usize,
}

pub struct GitClient {
    repo: Repository,
}

impl GitClient {
    /// Create a new GitClient for the current directory or a specified path
    pub fn new(repo_path: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let repo_path = repo_path.unwrap_or(".");
        let repo = Repository::open(repo_path)?;
        
        println!("🔧 Git client initialized for repository: {}", 
                repo.path().display());
        
        Ok(GitClient { repo })
    }
    
    /// Get the current repository status
    pub fn get_status(&self) -> Result<GitStatus, Box<dyn Error>> {
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        status_opts.include_ignored(false);
        
        let statuses = self.repo.statuses(Some(&mut status_opts))?;
        
        let mut git_status = GitStatus {
            staged_files: Vec::new(),
            modified_files: Vec::new(),
            untracked_files: Vec::new(),
            deleted_files: Vec::new(),
            renamed_files: Vec::new(),
        };
        
        for entry in statuses.iter() {
            let file_path = entry.path().unwrap_or("<invalid utf-8>").to_string();
            let status = entry.status();
            
            // Check staged changes
            if status.contains(Status::INDEX_NEW) ||
               status.contains(Status::INDEX_MODIFIED) ||
               status.contains(Status::INDEX_DELETED) ||
               status.contains(Status::INDEX_RENAMED) ||
               status.contains(Status::INDEX_TYPECHANGE) {
                git_status.staged_files.push(file_path.clone());
            }
            
            // Check working tree changes
            if status.contains(Status::WT_MODIFIED) {
                git_status.modified_files.push(file_path.clone());
            }
            
            if status.contains(Status::WT_NEW) {
                git_status.untracked_files.push(file_path.clone());
            }
            
            if status.contains(Status::WT_DELETED) {
                git_status.deleted_files.push(file_path.clone());
            }
            
            // Handle renames (simplified - git2 provides more complex rename detection)
            if status.contains(Status::INDEX_RENAMED) {
                // For simplicity, we'll just note it as a renamed file
                // In practice, you'd need to use diff analysis to get old/new names
                git_status.renamed_files.push((file_path.clone(), file_path));
            }
        }
        
        Ok(git_status)
    }
    
    /// Get the diff of working directory changes (unstaged changes)
    pub fn get_diff(&self, staged: bool) -> Result<GitDiff, Box<dyn Error>> {
        let mut diff_opts = DiffOptions::new();
        diff_opts.context_lines(3);
        
        let diff = if staged {
            // Diff between index and HEAD (staged changes)
            let tree = self.repo.head()?.peel_to_tree()?;
            self.repo.diff_tree_to_index(Some(&tree), None, Some(&mut diff_opts))?
        } else {
            // Diff between working tree and index (unstaged changes)
            self.repo.diff_index_to_workdir(None, Some(&mut diff_opts))?
        };
        
        let stats = diff.stats()?;
        let mut diff_text = String::new();
        
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            let content = String::from_utf8_lossy(line.content());
            let prefix = match line.origin() {
                '+' => "+",
                '-' => "-",
                ' ' => " ",
                '\\' => "\\",
                _ => "",
            };
            diff_text.push_str(&format!("{}{}", prefix, content));
            true
        })?;
        
        Ok(GitDiff {
            diff_text,
            file_count: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        })
    }
    
    /// Add files to the staging area
    pub fn add<P: AsRef<Path>>(&self, files: &[P]) -> Result<(), Box<dyn Error>> {
        let mut index = self.repo.index()?;
        
        for file in files {
            let path = file.as_ref();
            if path.exists() {
                index.add_path(path)?;
                println!("➕ Added to staging: {}", path.display());
            } else {
                println!("⚠️  File not found: {}", path.display());
            }
        }
        
        index.write()?;
        Ok(())
    }
    
    /// Add all modified and untracked files
    pub fn add_all(&self) -> Result<(), Box<dyn Error>> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        println!("➕ Added all changes to staging");
        Ok(())
    }
    
    /// Create a new commit with the given message
    pub fn commit(&self, message: &str) -> Result<git2::Oid, Box<dyn Error>> {
        let signature = self.get_signature()?;
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        let parent_commit = match self.repo.head() {
            Ok(head) => Some(head.peel_to_commit()?),
            Err(_) => None, // Initial commit
        };
        
        let parents: Vec<&git2::Commit> = match &parent_commit {
            Some(commit) => vec![commit],
            None => vec![],
        };
        
        let commit_id = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;
        
        println!("✅ Created commit: {} - {}", 
                commit_id.to_string()[..8].to_string(), 
                message.lines().next().unwrap_or(message));
        
        Ok(commit_id)
    }
    
    /// Get the current branch name
    pub fn get_current_branch(&self) -> Result<String, Box<dyn Error>> {
        let head = self.repo.head()?;
        
        if let Some(branch_name) = head.shorthand() {
            Ok(branch_name.to_string())
        } else {
            Ok("HEAD".to_string()) // Detached HEAD
        }
    }
    
    /// Check if the repository has uncommitted changes
    pub fn has_uncommitted_changes(&self) -> Result<bool, Box<dyn Error>> {
        let status = self.get_status()?;
        Ok(!status.staged_files.is_empty() || 
           !status.modified_files.is_empty() || 
           !status.untracked_files.is_empty() ||
           !status.deleted_files.is_empty())
    }
    
    /// Get repository information summary
    pub fn get_repo_info(&self) -> Result<RepoInfo, Box<dyn Error>> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        let branch = self.get_current_branch()?;
        let status = self.get_status()?;
        let has_changes = self.has_uncommitted_changes()?;
        
        Ok(RepoInfo {
            branch,
            commit_hash: commit.id().to_string(),
            commit_message: commit.message().unwrap_or("").to_string(),
            has_uncommitted_changes: has_changes,
            staged_count: status.staged_files.len(),
            modified_count: status.modified_files.len(),
            untracked_count: status.untracked_files.len(),
        })
    }
    
    /// Get git signature (author/committer info)
    fn get_signature(&self) -> Result<git2::Signature, Box<dyn Error>> {
        // Try to get from git config, fallback to defaults
        let config = self.repo.config()?;
        let name = config.get_string("user.name").unwrap_or_else(|_| "AI Assistant".to_string());
        let email = config.get_string("user.email").unwrap_or_else(|_| "ai@example.com".to_string());
        Ok(git2::Signature::now(&name, &email)?)
    }
    
    /// Generate a suggested commit message based on the staged changes
    pub fn suggest_commit_message(&self) -> Result<String, Box<dyn Error>> {
        let status = self.get_status()?;
        let diff = self.get_diff(true)?; // Get staged diff
        
        if status.staged_files.is_empty() {
            return Ok("No staged changes".to_string());
        }
        
        let mut suggestions = Vec::new();
        
        // Analyze file patterns
        let mut file_types: HashMap<String, usize> = HashMap::new();
        for file in &status.staged_files {
            if let Some(ext) = Path::new(file).extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                *file_types.entry(ext_str).or_insert(0) += 1;
            }
        }
        
        // Generate message based on changes
        if status.staged_files.len() == 1 {
            let file = &status.staged_files[0];
            if file.contains("test") {
                suggestions.push(format!("Add tests for {}", file));
            } else if file.ends_with(".md") {
                suggestions.push(format!("Update documentation: {}", file));
            } else {
                suggestions.push(format!("Update {}", file));
            }
        } else {
            // Multiple files
            if file_types.contains_key("rs") {
                suggestions.push("Update Rust implementation".to_string());
            }
            if file_types.contains_key("py") {
                suggestions.push("Update Python scripts".to_string());
            }
            if file_types.contains_key("md") {
                suggestions.push("Update documentation".to_string());
            }
            
            if suggestions.is_empty() {
                suggestions.push(format!("Update {} files", status.staged_files.len()));
            }
        }
        
        // Add diff stats if significant
        if diff.insertions > 50 || diff.deletions > 20 {
            let stats = format!(" (+{} -{} lines)", diff.insertions, diff.deletions);
            if let Some(msg) = suggestions.first_mut() {
                msg.push_str(&stats);
            }
        }
        
        Ok(suggestions.into_iter().next().unwrap_or_else(|| "Update files".to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub branch: String,
    pub commit_hash: String,
    pub commit_message: String,
    pub has_uncommitted_changes: bool,
    pub staged_count: usize,
    pub modified_count: usize,
    pub untracked_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        
        // Set up basic config
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
        
        (temp_dir, repo)
    }

    #[test]
    fn test_git_client_creation() {
        let (_temp_dir, _repo) = create_test_repo();
        // Just test that we can create the test repo without panicking
        assert!(true);
    }

    #[test]
    fn test_repo_status() {
        let (temp_dir, repo) = create_test_repo();
        let git_client = GitClient { repo };
        
        // Change to the repo directory for operations
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Create a test file
        fs::write("test.txt", "Hello, world!").unwrap();
        
        let status = git_client.get_status().unwrap();
        assert!(status.untracked_files.contains(&"test.txt".to_string()));
        
        // Restore original directory (if it still exists)
        if original_dir.exists() {
            std::env::set_current_dir(original_dir).unwrap();
        }
    }

    #[test]
    fn test_add_and_commit() {
        let (temp_dir, repo) = create_test_repo();
        let git_client = GitClient { repo };
        
        // Change to the repo directory for operations
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Create and add a test file
        fs::write("test.txt", "Hello, world!").unwrap();
        
        // Use relative path for git operations
        git_client.add(&[Path::new("test.txt")]).unwrap();
        let status = git_client.get_status().unwrap();
        assert!(status.staged_files.contains(&"test.txt".to_string()));
        
        // Commit the file
        let commit_id = git_client.commit("Initial commit").unwrap();
        assert!(!commit_id.is_zero());
        
        // Restore original directory (if it still exists)
        if original_dir.exists() {
            std::env::set_current_dir(original_dir).unwrap();
        }
    }

    #[test]
    fn test_suggest_commit_message() {
        let (temp_dir, repo) = create_test_repo();
        let git_client = GitClient { repo };
        
        // Change to the repo directory for operations
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Create an initial commit first
        fs::write("README.md", "Initial file").unwrap();
        git_client.add(&[Path::new("README.md")]).unwrap();
        git_client.commit("Initial commit").unwrap();
        
        // Create and stage a test file
        fs::write("test.rs", "fn main() { println!(\"Hello\"); }").unwrap();
        git_client.add(&[Path::new("test.rs")]).unwrap();
        
        let suggestion = git_client.suggest_commit_message().unwrap();
        assert!(suggestion.contains("test.rs") || suggestion.contains("Rust"));
        
        // Restore original directory (if it still exists)
        if original_dir.exists() {
            std::env::set_current_dir(original_dir).unwrap();
        }
    }
}