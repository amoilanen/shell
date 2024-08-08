use std::io::{self, Write};
use std::process;

fn main() -> Result<(), anyhow::Error> {
    loop {
        let stdin = io::stdin();
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush()?;
        stdin.read_line(&mut input)?;
        let command_and_args: Vec<&str> = input.split_whitespace().collect();
        if command_and_args.len() == 0 {
            continue;
        }
        let command = command_and_args[0].to_string();
        let args: Vec<&str> = command_and_args[1..].to_vec();
        if command == "exit" {
            let exit_code: i32 = if let Some(exit_code) = args.get(0) {
                match exit_code.parse() {
                    Ok(exit_code) => exit_code,
                    Err(_) => -1
                }
            } else {
                -1
            };
            if exit_code >= 0 {
                process::exit(exit_code);
            }
        } else {
            println!("{}: command not found", command.trim());
        }
    }
    //Ok(())
}

//TODO: Add tests
// Unknown command provided
// Known command provided
// Unknown command with arguments provided
// Known command with arguments provided
// Known command is provided but its arguments are incomplete or invalid => command execution fails. REPL loop should still continue
