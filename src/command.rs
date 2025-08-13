use crate::path;

pub mod builtin;
pub mod exec;

pub(crate) enum ShellCommand {
    Cd,
    Echo,
    Exit,
    Pwd,
    Type { path: path::Path, builtin_commands: Vec<String> },
    Exec { executable: String }
}

impl ShellCommand {

    pub(crate) fn run(&self, args: &[&str]) -> () {
        match self {
            ShellCommand::Cd => builtin::cd::run(args),
            ShellCommand::Echo => builtin::echo::run(args),
            ShellCommand::Exec { executable } => exec::run(args, executable),
            ShellCommand::Exit => builtin::exit::run(args),
            ShellCommand::Pwd => builtin::pwd::run(args),
            ShellCommand::Type { path, builtin_commands } =>
                builtin::type_::run(args, path, builtin_commands.iter().map(|c| c.as_str()).collect::<Vec<&str>>() .as_slice()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct CommandWithArgs {
    pub(crate) command: String,
    pub(crate) args: Vec<String>
}

impl CommandWithArgs {
    pub(crate) fn parse_command(input: &str) -> Result<Option<CommandWithArgs>, anyhow::Error> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(None);
        }
        
        let command_and_args: Vec<&str> = input.splitn(2," ").collect();
        let command = command_and_args.get(0).ok_or(anyhow::anyhow!("No command provided"))?.to_string();
        if command_and_args.len() >= 2 {
            let all_args = command_and_args.get(1).ok_or(anyhow::anyhow!("No arguments provided"))?.to_string();
            let mut arguments = Vec::new();
            let mut inside_single_quotes = false;
            let mut current_arg = String::new();
            for ch in all_args.chars() {
                if inside_single_quotes {
                    if ch == '\'' {
                        inside_single_quotes = false;
                        arguments.push(current_arg.clone());
                        current_arg = String::new();
                    } else {
                        current_arg.push(ch);
                    }
                } else {
                    if ch == '\'' {
                        inside_single_quotes = true;
                    } else if ch == ' ' {
                        if current_arg.len() > 0 {
                            arguments.push(current_arg.clone());
                            current_arg = String::new();
                        }
                    } else {
                        current_arg.push(ch);
                    }
                }
            }
            if current_arg.len() > 0 {
                arguments.push(current_arg);
            }
            Ok(Some(CommandWithArgs { command, args: arguments }))
        } else {
            Ok(Some(CommandWithArgs { command, args: vec![] }))
        }
    }

    pub(crate) fn get_args(&self) -> Vec<&str> {
        self.args.iter().map(|arg| arg.as_str()).collect::<Vec<&str>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(command: &str, args: Vec<&str>) -> CommandWithArgs {
        CommandWithArgs {
            command: command.to_string(),
            args: args.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_parse_empty_command() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("")?;
        assert!(result.is_none(), "Empty command should return None");
        Ok(())
    }

    #[test]
    fn test_parse_whitespace_only_command() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("   \t\n   ")?;
        assert!(result.is_none(), "Whitespace-only command should return None");
        Ok(())
    }

    #[test]
    fn test_parse_simple_command_no_args() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("pwd")?;
        assert_eq!(result, Some(cmd("pwd", vec![])));
        Ok(())
    }

    #[test]
    fn test_parse_echo_with_number() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo 123")?;
        assert_eq!(result, Some(cmd("echo", vec!["123"])));
        Ok(())
    }

    #[test]
    fn test_parse_echo_with_multiple_args() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo hello world")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello", "world"])));
        Ok(())
    }

    #[test]
    fn test_parse_echo_with_single_quoted_string() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo 'hello    world'")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello    world"])));
        Ok(())
    }

    #[test]
    fn test_parse_cat_with_quoted_file_paths() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("cat '/tmp/file name' '/tmp/file name with spaces'")?;
        assert_eq!(result, Some(cmd("cat", vec!["/tmp/file name", "/tmp/file name with spaces"])));
        Ok(())
    }

    #[test]
    fn test_parse_mixed_quoted_and_unquoted_args() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("cp 'file with spaces.txt' /tmp/destination")?;
        assert_eq!(result, Some(cmd("cp", vec!["file with spaces.txt", "/tmp/destination"])));
        Ok(())
    }

    #[test]
    fn test_parse_multiple_spaces_between_args() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo   hello    world   ")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello", "world"])));
        Ok(())
    }

    #[test]
    fn test_parse_empty_quoted_string() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo ''")?;
        assert_eq!(result, Some(cmd("echo", vec![""])));
        Ok(())
    }

    #[test]
    fn test_parse_single_quote_in_middle() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo don't")?;
        assert_eq!(result, Some(cmd("echo", vec!["dont"])));
        Ok(())
    }

    #[test]
    fn test_parse_command_with_leading_trailing_spaces() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("  echo hello  ")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello"])));
        Ok(())
    }

    #[test]
    fn test_parse_complex_command_line() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("rsync -av 'source dir/' '/dest/path with spaces/' --exclude='*.tmp'")?;
        assert_eq!(result, Some(cmd("rsync", vec!["-av", "source dir/", "/dest/path with spaces/", "--exclude=*.tmp"])));
        Ok(())
    }

    #[test]
    fn test_get_args() {
        let command = cmd("test", vec!["arg1", "arg2"]);
        let args = command.get_args();
        assert_eq!(args, vec!["arg1", "arg2"]);
    }
}