use clap::Parser;

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Parser)]
#[command(name = "Todo App")]
#[command(version = "1.0")]
#[command(about = "A simple todo application", long_about = None)]
pub struct Cli {
    /// List all todos in a terminal UI
    #[arg(short, long)]
    pub list: bool,

    // Export todos indo Excel file
    #[arg(short = 'E', long)]
    pub export: bool,

    /// Add a new todo item
    #[arg(short = 'a', long, value_name = "TEXT", num_args = 1.., value_delimiter = ' ')]
    pub add: Option<Vec<String>>,

    /// PASS A LONG DESCRIPTION TO THE ARGUMENT
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
    #[arg(short = 'C', long = "done", value_name = "ID")]
    pub done: Option<i32>,

    /// Clear all todos
    #[arg(short = 'c', long)]
    pub clear: bool,

    /// Show all options
    #[arg(short, long)]
    pub show: bool,

    /// OWNER NAME
    #[arg(short, long, value_name = "OWNER", requires = "add")]
    pub owner: Option<String>,

    /// DUE DATE
    #[arg(short = 'd', long, value_name = "DUE DATE", requires = "add")]
    pub due: Option<String>,

    /// pass the API key credentrials
    #[arg(short = 'k', long, value_name = "API_KEY")]
    pub apikey: Option<String>,

    /// ASK GEMINI
    #[arg(short = 'A', long, value_name = "PROMPT")]
    pub prompt: Option<String>,

    /// Version Check
    #[arg(short, long)]
    pub release: bool,

    /// Clear the databse
    #[arg(short, long)]
    pub flush: bool,

    // Import todos from Excel file
    #[arg(short = 'I', long, value_name = "FILE")]
    pub import: Option<String>,
}
