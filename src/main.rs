use std::io::{self, Write};

fn main() -> Result<(), anyhow::Error> {
    let registered_commands: Vec<String> = Vec::new();

    loop {
        let stdin = io::stdin();
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush()?;
        stdin.read_line(&mut input)?;
        if !registered_commands.contains(&input) {
            println!("{}: command not found", input.trim());
        }
    }
    //Ok(())
}

