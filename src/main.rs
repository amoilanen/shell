use std::io::{self, Write};
use std::panic;
use std::fmt::Debug;

mod builtin;

fn execute<F, R>(f: F, command_and_args: &Vec<&str>) -> ()
where
  F: Fn(&Vec<&str>) -> R,
  R: Debug
{
    match panic::catch_unwind(panic::AssertUnwindSafe(|| {
        f(command_and_args)
    })) {
        Ok(_) => (),
        Err(err) => {
            println!("{:?}", err);
            ()
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    loop {
        let stdin = io::stdin();
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush()?;
        stdin.read_line(&mut input)?;
        let command_and_args: Vec<&str> = input.splitn(2, " ").collect();
        if command_and_args.len() > 0 {
            let command = command_and_args[0].to_string();
            if command == "exit" {
                execute(builtin::exit::command,&command_and_args);
            } else if command == "echo" {
                execute(builtin::echo::command, &command_and_args);
            } else {
                println!("{}: command not found", command.trim());
            }
        }
    }
}

//TODO: Add tests
// Unknown command provided
// Known command provided
// Unknown command with arguments provided
// Known command with arguments provided
// Known command is provided but its arguments are incomplete or invalid => command execution fails. REPL loop should still continue
