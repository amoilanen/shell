use std::io::{self, Write};

fn main() -> Result<(), anyhow::Error> {
    print!("$ ");
    io::stdout().flush()?;

    let registered_commands: Vec<String> = Vec::new();

    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)?;
    if !registered_commands.contains(&input) {
        println!("{}: command not found", input.trim());
    }
    Ok(())
}

