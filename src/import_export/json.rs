use std::{
    fs,
    io::{Read, Write},
};

use rusqlite::params;

use crate::{arguments::models::Todo, data, database};

pub fn export_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let todos = data::sample_todos();
    let json = serde_json::to_string(&todos);

    // write the file to HD
    let mut file = std::fs::File::create("Voido - Todos.json")?;
    file.write_all(json?.as_bytes())?;

    println!(" \n🤖 Todos exported successfully!");

    Ok(())
}

pub fn import_from_json(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the entire file at once (more idiomatic)
    let json = fs::read_to_string(file_path)?;

    println!(" \n🤖 Importing todos from JSON...");
    println!("");
    println!("JSON content:"); // Print the actual content
    println!("");
    println!("{}", json); // Print the actual content
    println!(""); //
    eprint!("‼️ This will replace all existing todos. Continue? [y/N] ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "y" {
        println!("Import cancelled");
        return Ok(());
    } else {
        // Parse the JSON (example with serde)
        let todos: Vec<Todo> = serde_json::from_str(&json)?; // Assuming you have a Todo struct

        // Append the todos to the database
        let mut db = database::DBtodo::new()?;

        // clear existing entries on the table
        let tx = db.connection.transaction()?;
        tx.execute("DELETE FROM todos", params![])?;
        tx.execute("DELETE FROM subtasks", params![])?;
        tx.commit()?;

        for todo in &todos {
            db.add_todo(&todo)?;
        }

        println!(" \n🤖 Todos imported successfully!");
    }

    Ok(())
}
