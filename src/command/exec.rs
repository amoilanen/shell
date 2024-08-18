use crate::command::ShellCommand;
use std::process::Command;

pub(crate) struct Exec {
    pub(crate) found_executable: String
}

impl ShellCommand for Exec {
    fn run(&self, command_and_args: &Vec<&str>) -> () {
        let args: Vec<&str> = command_and_args[1..].to_vec().iter().map(|arg| arg.trim()).collect();
        let output = Command::new(self.found_executable.clone())
            .args(&args)
            .output()
            .expect(&format!("Failed to execute process {}", self.found_executable));
        if output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            print!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }
    fn name(&self) -> String {
        "exec".to_string()
    }
}