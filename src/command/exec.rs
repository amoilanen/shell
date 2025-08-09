use std::process::Command;

pub(crate) fn run(command_and_args: &[&str], executable: &str) -> () {
    let args: Vec<&str> = command_and_args[1..].to_vec().iter().map(|arg| arg.trim()).collect();
    let output = Command::new(executable)
        .args(&args)
        .output()
        .expect(&format!("Failed to execute process {}", executable));
    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        print!("{}", String::from_utf8_lossy(&output.stderr));
    }
}