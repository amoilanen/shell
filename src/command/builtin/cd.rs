use std::env;
use std::path;

pub(crate) fn run(command_and_args: &[&str]) -> () {
    let home_directory = path::Path::new(&env::var("HOME").unwrap()).to_path_buf();
    let destination_directory = if command_and_args.len() > 1 {
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
            } else if destination_part == "~" && idx == 0 {
                current_directory = home_directory.clone();
            } else {
                current_directory.push(destination_part);
            }
        }
        current_directory
    } else {
        home_directory
    };
    match env::set_current_dir(destination_directory.clone()) {
        Ok(_) => (),
        Err(_) =>
            println!("cd: {}: No such file or directory", destination_directory.to_string_lossy())
    }
}