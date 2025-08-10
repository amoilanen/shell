use std::collections::HashMap;
use std::io::{self, Write};
use std::env;
use std::panic;
use std::fmt::Debug;
use crate::command::ShellCommand;

mod path;
mod command;

fn execute<F, R>(f: F, command_and_args: &[&str]) -> ()
where
  F: Fn(&[&str]) -> R,
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
    let path = path::Path::parse(&env::var("PATH")?)?;
    let builtin_commands: HashMap<&str, ShellCommand> = [
        ("echo", command::ShellCommand::Echo {}),
        ("cd", command::ShellCommand::Cd {}),
        ("pwd", command::ShellCommand::Pwd {}),
        ("exit", command::ShellCommand::Exit {}),
        ("type", command::ShellCommand::Type {
            path: path.clone(),
            builtin_commands: vec!["echo", "cd", "pwd", "exit", "type"].into_iter().map(|c| c.to_string()).collect()
        })
    ].into_iter().collect();

    loop {
        let stdin = io::stdin();
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush()?;
        stdin.read_line(&mut input)?;
        let command_and_args: Vec<&str> = input.split(" ").collect();
        if command_and_args.len() > 0 {
            let command: &str = command_and_args[0].trim();
            if let Some(builtin_command) = builtin_commands.get(command) {
                execute(|command_and_args| builtin_command.run(command_and_args), &command_and_args);
            } else if let Some(found_executable) = path.find_command(command) {
                let command = command::ShellCommand::Exec {
                    executable: found_executable
                };
                execute(|command_and_args| command.run(command_and_args), &command_and_args);
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
