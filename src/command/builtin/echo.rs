use std::fs::OpenOptions;
use std::io::{self, Write};
use crate::command::ParsedCommand;

pub(crate) fn generate_output(args: &[&str]) -> Result<Vec<u8>, anyhow::Error> {
    Ok(format!("{}\n", args.join(" ")).into_bytes())
}

pub(crate) fn run(args: &[&str], parsed_command: &ParsedCommand) -> Result<(), anyhow::Error> {
    let to_output = generate_output(args)?;
    if let Some(stdout_redirect) = &parsed_command.stdout_redirect {
        write_to_file(&stdout_redirect.filename, &to_output, stdout_redirect.should_append)?;
    } else {
        print!("{}", String::from_utf8_lossy(&to_output));
        io::stdout().flush()?;
    }
    if let Some(stderr_redirect) = &parsed_command.stderr_redirect {
        write_to_file(&stderr_redirect.filename, b"", stderr_redirect.should_append)?;
    }
    Ok(())
}

fn write_to_file(filename: &str, content: &[u8], should_append: bool) -> Result<(), anyhow::Error> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(should_append)
        .open(filename)
        .map_err(|e| anyhow::anyhow!("Failed to open stderr file '{}': {}", filename, e))?;
    file.write_all(content)
        .map_err(|e| anyhow::anyhow!("Failed to write to stderr file '{}': {}", filename, e))?;
    Ok(())
}