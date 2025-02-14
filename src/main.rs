use std::io::{self, Write};
use std::env;
use std::panic;
use std::fmt::Debug;
use crate::command::builtin;
use crate::command::ShellCommand;

mod path;
mod command;

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
    let path = path::Path::parse(&env::var("PATH")?)?;
    let builtin_commands: Vec<Box<dyn ShellCommand>> = vec![
        Box::new(builtin::echo::Echo {}),
        Box::new(builtin::exit::Exit {}),
        Box::new(builtin::pwd::Pwd {}),
        Box::new(builtin::cd::Cd {}),
        Box::new(builtin::type_::Type {
            path: path.clone(),
            builtin_commands: vec!["echo", "cd", "pwd", "exit", "type"].into_iter().map(|c| c.to_string()).collect()
        })
    ];

    loop {
        let stdin = io::stdin();
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush()?;
        stdin.read_line(&mut input)?;
        let command_and_args: Vec<&str> = input.splitn(2, " ").collect();
        if command_and_args.len() > 0 {
            let command: &str = command_and_args[0].trim();
            let found_builtin_command = builtin_commands.iter().find(|c| {
                c.name() == command.to_string()
            });
            if let Some(builtin_command) = found_builtin_command {
                execute(|command_and_args| builtin_command.run(command_and_args), &command_and_args);
            } else if let Some(found_executable) = path.find_command(command) {
                let command = command::exec::Exec {
                    found_executable
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
