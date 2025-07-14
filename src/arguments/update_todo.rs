use std::error::Error;

use crate::database::DBtodo;

pub fn update_todo(id: i32, status: String) -> Result<(), Box<dyn Error>> {
    let db = DBtodo::new()?;

    let status = Some(status);

    db.update_todo(id, status)
}
