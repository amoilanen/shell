use std::collections::HashMap;
use std::io::{self, Write};
use std::env;
use std::panic;
use crate::command::{ParsedCommand, ShellCommand};
use crate::input::autocompletion::AutoCompletion;
use crate::input::read_line_with_completion;

mod path;
mod command;
mod input;

fn execute<F>(f: F) -> ()
where
  F: Fn() -> Result<(), anyhow::Error>,
{
    match panic::catch_unwind(panic::AssertUnwindSafe(|| {
        f()
    })) {
        Ok(result) => {
            if let Err(err) = result {
                eprintln!("{}", err);
            }
        },
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

    let autocomplete = AutoCompletion::new(vec!["echo", "cd", "pwd", "exit", "type"]);

    loop {
        print!("\r$ ");
        io::stdout().flush()?;
        let input = read_line_with_completion(&autocomplete)?;
        let parsed_command = ParsedCommand::parse_command(&input)?;
        if let Some(parsed_command) = parsed_command {
            let command = &parsed_command.command;
            if let Some(builtin_command) = builtin_commands.get(command.as_str()) {
                execute(|| builtin_command.run(&parsed_command));
            } else if let Some(found_executable) = path.find_command(command.as_str()) {
                let command = command::ShellCommand::Exec {
                    executable: found_executable
                };
                execute(|| command.run(&parsed_command));
            } else {
                println!("\r{}: command not found", command.trim());
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
