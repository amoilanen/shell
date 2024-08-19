use crate::command::ShellCommand;
use std::env;

pub(crate) struct Pwd {}

impl ShellCommand for Pwd {
    fn run(&self, command_and_args: &Vec<&str>) -> () {
        let current_directory = env::current_dir().unwrap();
        println!("{}", current_directory.to_str().unwrap());
    }
    fn name(&self) -> String {
        "pwd".to_string()
    }
}