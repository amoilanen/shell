pub(crate) mod cd;
pub(crate) mod exit;
pub(crate) mod echo;
pub(crate) mod pwd;
pub(crate) mod type_;

const BUILTIN_COMMANDS: [&str; 5] = ["echo", "pwd", "type", "cd", "exit"];

pub(crate) fn is_builtin(command: &str) -> bool {
    BUILTIN_COMMANDS.contains(&command)
}

pub(crate) fn generate_output(command: &str, args: &[String]) -> Result<Vec<u8>, anyhow::Error> {
    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    match command {
        "echo" => echo::generate_output(&args_str),
        "pwd" => pwd::generate_output(),
        "type" => type_::generate_output(&args_str),
        "cd" | "exit" => {
            // cd and exit don't make sense in a pipeline, return empty output
            Ok(Vec::new())
        }
        _ => Err(anyhow::anyhow!("Unknown builtin command: {}", command)),
    }
}