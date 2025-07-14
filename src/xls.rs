use std::path::Path;

use crate::{arguments::models::Todo, database::DBtodo};
use calamine::{Data, DataType, Reader, Xlsx, open_workbook};
use xlsxwriter::*;

pub fn export_todos() -> Result<(), XlsxError> {
    // Initialize database and get todos
    let db = DBtodo::new().expect("Failed to initialize database");
    let todos = db.get_todos().expect("Failed to get todos");

    // Create workbook
    let mut workbook = Workbook::new("VoiDo - Todos Export.xlsx")?;
    let mut worksheet = workbook.add_worksheet(None)?;

    // Write headers
    let headers = [
        "ID",
        "PRIORITY",
        "TOPIC",
        "TODO",
        "DESCRIPTION",
        "CREATED",
        "DUE DATE",
        "STATUS",
        "OWNER",
    ];

    for (col_num, header) in headers.iter().enumerate() {
        worksheet.write_string(0, col_num as u16, header, None)?;
    }

    // Write data
    for (row_num, todo) in todos.iter().enumerate() {
        let row = row_num as u32 + 1;

        // Handle potential Option fields with unwrap_or_default()
        worksheet
            .write_number(row, 0, todo.id as f64, None)
            .expect("Failed to write ID");
        worksheet.write_string(row, 1, &todo.priority.to_string(), None)?;
        worksheet.write_string(row, 2, &todo.topic, None)?;
        worksheet.write_string(row, 3, &todo.text, None)?;
        worksheet.write_string(row, 4, &todo.desc, None)?;
        worksheet.write_string(row, 5, &todo.date_added.to_string(), None)?;
        worksheet.write_string(row, 6, &todo.due.to_string(), None)?;
        worksheet.write_string(row, 7, &todo.status.to_string(), None)?;
        worksheet.write_string(row, 8, &todo.owner, None)?;
    }

    workbook.close()?;
    println!("");
    println!("🤖 Todos exported to VoiDo - Todos Export.xlsx");
    println!("");
    Ok(())
}

pub fn import_todos(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Open the Excel file
    let path = Path::new(file_path);
    let mut workbook: Xlsx<_> = open_workbook(path)?;

    // Get the first worksheet
    let range = workbook
        .worksheet_range_at(0)
        .ok_or("No worksheet found")??;

    // Connect to the database
    let db = DBtodo::new()?;

    // Process each row (skip header if present)
    for row in range.rows().skip(1) {
        // Skip empty rows
        if row.is_empty() {
            continue;
        }

        // Parse each cell directly
        let id = match &row[0] {
            Data::String(s) => s.to_string(),
            Data::Float(f) => f.to_string(),
            Data::Int(i) => i.to_string(),
            _ => "0".to_string(),
        };

        let priority = match &row[1] {
            Data::String(s) => s.to_string(),
            _ => "".to_string(),
        };

        let topic = match &row[2] {
            Data::String(s) => s.to_string(),
            _ => "".to_string(),
        };

        let text = match &row[3] {
            Data::String(s) => s.to_string(),
            _ => "".to_string(),
        };

        let desc = match &row[4] {
            Data::String(s) => s.to_string(),
            _ => "".to_string(),
        };

        let date_added = match &row[5] {
            Data::String(s) => s.to_string(),
            Data::DateTime(d) => d.to_string(),
            _ => "".to_string(),
        };

        let due = match &row[6] {
            Data::String(s) => s.to_string(),
            Data::DateTime(d) => d.to_string(),
            _ => "".to_string(),
        };

        let status = match &row[7] {
            Data::String(s) => s.to_string(),
            _ => "Pending".to_string(),
        };

        let owner = match &row[8] {
            Data::String(s) => s.to_string(),
            _ => "".to_string(),
        };

        // Create and insert Todo
        let todo = Todo {
            id: id.parse().unwrap_or(0),
            priority,
            topic,
            text,
            desc,
            date_added,
            due,
            status,
            owner,
        };

        db.add_todo(&todo)?;
    }

    println!("");
    println!("🤖 Todos imported from {}", file_path);
    println!("");

    Ok(())
}
