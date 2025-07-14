use crate::data;

pub fn print_todos() {
    let todos = data::sample_todos();

    println!("Todos: ,{:?} ", todos);

    for todo in todos {
        println!("ID: {}", todo.id);
        println!("Priority: {}", todo.priority);
        println!("Topic: {}", todo.topic);
        println!("Text: {}", todo.text);
        println!("Date Added: {}", todo.date_added);
        println!("Status: {}", todo.status);
        println!("Owner: {}", todo.owner);
        println!("Due Date: {}", todo.due);
        println!("Subtasks: {:?} ", todo.subtasks);
        println!();
    }
}
