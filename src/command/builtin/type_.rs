use crate::path;

pub(crate) fn run(args: &[&str], path: &path::Path, builtin_commands: &[&str]) -> Result<(), anyhow::Error> {
    let mut is_shell_builtin = false;
    if let Some(command_name) =  args.get(0) {
        if builtin_commands.iter().find(|c| c.to_string() == command_name.trim().to_string()).is_some() {
            is_shell_builtin = true;
        }
        if is_shell_builtin {
            println!("\r{} is a shell builtin", command_name.trim());
        } else {
            if let Some(found_executable) = path.find_command(command_name.trim()) {
                println!("\r{} is {}", command_name.trim(), found_executable);
            } else {
                println!("\r{}: not found", command_name.trim());
            }
        }
    }
    Ok(())
}