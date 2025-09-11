use std::fs::OpenOptions;
use std::io::Write;
use crate::command::ParsedCommand;

pub(crate) fn run(args: &[&str], parsed_command: &ParsedCommand) -> () {
    let to_output = format!("{}\n", args.join(" "));
    if let Some(stdout_redirect) = &parsed_command.stdout_redirect {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(stdout_redirect.should_append)
            .open(&stdout_redirect.filename)
            .unwrap();
        file.write_all(to_output.as_bytes()).unwrap();
    } else {
        print!("{}", to_output);
    }
    if let Some(stderr_redirect) = &parsed_command.stderr_redirect {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(stderr_redirect.should_append)
            .open(&stderr_redirect.filename)
            .unwrap();
        file.write_all(b"").unwrap();
    }
}