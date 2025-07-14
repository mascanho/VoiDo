use std::error::Error;

use crate::database::DBtodo;

pub fn remove_todo(id: i32) -> Result<(), Box<dyn Error>> {
    let db = DBtodo::new()?;

    db.delete_todo(id)
}

pub fn clear_todos() -> Result<(), Box<dyn Error>> {
    let db = DBtodo::new()?;

    db.clear_all_todos()
}
