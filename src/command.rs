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
            let mut inside_double_quotes = false;
            let mut current_arg = String::new();
            let mut is_escaped_character = false;
            for ch in all_args.chars() {
                if inside_single_quotes {
                    if ch == '\'' {
                        inside_single_quotes = false;
                    } else {
                        current_arg.push(ch);
                    }
                } else if inside_double_quotes {
                    if is_escaped_character {
                        if ch == '\"' || ch == '\\' || ch == '$' || ch == '`' {
                            current_arg.push(ch);
                        } else if ch == 'n' {
                            current_arg.push('\n');
                        } else {
                            // If it's not a recognized escape sequence, treat the backslash as literal
                            current_arg.push('\\');
                            current_arg.push(ch);
                        }
                        is_escaped_character = false;
                    } else if ch == '"' {
                        inside_double_quotes = false;
                    } else if ch == '\\' {
                        is_escaped_character = true;
                    } else {
                        current_arg.push(ch);
                    }
                } else {
                    if is_escaped_character {
                        is_escaped_character = false;
                        current_arg.push(ch)
                    } else if ch == '\\' {
                        is_escaped_character = true;
                    } else if ch == '\'' {
                        inside_single_quotes = true;
                    } else if ch == '"' {
                        inside_double_quotes = true;
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
        assert_eq!(result, Some(cmd("echo", vec![])));
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
    fn test_parse_quotes_next_to_each_other() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo 'example     shell' 'hello''test' script''world")?;
        assert_eq!(result, Some(cmd("echo", vec!["example     shell", "hellotest", "scriptworld"])));
        Ok(())
    }

    #[test]
    fn test_get_args() {
        let command = cmd("test", vec!["arg1", "arg2"]);
        let args = command.get_args();
        assert_eq!(args, vec!["arg1", "arg2"]);
    }

    #[test]
    fn test_parse_echo_with_double_quoted_strings() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"quz  hello\"  \"bar\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["quz  hello", "bar"])));
        Ok(())
    }

    #[test]
    fn test_parse_echo_with_double_quoted_strings_containing_single_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"bar\"  \"shell's\"  \"foo\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["bar", "shell's", "foo"])));
        Ok(())
    }

    #[test]
    fn test_parse_cat_with_double_quoted_file_paths() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("cat \"/tmp/file name\" \"/tmp/'file name' with spaces\"")?;
        assert_eq!(result, Some(cmd("cat", vec!["/tmp/file name", "/tmp/'file name' with spaces"])));
        Ok(())
    }

    #[test]
    fn test_parse_empty_double_quoted_string() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"\"")?;
        assert_eq!(result, Some(cmd("echo", vec![])));
        Ok(())
    }

    #[test]
    fn test_parse_mixed_single_and_double_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo 'single quoted' \"double quoted\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["single quoted", "double quoted"])));
        Ok(())
    }

    #[test]
    fn test_parse_double_quotes_next_to_each_other() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"hello\"\"world\" \"test\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["helloworld", "test"])));
        Ok(())
    }

    #[test]
    fn test_parse_backslash_before_blank_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"before\\   after\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["before\\   after"])));
        Ok(())
    }

    #[test]
    fn test_parse_multiple_backslashes_before_blanks() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo world\\ \\ \\ \\ \\ \\ script")?;
        assert_eq!(result, Some(cmd("echo", vec!["world      script"])));
        Ok(())
    }

    #[test]
    fn test_parse_multiple_backslashes_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("cat \"/tmp/file\\\\name\" \"/tmp/file\\ name\"")?;
        assert_eq!(result, Some(cmd("cat", vec!["/tmp/file\\name", "/tmp/file\\ name"])));
        Ok(())
    }

    #[test]
    fn test_parse_escape_sequences_in_double_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("cat \"/tmp/quz/f\\n36\" \"/tmp/quz/f\\t12\" \"/tmp/quz/f\\'52\"")?;
        assert_eq!(result, Some(cmd("cat", vec!["/tmp/quz/f\n36", "/tmp/quz/f\\t12", "/tmp/quz/f\\'52"])));
        Ok(())
    }

    #[test]
    fn test_parse_escape_sequences_outside_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo world\\nshell")?;
        assert_eq!(result, Some(cmd("echo", vec!["worldnshell"])));
        Ok(())
    }

    #[test]
    fn test_parse_simple_escaped_quote() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \\'hello\\'") ?;
        assert_eq!(result, Some(cmd("echo", vec!["'hello'"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_quotes_with_space() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \\'\\\"script shell\\\"\\'") ?;
        assert_eq!(result, Some(cmd("echo", vec!["'\"script", "shell\"'"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_double_quote_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"hello\\\"world\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello\"world"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_backslash_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"hello\\\\world\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello\\world"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_dollar_sign_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"hello\\$world\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello$world"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_backtick_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"hello\\`world\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello`world"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_newline_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"hello\\nworld\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello\nworld"])));
        Ok(())
    }

    #[test]
    fn test_parse_multiple_escaped_characters_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"\\\"hello\\\" \\$world \\`test\\` \\\\backslash\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["\"hello\" $world `test` \\backslash"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_characters_at_beginning_and_end() -> Result<(), anyhow::Error> {
        let result = CommandWithArgs::parse_command("echo \"\\\"start\\\" and \\\"end\\\"\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["\"start\" and \"end\""])));
        Ok(())
    }
}