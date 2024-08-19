use crate::command::ShellCommand;
use std::env;

pub(crate) struct Cd {}

impl ShellCommand for Cd {
    fn run(&self, command_and_args: &Vec<&str>) -> () {
        let directory = command_and_args[1].trim();
        match env::set_current_dir(&directory) {
            Ok(_) => (),
            Err(_) =>
                println!("cd: {}: No such file or directory", directory)
        }
    }
    fn name(&self) -> String {
        "cd".to_string()
    }
}