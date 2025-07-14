use colored::*;

pub fn print_args() {
    println!("{}\n", "Usage: voido [options]".bright_blue().underline());

    println!("{}\n", "Options:".yellow().bold());

    println!("{}\n", "-h, --help".yellow().bold());

    println!("    {}\n", "Print this help message".white());

    println!("{}\n", "-a, --add".yellow().bold());

    println!("    {}\n", "Add a new todo".white());

    println!(
        "    {}: {}",
        "Example:".bright_black(),
        " --add \"Buy groceries\"".italic()
    );

    println!("{}\n", "-r, --remove".yellow().bold());

    println!("    {}\n", "Remove a todo by ID".white());

    println!(
        "    {}: {}",
        "Example:".bright_black(),
        "voido --remove 1".italic()
    );

    println!("{}\n", "-u, --update".yellow().bold());

    println!("    {}\n", "Update a todo by ID".white());

    println!(
        "    {}: {}",
        "Example:".bright_black(),
        "voido --update 1 --name \"New Task Name\"".italic()
    );

    println!("{}\n", "-d, --done".yellow().bold());

    println!("    {}\n", "Mark a todo as done by ID".white());

    println!(
        "    {}: {}",
        "Example:".bright_black(),
        "voido --done 1".italic()
    );

    println!("{}\n", "-p, --print".yellow().bold());

    println!("    {}\n", "Print all todos".white());

    println!(
        "    {}: {}",
        "Example:".bright_black(),
        "voido --print".italic()
    );
}
