use project_x::utils::git_client::GitClient;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🔧 Git Client Demo");
    
    // Create a git client for the current repository
    let git_client = GitClient::new(None)?;
    
    // Get repository information
    println!("\n📊 Repository Information:");
    let repo_info = git_client.get_repo_info()?;
    println!("Current branch: {}", repo_info.branch);
    println!("Latest commit: {} - {}", 
             &repo_info.commit_hash[..8], 
             repo_info.commit_message.lines().next().unwrap_or(""));
    println!("Has uncommitted changes: {}", repo_info.has_uncommitted_changes);
    
    // Get repository status
    println!("\n📋 Repository Status:");
    let status = git_client.get_status()?;
    
    if !status.staged_files.is_empty() {
        println!("Staged files ({}):", status.staged_files.len());
        for file in &status.staged_files {
            println!("  ✅ {}", file);
        }
    }
    
    if !status.modified_files.is_empty() {
        println!("Modified files ({}):", status.modified_files.len());
        for file in &status.modified_files {
            println!("  📝 {}", file);
        }
    }
    
    if !status.untracked_files.is_empty() {
        println!("Untracked files ({}):", status.untracked_files.len());
        for file in status.untracked_files.iter().take(5) {  // Limit to 5 for readability
            println!("  ❓ {}", file);
        }
        if status.untracked_files.len() > 5 {
            println!("  ... and {} more", status.untracked_files.len() - 5);
        }
    }
    
    if !status.deleted_files.is_empty() {
        println!("Deleted files ({}):", status.deleted_files.len());
        for file in &status.deleted_files {
            println!("  🗑️ {}", file);
        }
    }
    
    if status.staged_files.is_empty() && status.modified_files.is_empty() && 
       status.untracked_files.is_empty() && status.deleted_files.is_empty() {
        println!("✅ Working directory is clean!");
    }
    
    // Show diff if there are changes
    if !status.modified_files.is_empty() || !status.staged_files.is_empty() {
        println!("\n📄 Working Directory Diff (unstaged changes):");
        match git_client.get_diff(false) {
            Ok(diff) => {
                if !diff.diff_text.is_empty() {
                    println!("Files changed: {}, +{} -{} lines", 
                            diff.file_count, diff.insertions, diff.deletions);
                    
                    // Show first few lines of diff
                    let lines: Vec<&str> = diff.diff_text.lines().take(20).collect();
                    for line in lines {
                        println!("{}", line);
                    }
                    if diff.diff_text.lines().count() > 20 {
                        println!("... (diff truncated, {} total lines)", diff.diff_text.lines().count());
                    }
                } else {
                    println!("No unstaged changes to show");
                }
            },
            Err(e) => println!("Error getting diff: {}", e)
        }
        
        if !status.staged_files.is_empty() {
            println!("\n📄 Staged Changes Diff:");
            match git_client.get_diff(true) {
                Ok(diff) => {
                    if !diff.diff_text.is_empty() {
                        println!("Files changed: {}, +{} -{} lines", 
                                diff.file_count, diff.insertions, diff.deletions);
                        
                        // Show first few lines of diff
                        let lines: Vec<&str> = diff.diff_text.lines().take(10).collect();
                        for line in lines {
                            println!("{}", line);
                        }
                        if diff.diff_text.lines().count() > 10 {
                            println!("... (diff truncated)");
                        }
                    } else {
                        println!("No staged changes to show");
                    }
                },
                Err(e) => println!("Error getting staged diff: {}", e)
            }
        }
    }
    
    // Generate commit message suggestion if there are staged files
    if !status.staged_files.is_empty() {
        println!("\n💡 Suggested Commit Message:");
        match git_client.suggest_commit_message() {
            Ok(suggestion) => println!("\"{}\"", suggestion),
            Err(e) => println!("Error generating suggestion: {}", e)
        }
    }
    
    // Show example usage
    println!("\n🛠️  Example Git Operations:");
    println!("  • Add files: git_client.add(&[Path::new(\"file.rs\")])?");
    println!("  • Add all: git_client.add_all()?");
    println!("  • Commit: git_client.commit(\"Your commit message\")?");
    println!("  • Check status: git_client.get_status()?");
    println!("  • Get diff: git_client.get_diff(false)?  // false = unstaged, true = staged");
    
    println!("\n✅ Git client demo complete!");
    
    Ok(())
}