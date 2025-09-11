use std::fs;
use crate::command::ParsedCommand;

pub(crate) fn run(args: &[&str], parsed_command: &ParsedCommand) -> () {
    let to_output = format!("{}\n", args.join(" "));
    if let Some(stdout_redirect_filename) = &parsed_command.stdout_redirect_filename {
        fs::write(stdout_redirect_filename, to_output).unwrap();
    } else {
        print!("{}", to_output);
    }
    if let Some(stderr_redirect_filename) = &parsed_command.stderr_redirect_filename {
        fs::write(stderr_redirect_filename, "").unwrap();
    }
}