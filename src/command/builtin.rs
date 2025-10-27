use std::collections::HashMap;
use crate::{command::{self, ShellCommand}, history::History};
use lazy_static::lazy_static;

pub(crate) mod cd;
pub(crate) mod exit;
pub(crate) mod echo;
pub(crate) mod pwd;
pub(crate) mod type_;
pub(crate) mod history;

lazy_static! {
    pub(crate) static ref BUILTIN_COMMANDS: HashMap<&'static str, ShellCommand> = {
        let mut m = HashMap::new();
        m.insert("echo", command::ShellCommand::Echo {});
        m.insert("cd", command::ShellCommand::Cd {});
        m.insert("pwd", command::ShellCommand::Pwd {});
        m.insert("history", command::ShellCommand::History {});
        m.insert("exit", command::ShellCommand::Exit {});
        m.insert("type", command::ShellCommand::Type {});
        m
    };
}

pub(crate) fn is_builtin(command: &str) -> bool {
    BUILTIN_COMMANDS.keys().any(|key| key == &command)
}

pub(crate) fn generate_output(command: &str, args: &[String], history: &History) -> Result<Vec<u8>, anyhow::Error> {
    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    match command {
        "echo" => echo::generate_output(&args_str),
        "pwd" => pwd::generate_output(),
        "type" => type_::generate_output(&args_str),
        "history" => history::generate_output(&args_str, history),
        "cd" | "exit" => {
            // cd and exit don't make sense in a pipeline, return empty output
            Ok(Vec::new())
        }
        _ => Err(anyhow::anyhow!("Unknown builtin command: {}", command)),
    }
}