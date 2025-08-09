use crate::path;

pub mod builtin;
pub mod exec;

pub(crate) enum ShellCommand {
    Cd,
    Echo,
    Exit,
    Pwd,
    Type { path: path::Path, builtin_commands: Vec<String> },
    Exec { executable: String }
}

impl ShellCommand {

    pub(crate) fn run(&self, command_and_args: &[&str]) -> () {
        match self {
            ShellCommand::Cd => builtin::cd::run(command_and_args),
            ShellCommand::Echo => builtin::echo::run(command_and_args),
            ShellCommand::Exec { executable } => exec::run(command_and_args, executable),
            ShellCommand::Exit => builtin::exit::run(command_and_args),
            ShellCommand::Pwd => builtin::pwd::run(command_and_args),
            ShellCommand::Type { path, builtin_commands } =>
                builtin::type_::run(command_and_args, path, builtin_commands.iter().map(|c| c.as_str()).collect::<Vec<&str>>() .as_slice()),
        }
    }
}