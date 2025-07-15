use std::{io, path::Path};

use crate::{
    arguments::models::{Subtask, Todo},
    database::DBtodo,
};
use calamine::{Data, DataType, Reader, Xlsx, open_workbook};
use rusqlite::params;
use xlsxwriter::*;

pub fn export_todos() -> Result<(), XlsxError> {
    let db = DBtodo::new().expect("Failed to initialize database");
    let todos = db.get_todos().expect("Failed to get todos");

    // Determine maximum number of subtasks
    let max_subtasks = todos
        .iter()
        .map(|todo| todo.subtasks.len())
        .max()
        .unwrap_or(0);

    // Create workbook
    let mut workbook = Workbook::new("VoiDo - Todos Export.xlsx")?;
    let mut worksheet = workbook.add_worksheet(None)?;

    // Write headers - using owned Strings
    let mut headers = vec![
        "ID".to_string(),
        "PRIORITY".to_string(),
        "TOPIC".to_string(),
        "TODO".to_string(),
        "DESCRIPTION".to_string(),
        "CREATED".to_string(),
        "DUE DATE".to_string(),
        "STATUS".to_string(),
        "OWNER".to_string(),
    ];

    // Add generic subtask headers - using owned Strings
    for i in 1..=max_subtasks {
        headers.push(format!("Subtask {}", i));
    }

    // Write headers to worksheet
    for (col_num, header) in headers.iter().enumerate() {
        worksheet.write_string(0, col_num as u16, header, None)?;
    }

    // Helper functions that return owned Strings
    fn get_value(value: impl AsRef<str>) -> String {
        let s = value.as_ref();
        if s.is_empty() {
            "".to_string()
        } else {
            s.to_string()
        }
    }

    fn get_due_date(value: impl AsRef<str>) -> String {
        let s = value.as_ref();
        if s.is_empty() {
            "-".to_string()
        } else {
            s.to_string()
        }
    }

    // Write data
    for (row_num, todo) in todos.iter().enumerate() {
        let row = row_num as u32 + 1;

        worksheet.write_number(row, 0, todo.id as f64, None)?;
        worksheet.write_string(row, 1, &get_value(&todo.priority), None)?;
        worksheet.write_string(row, 2, &get_value(&todo.topic), None)?;
        worksheet.write_string(row, 3, &get_value(&todo.text), None)?;
        worksheet.write_string(row, 4, &get_value(&todo.desc), None)?;
        worksheet.write_string(row, 5, &get_value(&todo.date_added), None)?;
        worksheet.write_string(row, 6, &get_due_date(&todo.due), None)?;
        worksheet.write_string(row, 7, &get_value(&todo.status), None)?;
        worksheet.write_string(row, 8, &get_value(&todo.owner), None)?;

        // Write subtasks
        for (col_offset, subtask) in todo.subtasks.iter().enumerate() {
            worksheet.write_string(row, 9 + col_offset as u16, &get_value(&subtask.text), None)?;
        }
    }

    workbook.close()?;
    println!("\n🤖 Todos exported to VoiDo - Todos Export.xlsx\n");
    Ok(())
}
// TODO: Add support for Appending TODOS to the existing ones in the DB
// IMPORT TODOs
pub fn import_todos(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Open the Excel file
    let path = Path::new(file_path);
    let mut workbook: Xlsx<_> = open_workbook(path)?;

    // Get the first worksheet
    let range = workbook
        .worksheet_range_at(0)
        .ok_or("No worksheet found")??;

    // Connect to the database (make mutable)
    let mut db = DBtodo::new()?;

    // Clear existing todos (like flush_db but with confirmation)
    println!("⚠️ This will delete all existing todos. Continue? [y/N]");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "y" {
        println!("Import cancelled");
        return Ok(());
    }

    // Start transaction for bulk import
    let tx = db.connection.transaction()?;

    // Clear existing data
    tx.execute("DELETE FROM subtasks", params![])?;
    tx.execute("DELETE FROM todos", params![])?;

    // Process each row (skip header row)
    for (row_num, row) in range.rows().skip(1).enumerate() {
        // Skip empty rows
        if row.is_empty() {
            continue;
        }

        // Helper function to parse cell values
        fn parse_cell(cell: &Data) -> String {
            match cell {
                Data::String(s) => s.trim().to_string(),
                Data::Float(f) => f.to_string(),
                Data::Int(i) => i.to_string(),
                Data::DateTime(d) => d.to_string(),
                _ => String::new(),
            }
        }

        // Parse main todo fields
        let id = (row_num + 1) as i32; // Generate sequential IDs
        let priority = parse_cell(&row[1]);
        let topic = parse_cell(&row[2]);
        let text = parse_cell(&row[3]);
        let desc = parse_cell(&row[4]);
        let date_added = parse_cell(&row[5]);
        let due = parse_cell(&row[6]);
        let status = parse_cell(&row[7]);
        let owner = parse_cell(&row[8]);

        // Insert todo
        tx.execute(
            "INSERT INTO todos (id, priority, topic, text, desc, date_added, due, status, owner) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id, priority, topic, text, desc, date_added, due, status, owner
            ],
        )?;

        // Parse and insert subtasks (columns 9+)
        for (subtask_num, cell) in row.iter().skip(9).enumerate() {
            let text = parse_cell(cell);
            if !text.is_empty() {
                tx.execute(
                    "INSERT INTO subtasks (todo_id, text, status) 
                     VALUES (?1, ?2, ?3)",
                    params![id, text, "Pending"], // Default status
                )?;
            }
        }
    }

    // Commit the transaction
    tx.commit()?;

    println!("\n✅ Todos imported successfully from {}", file_path);
    println!("   Total todos imported: {}", range.rows().count() - 1); // Subtract header row

    Ok(())
}
