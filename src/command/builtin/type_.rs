use crate::path;

pub(crate) fn run(command_and_args: &[&str], path: &path::Path, builtin_commands: &[&str]) -> () {
    let mut is_shell_builtin = false;
    if let Some(command_name) =  command_and_args.get(1) {
        if builtin_commands.iter().find(|c| c.to_string() == command_name.trim().to_string()).is_some() {
            is_shell_builtin = true;
        }
        if is_shell_builtin {
            println!("{} is a shell builtin", command_name.trim());
        } else {
            if let Some(found_executable) = path.find_command(command_name.trim()) {
                println!("{} is {}", command_name.trim(), found_executable);
            } else {
                println!("{}: not found", command_name.trim());
            }
        }
    }
}