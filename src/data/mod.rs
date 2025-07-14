use crate::{arguments::models::Todo, database::DBtodo};

pub fn sample_todos() -> Vec<Todo> {
    let db = DBtodo::new().unwrap();

    let todos = db.get_todos().unwrap().iter().cloned().collect();
    todos
}
