use arguments::{
    delete_todo,
    models::{self, Cli, Todo},
};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use data::sample_todos;
use ratatui::widgets::{ListState, TableState};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
};
use search::{FuzzySearch, InputField};
use std::io;
use ui::{calculate_stats, draw_ui};

mod ai; // LLMS stuff
mod args; // Print all the args available in the App so it does not clutter the main.rs
mod arguments;
mod configs;
mod data; // DATABASE STUFF;
mod database;
mod modals; // All the modals logic
mod search;
mod settings;
mod ui; // ALL THE UI STUFF
mod xls; // Fuzy serach and UI input logic

#[derive(Debug)]
pub enum InputMode {
    Normal,
    Search,
}

#[derive(Debug)]
pub struct App {
    pub todos: Vec<Todo>,
    pub state: TableState,
    pub show_modal: bool,
    pub selected_todo: Option<Todo>,
    pub show_delete_confirmation: bool,
    pub show_priority_modal: bool,
    pub show_main_menu_modal: bool,
    pub subtask_state: ListState,
    pub selected_subtask: Option<String>,
    pub show_search_input: bool,
    pub input_mode: InputMode,
    pub fuzzy_search: FuzzySearch,
    pub filtered_indices: Vec<usize>,
}

impl App {
    fn new(todos: Vec<Todo>) -> Self {
        let mut state = TableState::default();
        let filtered_indices = (0..todos.len()).collect();
        state.select(Some(0)); // Select first item by default
        Self {
            todos,
            state,
            show_modal: false,
            selected_todo: None,
            show_delete_confirmation: false,
            show_priority_modal: false,
            show_main_menu_modal: false,
            subtask_state: ListState::default(),
            selected_subtask: None,
            show_search_input: true,
            input_mode: InputMode::Normal,
            fuzzy_search: FuzzySearch::new(),
            filtered_indices,
        }
    }

    // Change subtask status
    fn change_subtask_status(
        &mut self,
        todo_id: i32,
        subtask_id: i32,
        status: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let db = database::DBtodo::new()?;
        db.change_subtask_status(todo_id, subtask_id, status)?;
        Ok(())
    }

    // Update TODOS to ensure SYNC with DB
    pub fn load_todo(&mut self, todo_id: usize) {
        if let Ok(db) = database::DBtodo::new() {
            if let Ok(todos) = db.get_todos() {
                // Update the selected todo
                if let Some(updated_todo) = todos.iter().find(|t| t.id == todo_id).cloned() {
                    // Preserve selection state
                    let prev_selected = self.subtask_state.selected();

                    // Update selected todo
                    self.selected_todo = Some(updated_todo.clone());

                    // Update the main todos list
                    if let Some(todo) = self.todos.iter_mut().find(|t| t.id == todo_id) {
                        *todo = updated_todo;
                    }

                    // Restore selection
                    if let Some(selected) = prev_selected {
                        self.subtask_state.select(Some(selected));
                    }
                }
            }
        }
    } // CHANGE todo Priority
    fn change_priority(
        &mut self,
        id: i32,
        priority: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let db = database::DBtodo::new()?;
        db.update_priority(id, priority.clone())?;

        // Find the todo by ID instead of using ID as index
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id as usize) {
            todo.priority = priority;
        }

