use crate::command::ShellCommand;
use std::env;
use std::path;

pub(crate) struct Cd {}

impl ShellCommand for Cd {
    fn run(&self, command_and_args: &Vec<&str>) -> () {
        //TODO: If len == 0, then navigate to the home directory
        if command_and_args.len() > 0 {
            let destination = command_and_args[1].trim().to_string();
            let destination_parts = destination.split("/");
            let mut current_directory = env::current_dir().unwrap();
    
            for (idx, destination_part) in destination_parts.into_iter().enumerate() {
                if destination_part == "." {
                    //Do nothing, already in the correct directory (current directory)
                } else if destination_part == ".." {
                    current_directory.pop();
                } else if destination_part == "" && idx == 0 {
                    current_directory = path::Path::new("/").to_path_buf();
                } else {
                    current_directory.push(destination_part);
                }
            };
            match env::set_current_dir(current_directory.clone()) {
                Ok(_) => (),
                Err(_) =>
                    println!("cd: {}: No such file or directory", current_directory.to_string_lossy())
            }
        }
    }
    fn name(&self) -> String {
        "cd".to_string()
    }
}