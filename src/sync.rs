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
            self.create_github_repo(is_private)?;
        }

        // Check if we need to push
        let status = Command::new("git")
            .args(["status", "--porcelain", "--branch"])
            .current_dir(&self.config_dir)
            .output()?;

        let status_str = String::from_utf8_lossy(&status.stdout);
        if status_str.contains("ahead") {
            self.run_git_command(&["push", "-u", "origin", "main"], "Push to GitHub")?;
            println!("✓ Changes pushed to GitHub");
        } else {
            println!("✓ No changes to push (already up-to-date)");
        }

        Ok(())
    }

    fn create_github_repo(&self, is_private: bool) -> Result<(), Box<dyn std::error::Error>> {
        // First try using GitHub CLI if available
        if Command::new("gh")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
        {
            let privacy_flag = if is_private { "--private" } else { "--public" };
            self.run_command(
                "gh",
                &[
                    "repo",
                    "create",
                    &self.repo_name,
                    privacy_flag,
                    "--source=.",
                    "--remote=origin",
                    "--push",
                ],
                "Create GitHub repository (using gh CLI)",
            )?;
        } else {
            // Manual git remote setup
            let remote_url = format!(
                "git@github.com:{}/{}.git",
                self.git_username, self.repo_name
            );
            self.run_git_command(&["remote", "add", "origin", &remote_url], "Add git remote")?;

            println!(
                "Repository created at: https://github.com/{}/{}",
                self.git_username, self.repo_name
            );
            println!("Note: You need to create this repository on GitHub first");
        }

        Ok(())
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
        println!("> {}", description);
        self.run_command("git", args, description)
    }

    fn run_command(
        &self,
        cmd: &str,
        args: &[&str],
        description: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let status = Command::new(cmd)
            .args(args)
            .current_dir(&self.config_dir)
            .status()?;

        if !status.success() {
            return Err(format!("Failed to {} ({} {})", description, cmd, args.join(" ")).into());
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

    let backup = sync.backup_todos(todos)?;
    println!("✓ Todos backed up to: {}", backup.display());

    // Step 1: Create backup file
    let backup_path = sync.backup_todos(todos)?;
    println!("✓ Todos backed up to: {}", backup_path.display());

    // Step 2: Initialize repository
    sync.init_repo()?;
    println!("✓ Git repository initialized");

    // Step 3: Commit changes
    sync.commit_changes("Update todo list")?;
    println!("✓ Changes committed");

    // Step 4: Sync with GitHub
    sync.sync_to_github()?;
    println!("✓ Successfully synced with GitHub!");

    Ok(())
}
