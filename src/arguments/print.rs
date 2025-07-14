use crate::data;

pub fn print_todos() {
    let todos = data::sample_todos();

    for todo in todos {
        println!("ID: {}", todo.id);
        println!("Priority: {}", todo.priority);
        println!("Topic: {}", todo.topic);
        println!("Text: {}", todo.text);
        println!("Date Added: {}", todo.date_added);
        println!("Status: {}", todo.status);
        println!();
    }
}
