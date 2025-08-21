use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: usize,
    pub priority: String,
    pub topic: String,
    pub text: String,
    pub desc: String,
    pub date_added: String,
    pub status: String,
    pub owner: String,
    pub due: String,
    pub subtasks: Vec<Subtask>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub todo_id: usize,
    pub subtask_id: usize,
    pub text: String,
    pub status: String,
}

#[derive(Debug, Parser)]
#[command(name = "VoiDo")]
#[command(version = "1.0")]
#[command(about = "A powerful and intuitive command-line (CLI) todo application built with Rust, supercharged with AI capabilities.", long_about = None)]
pub struct Cli {
    /// List all todos in a terminal UI
    #[arg(short, long)]
    pub list: bool,

    /// Export todos into an Excel file
    #[arg(short = 'E', long)]
    pub export: bool,

    /// Add a new todo item
    #[arg(short = 'a', long, value_name = "TEXT", num_args = 1.., value_delimiter = ' ')]
    pub add: Option<Vec<String>>,

    /// A more detailed description for the todo.
    /// Ownder of the todo (requires --add)
    #[arg(short = 'w', long, value_name = "DESCRIPTION", num_args = 1.., value_delimiter = ' ', requires = "add")]
    pub desc: Option<Vec<String>>,

    /// Topic for the new todo item (requires --add)
    #[arg(short = 't', long, value_name = "TOPIC", requires = "add")]
    pub topic: Option<String>,

    /// Priority for the todo (requires --add)
    #[arg(short = 'p', long, value_name = "PRIORITY", requires = "add")]
    pub priority: Option<String>,

    /// Print all todos to the console
    #[arg(short = 'P', long)]
    pub print: bool,

    /// Delete a todo by ID
    #[arg(short = 'D', long = "delete", value_name = "ID")]
    pub delete: Option<i32>,

    /// ID of the todo to update
    #[arg(short = 'u', long, value_name = "ID")]
    pub update_id: Option<i32>,

    /// New status for the todo (requires --update-id)
    #[arg(long, value_name = "STATUS", requires = "update_id")]
    pub status: Option<String>,

    /// Mark a todo as done by ID
    #[arg(short = 'c', long = "done", value_name = "ID")]
    pub done: Option<i32>,

    /// Clear all todos
    #[arg(short = 'C', long)]
    pub clear: bool,

    /// Show available command-line arguments and options.
    #[arg(short = 'S', long)]
    pub show: bool,

    /// The person responsible for the task.
    #[arg(short, long, value_name = "OWNER", requires = "add")]
    pub owner: Option<String>,

    /// A due date for the task.
    #[arg(short = 'd', long, value_name = "DUE DATE", requires = "add")]
    pub due: Option<String>,

    /// Set your Google Gemini API key.
    #[arg(short = 'k', long, value_name = "API_KEY")]
    pub apikey: Option<String>,

    /// Get AI-powered task suggestions from Google Gemini.
    #[arg(short = 'g', long, value_name = "PROMPT")]
    pub gemini: Option<String>,

    /// Display the current version of VoiDo.
    #[arg(short, long)]
    pub release: bool,

    /// Flush (clear) the entire database.
    #[arg(short, long)]
    pub flush: bool,

    // Import todos from Excel file
    #[arg(short = 'I', long, value_name = "FILE")]
    pub import: Option<String>,

    /// Synchronize todos with a GitHub repository.
    #[arg(short = 'G', long, value_name = "GITHUB")]
    pub github: bool,

    // Pass sub tasks that are part of a todo
    #[arg(short = 's', long, value_name = "SUB TASKS", requires = "add")]
    pub sub: Option<Vec<String>>,

    #[arg(
        short = 'T',
        long = "subtask",
        value_name = "ID:TEXT",
        value_parser = parse_subtask,
        help = "Add a subtask in the format `ID:TEXT` (e.g., `-T 2:\"my task\"`)"
    )]
    pub subtasks: Vec<(i32, String)>,
}

// Parses a string in the format `ID:TEXT` into `(i32, String)`
fn parse_subtask(s: &str) -> Result<(i32, String), String> {
    let Some((id_part, text_part)) = s.split_once(':') else {
        return Err("Expected format `ID:TEXT`".to_string());
    };
    let id = id_part.parse().map_err(|_| "ID must be a number")?;
    let text = text_part.trim_matches('"').to_string();
    Ok((id, text))
}
