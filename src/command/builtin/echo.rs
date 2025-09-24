use std::fs::OpenOptions;
use std::io::Write;
use crate::command::ParsedCommand;

pub(crate) fn run(args: &[&str], parsed_command: &ParsedCommand) -> Result<(), anyhow::Error> {
    let to_output = format!("{}\n", args.join(" "));
    if let Some(stdout_redirect) = &parsed_command.stdout_redirect {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(stdout_redirect.should_append)
            .open(&stdout_redirect.filename)
            .map_err(|e| anyhow::anyhow!("Failed to open file '{}': {}", stdout_redirect.filename, e))?;
        file.write_all(to_output.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write to file '{}': {}", stdout_redirect.filename, e))?;
    } else {
        print!("\r{}", to_output);
    }
    if let Some(stderr_redirect) = &parsed_command.stderr_redirect {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(stderr_redirect.should_append)
            .open(&stderr_redirect.filename)
            .map_err(|e| anyhow::anyhow!("Failed to open stderr file '{}': {}", stderr_redirect.filename, e))?;
        file.write_all(b"")
            .map_err(|e| anyhow::anyhow!("Failed to write to stderr file '{}': {}", stderr_redirect.filename, e))?;
    }
    Ok(())
}