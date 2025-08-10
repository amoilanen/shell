use std::process::Command;
use std::path::Path;

pub(crate) fn run(command_and_args: &[&str], executable: &str) -> () {
    let args: Vec<&str> = command_and_args[1..].to_vec().iter().map(|arg| arg.trim()).collect();

    let path = Path::new(executable);
    let executable_name = path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(executable);
    let executable_dir = path.parent()
        .and_then(|dir| dir.to_str())
        .unwrap_or(".");

    let mut command = Command::new(executable_name);
    command.args(&args);

    if executable_dir != "." {
        command.current_dir(executable_dir);
    }
    
    let output = command
        .output()
        .expect(&format!("Failed to execute process {}", executable));
    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        print!("{}", String::from_utf8_lossy(&output.stderr));
    }
}