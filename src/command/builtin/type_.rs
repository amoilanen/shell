use std::env;
use std::io::{self, Write};
use crate::path;
use crate::command::builtin;

pub(crate) fn generate_output(args: &[&str]) -> Result<Vec<u8>, anyhow::Error> {
    let path = path::Path::parse(&env::var("PATH")?)?;
    if let Some(command_name) = args.get(0) {
        let output = if builtin::is_builtin(command_name) {
            format!("{} is a shell builtin\n", command_name.trim())
        } else {
            if let Some(found_executable) = path.find_command(command_name.trim()) {
                format!("{} is {}\n", command_name.trim(), found_executable)
            } else {
                format!("{}: not found\n", command_name.trim())
            }
        };
        Ok(output.into_bytes())
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn run(args: &[&str]) -> Result<(), anyhow::Error> {
    let output = generate_output(args)?;
    print!("{}", String::from_utf8_lossy(&output));
    io::stdout().flush()?;
    Ok(())
}