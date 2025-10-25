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
    let builtin_commands: HashMap<&str, ShellCommand> = [
        ("echo", command::ShellCommand::Echo {}),
        ("cd", command::ShellCommand::Cd {}),
        ("pwd", command::ShellCommand::Pwd {}),
        ("exit", command::ShellCommand::Exit {}),
        ("type", command::ShellCommand::Type {})
    ].into_iter().collect();

    let path = path::Path::parse(&env::var("PATH")?)?;
    let automcomplete_path = path.clone();
    let autocomplete = AutoCompletion::new_with_dynamic_completion(
        vec!["echo", "cd", "pwd", "exit", "type"],
        Box::new(move |partial: &str| automcomplete_path.find_matching_executables(partial))
    );

    loop {
        print!("\r$ ");
        io::stdout().flush()?;
        let input = read_line_with_completion(&autocomplete)?;
        let parsed_command = ParsedCommand::parse_command(&input)?;
        if let Some(mut parsed_command) = parsed_command {
            if let Err(cmd_name) = path.resolve_piped_commands(&mut parsed_command) {
                println!("\r{}: command not found", cmd_name.trim());
                continue;
            }

            let command = &parsed_command.command;
            if parsed_command.piped_command.is_some() {
                let command = command::ShellCommand::Exec;
                execute(|| command.run(&parsed_command));
            } else if let Some(builtin_command) = builtin_commands.get(command.as_str()) {
                execute(|| builtin_command.run(&parsed_command));
            } else if let Some(_found_executable) = path.find_command(command.as_str()) {
                let command = command::ShellCommand::Exec;
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
