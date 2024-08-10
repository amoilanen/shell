use std::io::{self, Write};
use std::process;

//TODO: Handle the case when the command produces an error/or when parsing of the next command is not successful
fn main() -> Result<(), anyhow::Error> {
    loop {
        let stdin = io::stdin();
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush()?;
        stdin.read_line(&mut input)?;
        let command_and_args: Vec<&str> = input.splitn(2, " ").collect();
        if command_and_args.len() == 0 {
            continue;
        }
        let command = command_and_args[0].to_string();
        if command == "exit" {
            let mut exit_code = 0;
            if let Some(args_input) =  command_and_args.get(1) {
                let args: Vec<&str> = args_input.split_whitespace().collect();
                if let Some(exit_code_arg) = args.get(0) {
                    if let Some(code) = exit_code_arg.parse().ok() {
                        exit_code = code;
                    }
                }
            }
            if exit_code >= 0 {
                process::exit(exit_code);
            }
        } else if command == "echo" {
            if let Some(args_input) = command_and_args.get(1) {
                print!("{}", args_input);
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