        Ok(())
    }

    fn handle_priority_change(&mut self, priority: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(selected) = self.state.selected() {
            if selected < self.todos.len() {
                let id = self.todos[selected].id;
                self.show_priority_modal = false;
                self.change_priority(id as i32, priority.to_string())?;
            } else {
                return Err("Selected index out of bounds!".into());
            }
        }
        Ok(())
    }

    // CHANGE TODO STATUS
    fn change_todo_status(
        &mut self,
        id: i32,
        status: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate selection exists
        let selected = self.state.selected().ok_or("No todo selected")?;

        // Validate selection is within bounds
        if selected >= self.todos.len() {
            return Err("Invalid selection".into());
        }

        // Update database
        let db = database::DBtodo::new()?;
        db.update_todo(id, Some(status.clone()))?;

        // Update local state
        self.todos[selected].status = status;

        // Maintain selection position
        if !self.todos.is_empty() {
            let new_selection = selected.min(self.todos.len().saturating_sub(1));
            self.state.select(Some(new_selection));
        }

        Ok(())
    }

    // Delete current selected TODO
    fn delete_current_todo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(selected) = self.state.selected() {
            if selected < self.todos.len() {
                let id = self.todos[selected].id;
                let db = database::DBtodo::new()?;
                db.delete_todo(id as i32)?;

                // Update local state
                self.todos.remove(selected);

                // Adjust selection
                if !self.todos.is_empty() {
                    self.state.select(Some(selected.min(self.todos.len() - 1)));
                } else {
                    self.state.select(None);
                }
            }
        }
        Ok(())
    }

    // Delete current TODO subtask
    fn delete_current_subtask(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(selected) = self.subtask_state.selected() {
            if selected < self.selected_todo.as_ref().unwrap().subtasks.len() {
                let id = self.selected_todo.as_ref().unwrap().subtasks[selected].subtask_id;
                let db = database::DBtodo::new()?;
                db.delete_subtask(id as i32)?;

                // Update local state
                self.selected_todo
                    .as_mut()
                    .unwrap()
                    .subtasks
                    .remove(selected);

                // Adjust selection
                if !self.selected_todo.as_ref().unwrap().subtasks.is_empty() {
                    self.subtask_state.select(Some(
                        selected.min(self.selected_todo.as_ref().unwrap().subtasks.len() - 1),
                    ));
                } else {
                    self.subtask_state.select(None);
                }
            }
        }
        Ok(())
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.todos.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.todos.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn select_current(&mut self) {
        if let Some(filtered_index) = self.state.selected() {
            if filtered_index < self.filtered_indices.len() {
                let original_index = self.filtered_indices[filtered_index];
                if original_index < self.todos.len() {
                    self.selected_todo = Some(self.todos[original_index].clone());
                    self.show_modal = true;
                }
            }
        }
    }

    fn close_modal(&mut self) {
        self.show_modal = false;
        self.selected_todo = None;
        self.show_priority_modal = false;
        self.show_main_menu_modal = false;

        // Re-apply filter if there's text in the search input
        if !self.fuzzy_search.input.value.is_empty() {
            self.fuzzy_search.update_matches(&self.todos);
            self.update_filtered_todos();
        }
    }

    fn handle_fuzzy_search(&mut self, event: &Event) -> bool {
        let event_handled = self.fuzzy_search.handle_event(event);

        if event_handled {
            // Always update matches and filtered todos if any event was handled by fuzzy search
            self.fuzzy_search.update_matches(&self.todos);
            self.update_filtered_todos();
        }
        event_handled
    }

    fn update_filtered_todos(&mut self) {
        // Update the filtered indices
        self.filtered_indices = self.fuzzy_search.matched_indices().to_vec();

        // Update table selection to match the fuzzy search selection
        if !self.filtered_indices.is_empty() {
            let selected_idx = self
                .fuzzy_search
                .selected_match()
                .min(self.filtered_indices.len().saturating_sub(1));
            self.state.select(Some(selected_idx));
        } else {
            self.state.select(None);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    // Create the configs
    let _ = configs::AppConfigs::create_default_config();

    // Initiate the base configs the user can tweak
    let _user_settings = settings::settings::AppConfig::create_default_config();

    let cli = Cli::parse();

    // Check if no arguments were provided
    let no_args_provided = std::env::args().count() == 1;

    // Terminal UI mode (default when no args provided or when --list is explicitly set)
    if cli.list || no_args_provided {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let todos = sample_todos();
        let mut app = App::new(todos);

        loop {
            terminal.draw(|f| draw_ui(f, &mut app))?;
            if let Event::Key(key) = event::read()? {
                if app.fuzzy_search.input.active {
                    if key.code == KeyCode::Enter {
                        app.fuzzy_search.input.unfocus();
                        app.input_mode = InputMode::Normal;
                        app.select_current(); // Select and show details immediately
                        continue; // Consume the event here
                    } else if key.code == KeyCode::Esc {
                        app.fuzzy_search.input.unfocus();
                        app.fuzzy_search.input.value.clear();
                        app.fuzzy_search.update_matches(&app.todos);
                        app.update_filtered_todos();
                        app.input_mode = InputMode::Normal;
                        continue;
                    } else if app.handle_fuzzy_search(&Event::Key(key)) {
                        continue;
                    }
                }

                match key.code {
                    KeyCode::Char('i') if !app.fuzzy_search.input.active => {
                        app.fuzzy_search.input.focus();
                        app.input_mode = InputMode::Search;
                        continue;
                    }
                    // Handle subtask navigation
                    // Only handle subtask navigation when modal is visible
                    KeyCode::Char('j') | KeyCode::Down if app.show_modal => {
                        if let Some(selected_todo) = &app.selected_todo {
                            if let Some(selected) = app.subtask_state.selected() {
                                if selected + 1 < selected_todo.subtasks.len() {
                                    app.subtask_state.select(Some(selected + 1));
                                }
                            } else if !selected_todo.subtasks.is_empty() {
                                app.subtask_state.select(Some(0));
                            }
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up if app.show_modal => {
                        if let Some(selected) = app.subtask_state.selected() {
                            if selected > 0 {
                                app.subtask_state.select(Some(selected - 1));
                            }
                        }
                    }
                    KeyCode::Char(' ') if app.show_modal => {
                        if let Some(selected) = app.subtask_state.selected() {
                            if let Some(todo) = &mut app.selected_todo {
                                if selected < todo.subtasks.len() {
                                    let subtask = &mut todo.subtasks[selected];
                                    subtask.status = if subtask.status == "Done" {
                                        "Pending".to_string()
                                    } else {
                                        "Done".to_string()
                                    };
                                }
                            }
                        }
                    }

                    // CHANGE SUBTASK STATUS
                    KeyCode::Char('d') if app.show_modal => {
                        // Early return if no selection or no todo
                        let Some(selected) = app.subtask_state.selected() else {
                            continue;
                        };
                        let Some(todo) = &app.selected_todo else {
                            continue;
                        };
                        let Some(subtask) = todo.subtasks.get(selected) else {
                            continue;
                        };

                        // Prepare update parameters
                        let todo_id = todo.id;
                        let subtask_id = subtask.subtask_id;

                        // Determine new status
                        let new_status = if subtask.status == "Done" {
                            "Pending".to_string()
                        } else {
                            "Done".to_string()
                        };

                        // Update database
                        if let Err(e) = app.change_subtask_status(
                            todo_id as i32,
                            subtask_id as i32,
                            new_status.clone(),
                        ) {
                            eprintln!("Error updating subtask: {}", e);
                            continue;
                        }

                        // Update both in-memory states
                        if let Some(todo) = &mut app.selected_todo {
                            if let Some(subtask) = todo.subtasks.get_mut(selected) {
                                subtask.status = new_status.clone();
                            }
                        }

                        // Update the main todos list
                        if let Some(todo) = app.todos.iter_mut().find(|t| t.id == todo_id) {
                            if let Some(subtask) = todo
                                .subtasks
                                .iter_mut()
                                .find(|s| s.subtask_id == subtask_id)
                            {
                                subtask.status = new_status;
                            }
                        }

                        // Force a full refresh from DB to ensure consistency
                        app.load_todo(todo_id);
                    }
                    //////
                    KeyCode::Char('d') => {
                        if let Some(selected) = app.state.selected() {
                            if selected < app.todos.len() {
                                let id = app.todos[selected].id;
                                let status = "Done".to_string();
                                if let Err(e) = app.change_todo_status(id as i32, status) {
                                    eprintln!("Error updating todo status: {}", e);
                                }
                            }
                        }
                    }

                    KeyCode::Char('o') => {
                        if let Some(selected) = app.state.selected() {
                            if selected < app.todos.len() {
                                let id = app.todos[selected].id;
                                let status = "Ongoing".to_string();
                                if let Err(e) = app.change_todo_status(id as i32, status) {
                                    eprintln!("Error updating todo status: {}", e);
                                }
                            }
                        }
                    }

                    KeyCode::Char('p') => {
                        if let Some(selected) = app.state.selected() {
                            if selected < app.todos.len() {
                                let id = app.todos[selected].id;
                                let status = "Pending".to_string();
                                if let Err(e) = app.change_todo_status(id as i32, status) {
                                    eprintln!("Error updating todo status: {}", e);
                                }
                            }
                        }
                    }

                    // Show main menu modal
                    KeyCode::Char('\\') => {
                        app.show_main_menu_modal = !app.show_main_menu_modal;
                    }

                    // SHOW PRIORITY MODAL
                    KeyCode::Char('P') => {
                        if let Some(selected) = app.state.selected() {
                            app.close_modal();
                            if selected < app.todos.len() {
                                app.show_priority_modal = true;
                            }
                        }
                    }

                    // Handle priority changes
                    KeyCode::Char('L') => {
                        if let Err(e) = app.handle_priority_change("Low") {
                            eprintln!("Error updating priority: {}", e);
                        }
                    }

                    KeyCode::Char('M') => {
                        if let Err(e) = app.handle_priority_change("Medium") {
                            eprintln!("Error updating priority: {}", e);
                        }
                    }

                    KeyCode::Char('H') => {
                        if let Err(e) = app.handle_priority_change("High") {
                            eprintln!("Error updating priority: {}", e);
                        }
                    }

                    // Delete todo
                    KeyCode::Delete | KeyCode::Char('x') => {
                        if !app.todos.is_empty() && !app.show_modal {
                            app.show_delete_confirmation = true;
                        }

                        // IF THE TODO MODAL IS SHOWING THE SUBTASKS
                        if app.show_modal {
                            // Execute the delete action on the subtasks only
                            if let Err(e) = app.delete_current_subtask() {
                                eprintln!("Error deleting subtask: {}", e);
                            }
                        }
                    }

                    // Handle delete confirmation
                    KeyCode::Char('y') if app.show_delete_confirmation => {
                        if let Err(e) = app.delete_current_todo() {
                            eprintln!("Error deleting todo: {}", e);
                        }
                        app.show_delete_confirmation = false;
                    }

                    KeyCode::Char('n') if app.show_delete_confirmation => {
                        app.show_delete_confirmation = false;
                    }
                    KeyCode::Char('q') => break,
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Enter | KeyCode::Char('l') => {
                        if app.show_modal
                            || app.show_main_menu_modal
                            || app.show_priority_modal
                            || app.show_delete_confirmation
                        {
                            app.close_modal();
                        } else {
                            app.select_current();
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('h') => {
                        if app.show_modal || app.show_priority_modal || app.show_main_menu_modal {
                            app.close_modal();
                        }
                    }
                    _ => {}
                }
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
    }
    // Append subtask to already existing TODO
    else if !cli.subtasks.is_empty() {
        for (id, text) in &cli.subtasks {
            match arguments::add_todo::append_subtask(*id, text.clone()) {
                Ok(_) => println!("✅ Subtask {}: '{}' added successfully!", id, text),
                Err(e) => eprintln!("Error adding subtask {}: {}", id, e),
            }
        }
    }
    // Import todos from excel file
    else if let Some(file_path) = cli.import {
        let _workbook = xls::import_todos(&file_path);
    }
    // Export TODOs into Excel File
    else if cli.export {
        let _workbook = xls::export_todos();
    }
    // PROMPT GEMINI
    else if let Some(prompt) = cli.prompt {
        match ai::ask_gemini(prompt).await {
            Ok(response) => {
                println!("");
                println!("🤖 {}", response);
                println!("")
            }
            Err(e) => eprintln!(
                "Error: {}. Please set an API key first using the -k flag.",
                e
            ),
        }
    }
    // Print version
    else if cli.release {
        println!("voido {}", env!("CARGO_PKG_VERSION"));
    }
    // Pass the API key
    else if let Some(key) = cli.apikey {
        let db = database::DBtodo::new().unwrap();
        db.set_api_credentials(Some(key)).unwrap_or_else(|e| {
            eprintln!("Error setting API credentials: {}", e);
        })
    }
    // Add new todo
    else if let Some(words) = cli.add {
        let text = words.join(" ");
        let desc = cli.desc.map(|desc| desc.join(" "));
        // get the subtasks that can be a vector of strings
        // Initialize subtasks vector
        let mut subtasks = Vec::new();

        // Extract subtasks from the command-line argument
        if let Some(sub_vec) = cli.sub {
            for subtask in sub_vec {
                subtasks.push(subtask);
            }
        }

        match arguments::add_todo::add_todo(
            text,
            cli.topic,
            cli.priority,
            cli.owner,
            cli.due,
            desc,
            subtasks,
        ) {
            Ok(_) => println!("✅ Todo added successfully!"),
            Err(e) => eprintln!("Error adding todo: {}", e),
        }
    }
    // Delete todo
    else if let Some(id) = cli.delete {
        match arguments::delete_todo::remove_todo(id) {
            Ok(_) => println!("✅ Todo deleted successfully!"),
            Err(e) => eprintln!("Error deleting todo: {}", e),
        }
    }
    // Update todo status
    else if let (Some(id), Some(status)) = (cli.update_id, cli.status) {
        if let Err(e) = arguments::update_todo::update_todo(id, status) {
            eprintln!("Error updating todo: {}", e);
        }
    }
    // UPDATE USING SHORT FORMAT
    else if let Some(id) = cli.done {
        if let Err(e) = arguments::update_todo::update_todo(id, "Done".to_string()) {
            eprintln!("Error updating todo: {}", e);
        }
    }
    // Clear all todos
    else if cli.clear {
        match arguments::delete_todo::clear_todos() {
            Ok(_) => println!("Todos deleted successfully!"),
            Err(e) => eprintln!("Error deleting todos: {}", e),
        }
    }
    // Print todos
    else if cli.print {
        arguments::print::print_todos();
    }
    // Print args
    else if cli.show {
        args::print_args();
    }
    // Clear the databse
    else if cli.flush {
        match database::DBtodo::new() {
            Ok(mut db) => match db.flush_db() {
                Ok(_) => println!(" Database flushed successfully!"),
                Err(e) => eprintln!("Error flushing database: {}", e),
            },
            Err(e) => eprintln!("Error creating database: {}", e),
        }
    }

    Ok(())
}
