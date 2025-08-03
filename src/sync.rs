use directories::BaseDirs;
use serde_json;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};

use crate::arguments::models::Todo;
use crate::{configs, data};

#[derive(Debug)]
pub struct GitHubSync {
    config_dir: PathBuf,
    repo_name: String,
    git_username: String,
}

#[derive(Debug)]
pub enum AuthMethod {
    SSH,
    HTTPS,
    Unknown,
}

impl GitHubSync {
    pub fn new(repo_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let base_dirs = BaseDirs::new().ok_or("Could not determine home directory")?;
        let config_dir = base_dirs.config_dir().join("voido");

        // Create directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;

        // Get git username
        let git_username = Command::new("git")
            .arg("config")
            .arg("user.name")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_else(|| "your-username".to_string())
            .trim()
            .to_string();

        Ok(GitHubSync {
            config_dir,
            repo_name: repo_name.to_string(),
            git_username,
        })
    }

    pub fn commit_changes(&self, message: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // First check if there are changes to commit
        let status = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.config_dir)
            .output()?;

        if status.stdout.is_empty() {
            println!("✓ No changes to commit");
            return Ok(false);
        }

        self.run_git_command(&["add", "."], "Stage all files")?;
        self.run_git_command(&["commit", "-m", message], "Commit changes")?;
        Ok(true)
    }

    pub fn backup_todos(&self, todos: &[Todo]) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let backup_path = self.config_dir.join("voido_BAK.json");
        let json_todos = serde_json::to_string_pretty(todos)?;
        fs::write(&backup_path, json_todos)?;
        Ok(backup_path)
    }

    pub fn init_repo(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_git_repo()? {
            self.run_git_command(&["init"], "Initialize git repository")?;
            self.run_git_command(&["branch", "-M", "main"], "Rename branch to main")?;
        }
        Ok(())
    }

    pub fn sync_to_github(&self) -> Result<(), Box<dyn std::error::Error>> {
        let is_private = true;

        // Check if remote exists
        if !self.has_remote("origin")? {
            self.setup_github_repo(is_private)?;
        }

        // Verify authentication before attempting to push
        self.verify_github_auth()?;

        // Check if we need to push
        let status = Command::new("git")
            .args(["status", "--porcelain", "--branch"])
            .current_dir(&self.config_dir)
            .output()?;

        let status_str = String::from_utf8_lossy(&status.stdout);
        if status_str.contains("ahead") || status_str.contains("Initial commit") {
            self.push_with_retry()?;
            println!("✓ Changes pushed to GitHub");
        } else {
            println!("✓ No changes to push (already up-to-date)");
        }

        Ok(())
    }

    fn setup_github_repo(&self, is_private: bool) -> Result<(), Box<dyn std::error::Error>> {
        // First try using GitHub CLI if available
        if self.is_gh_cli_available() && self.is_gh_authenticated()? {
            println!("📦 Creating GitHub repository using GitHub CLI...");
            let privacy_flag = if is_private { "--private" } else { "--public" };

            match self.run_command(
                "gh",
                &[
                    "repo",
                    "create",
                    &self.repo_name,
                    privacy_flag,
                    "--source=.",
                    "--remote=origin",
                ],
                "Create GitHub repository (using gh CLI)",
            ) {
                Ok(_) => {
                    println!("✓ Repository created successfully with GitHub CLI");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("⚠️  GitHub CLI failed: {}", e);
                    println!("Falling back to manual setup...");
                }
            }
        }

        // Manual setup - try SSH first, then HTTPS
        self.setup_manual_remote()?;
        Ok(())
    }

    fn setup_manual_remote(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Try SSH first (recommended for security)
        if self.has_ssh_key()? {
            let ssh_url = format!(
                "git@github.com:{}/{}.git",
                self.git_username, self.repo_name
            );
            println!("🔐 Setting up SSH remote...");

            match self.run_git_command(&["remote", "add", "origin", &ssh_url], "Add SSH remote") {
                Ok(_) => {
                    println!("✓ SSH remote configured");
                    self.print_manual_repo_instructions(&ssh_url, AuthMethod::SSH);
                    return Ok(());
                }
                Err(_) => {
                    eprintln!("⚠️  SSH setup failed, trying HTTPS...");
                }
            }
        }

        // Fallback to HTTPS with token
        let https_url = format!(
            "https://github.com/{}/{}.git",
            self.git_username, self.repo_name
        );
        self.run_git_command(&["remote", "add", "origin", &https_url], "Add HTTPS remote")?;
        println!("✓ HTTPS remote configured");
        self.print_manual_repo_instructions(&https_url, AuthMethod::HTTPS);

        Ok(())
    }

    fn verify_github_auth(&self) -> Result<(), Box<dyn std::error::Error>> {
        let auth_method = self.detect_auth_method()?;

        match auth_method {
            AuthMethod::SSH => self.verify_ssh_auth()?,
            AuthMethod::HTTPS => self.verify_https_auth()?,
            AuthMethod::Unknown => {
                return Err("Unable to determine authentication method. Please check your git remote configuration.".into());
            }
        }

        Ok(())
    }

    fn detect_auth_method(&self) -> Result<AuthMethod, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(&self.config_dir)
            .output()?;

        if !output.status.success() {
            return Ok(AuthMethod::Unknown);
        }

        let url = String::from_utf8_lossy(&output.stdout);
        if url.starts_with("git@github.com") {
            Ok(AuthMethod::SSH)
        } else if url.starts_with("https://github.com") {
            Ok(AuthMethod::HTTPS)
        } else {
            Ok(AuthMethod::Unknown)
        }
    }

    fn verify_ssh_auth(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔐 Verifying SSH authentication...");

        let output = Command::new("ssh")
            .args(["-T", "git@github.com"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);

        if stderr.contains("successfully authenticated") {
            println!("✓ SSH authentication verified");
            Ok(())
        } else {
            Err(format!(
                "SSH authentication failed. Please ensure:\n\
                1. You have generated an SSH key: ssh-keygen -t ed25519 -C \"your_email@example.com\"\n\
                2. Added it to ssh-agent: ssh-add ~/.ssh/id_ed25519\n\
                3. Added the public key to your GitHub account\n\
                4. Test with: ssh -T git@github.com\n\
                \nError: {}", stderr
            ).into())
        }
    }

    fn verify_https_auth(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔐 Verifying HTTPS authentication...");

        // Check if credential helper is configured
        let output = Command::new("git")
            .args(["config", "--get", "credential.helper"])
            .output()?;

        if !output.status.success() || output.stdout.is_empty() {
            return Err(
                "No credential helper configured for HTTPS authentication.\n\
                Please set up authentication:\n\
                1. Generate a Personal Access Token at: https://github.com/settings/tokens\n\
                2. Configure credential helper: git config --global credential.helper store\n\
                3. Or use GitHub CLI: gh auth login\n\
                \nNote: GitHub no longer accepts passwords for Git operations."
                    .into(),
            );
        }

        println!("✓ Credential helper configured");
        Ok(())
    }

    fn push_with_retry(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📤 Pushing to GitHub...");

        // First attempt
        match self.run_git_command(&["push", "-u", "origin", "main"], "Push to GitHub") {
            Ok(_) => return Ok(()),
            Err(e) => {
                let error_str = e.to_string();

                // Handle common authentication errors
                if error_str.contains("Permission denied")
                    || error_str.contains("authentication failed")
                {
                    return Err(format!(
                        "Authentication failed. {}\n\
                        \nFor SSH: Ensure your SSH key is added to GitHub\n\
                        For HTTPS: Use a Personal Access Token instead of password\n\
                        \nOriginal error: {}",
                        self.get_auth_help_message()?,
                        e
                    )
                    .into());
                }

                if error_str.contains("repository does not exist") {
                    return Err(format!(
                        "Repository does not exist on GitHub.\n\
                        Please create it manually at: https://github.com/new\n\
                        Repository name: {}\n\
                        \nOriginal error: {}",
                        self.repo_name, e
                    )
                    .into());
                }

                return Err(e);
            }
        }
    }

    fn get_auth_help_message(&self) -> Result<String, Box<dyn std::error::Error>> {
        let auth_method = self.detect_auth_method()?;

        let message = match auth_method {
            AuthMethod::SSH => {
                "SSH Authentication Help:\n\
                1. Generate SSH key: ssh-keygen -t ed25519 -C \"your_email@example.com\"\n\
                2. Add to ssh-agent: ssh-add ~/.ssh/id_ed25519\n\
                3. Copy public key: cat ~/.ssh/id_ed25519.pub\n\
                4. Add to GitHub: https://github.com/settings/ssh/new\n\
                5. Test: ssh -T git@github.com"
            }
            AuthMethod::HTTPS => {
                "HTTPS Authentication Help:\n\
                1. Create Personal Access Token: https://github.com/settings/tokens\n\
                2. Select scopes: 'repo' for private repos, 'public_repo' for public\n\
                3. Use token as password when prompted\n\
                4. Or configure credential helper: git config --global credential.helper store"
            }
            AuthMethod::Unknown => "Please check your git remote configuration",
        };

        Ok(message.to_string())
    }

    fn has_ssh_key(&self) -> Result<bool, io::Error> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let ssh_dir = Path::new(&home).join(".ssh");

        // Check for common SSH key files
        let key_files = ["id_ed25519", "id_rsa", "id_ecdsa"];
        for key_file in &key_files {
            if ssh_dir.join(key_file).exists() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn is_gh_cli_available(&self) -> bool {
        Command::new("gh")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn is_gh_authenticated(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let output = Command::new("gh")
            .args(["auth", "status"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        Ok(output.status.success())
    }

    fn print_manual_repo_instructions(&self, remote_url: &str, auth_method: AuthMethod) {
        println!("\n📋 Manual Setup Required:");
        println!("1. Create a new repository on GitHub:");
        println!("   → https://github.com/new");
        println!("   → Repository name: {}", self.repo_name);
        println!("   → Set as private: Yes");
        println!("   → Do NOT initialize with README, .gitignore, or license");
        println!("\n2. Remote URL configured: {}", remote_url);

        match auth_method {
            AuthMethod::SSH => {
                println!("\n3. SSH Authentication:");
                println!("   → Ensure your SSH key is added to GitHub");
                println!("   → Test with: ssh -T git@github.com");
            }
            AuthMethod::HTTPS => {
                println!("\n3. HTTPS Authentication:");
                println!(
                    "   → Create a Personal Access Token at: https://github.com/settings/tokens"
                );
                println!("   → Use the token as your password when Git prompts");
                println!("   → GitHub no longer accepts account passwords for Git operations");
            }
            AuthMethod::Unknown => {}
        }

        println!("\n4. Repository will be available at:");
        println!(
            "   → https://github.com/{}/{}",
            self.git_username, self.repo_name
        );
    }

    fn is_git_repo(&self) -> Result<bool, io::Error> {
        Ok(self.config_dir.join(".git").exists())
    }

    fn has_remote(&self, remote: &str) -> Result<bool, io::Error> {
        let output = Command::new("git")
            .args(["remote", "get-url", remote])
            .current_dir(&self.config_dir)
            .output()?;

        Ok(output.status.success())
    }

    fn run_git_command(
        &self,
        args: &[&str],
        description: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("⚡ {}", description);
        self.run_command("git", args, description)
    }

    fn run_command(
        &self,
        cmd: &str,
        args: &[&str],
        description: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new(cmd)
            .args(args)
            .current_dir(&self.config_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!(
                "Failed to {} ({} {})\nStdout: {}\nStderr: {}",
                description,
                cmd,
                args.join(" "),
                stdout,
                stderr
            )
            .into());
        }
        Ok(())
    }
}

// Usage with CLI flag
pub fn handle_github_sync() -> Result<(), Box<dyn std::error::Error>> {
    let todos = &data::sample_todos();

    let configs = configs::AppConfigs::read_configs_from_file().unwrap();

    let repo_name = &configs.repo_name;

    let sync = GitHubSync::new(repo_name)?;

    println!("🚀 Starting GitHub sync for repository: {}", repo_name);

    // Step 1: Create backup file
    let backup_path = sync.backup_todos(todos)?;
    println!("✓ Todos backed up to: {}", backup_path.display());

    // Step 2: Initialize repository
    sync.init_repo()?;
    println!("✓ Git repository initialized");

    // Step 3: Commit changes
    let has_changes = sync.commit_changes("Update todo list")?;
    if has_changes {
        println!("✓ Changes committed");
    }

    // Step 4: Sync with GitHub
    match sync.sync_to_github() {
        Ok(_) => {
            println!("🎉 Successfully synced with GitHub!");
            println!(
                "   Repository: https://github.com/{}/{}",
                sync.git_username, sync.repo_name
            );
        }
        Err(e) => {
            eprintln!("❌ Failed to sync with GitHub: {}", e);
            eprintln!("\n💡 Troubleshooting tips:");
            eprintln!("   • Ensure you have proper GitHub authentication set up");
            eprintln!("   • For SSH: Add your SSH key to GitHub");
            eprintln!("   • For HTTPS: Use a Personal Access Token");
            eprintln!("   • GitHub no longer accepts passwords for Git operations");
            return Err(e);
        }
    }

    Ok(())
}
