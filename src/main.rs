use std::collections::HashMap;
use std::io::{self, Write};
use std::env;
use std::panic;
use std::fmt::Debug;
use crate::command::{ParsedCommand, ShellCommand};

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
        let parsed_command = ParsedCommand::parse_command(&input)?;
        if let Some(parsed_command) = parsed_command {
            let command = &parsed_command.command;
            let args = &parsed_command.get_args();
            if let Some(builtin_command) = builtin_commands.get(command.as_str()) {
                execute(|command_and_args| builtin_command.run(command_and_args, parsed_command.stdout_redirect_filename.as_deref()), &args);
            } else if let Some(found_executable) = path.find_command(command.as_str()) {
                let command = command::ShellCommand::Exec {
                    executable: found_executable
                };
                execute(|command_and_args| command.run(command_and_args, parsed_command.stdout_redirect_filename.as_deref()), &args);
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
