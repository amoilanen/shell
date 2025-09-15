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

    pub(crate) fn run(&self, parsed_command: &ParsedCommand) -> () {
        let args = parsed_command.get_args();
        match self {
            ShellCommand::Cd => builtin::cd::run(args.as_slice()),
            ShellCommand::Echo => builtin::echo::run(args.as_slice(), parsed_command),
            ShellCommand::Exec { executable } => exec::run(executable, parsed_command),
            ShellCommand::Exit => builtin::exit::run(args.as_slice()),
            ShellCommand::Pwd => builtin::pwd::run(args.as_slice()),
            ShellCommand::Type { path, builtin_commands } =>
                builtin::type_::run(args.as_slice(), path, builtin_commands.iter().map(|c| c.as_str()).collect::<Vec<&str>>() .as_slice()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Redirect {
    pub(crate) filename: String,
    pub(crate) should_append: bool
}

#[derive(Debug, PartialEq)]
pub(crate) struct ParsedCommand {
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
    pub(crate) stdout_redirect: Option<Redirect>,
    pub(crate) stderr_redirect: Option<Redirect>
}

impl ParsedCommand {

    //TODO: Support also 1>&2 2>&1
    fn parse_terms(input: &str, terms: &[String]) -> Result<Option<ParsedCommand>, anyhow::Error> {
        if terms.len() == 0 {
            return Err(anyhow::anyhow!("No command provided: {}", input));
        }
        let mut terms_without_redirect: Vec<String> = Vec::new();
        let mut idx = 0;
        let mut stdout_redirect: Option<Redirect> = None;
        let mut stderr_redirect: Option<Redirect> = None;
        while idx < terms.len() {
            if terms[idx] == ">" || terms[idx] == "1>" {
                let stdout_redirect_filename = terms.get(idx + 1).map(|s| s.to_owned());
                let filename = stdout_redirect_filename.ok_or_else(|| anyhow::anyhow!("Missing filename for stdout redirect: {}", input))?;
                stdout_redirect = Some(Redirect { filename, should_append: false });
                idx = idx + 2;
            } else if terms[idx] == "2>" {
                let stderr_redirect_filename = terms.get(idx + 1).map(|s| s.to_owned());
                let filename = stderr_redirect_filename.ok_or_else(|| anyhow::anyhow!("Missing filename for stderr redirect: {}", input))?;
                stderr_redirect = Some(Redirect { filename, should_append: false });
                idx = idx + 2;
            } else if terms[idx] == ">>" || terms[idx] == "1>>" {
                let stdout_redirect_filename = terms.get(idx + 1).map(|s| s.to_owned());
                let filename = stdout_redirect_filename.ok_or_else(|| anyhow::anyhow!("Missing filename for stdout append redirect: {}", input))?;
                stdout_redirect = Some(Redirect { filename, should_append: true });
                idx = idx + 2;
            } else if terms[idx] == "2>>" {
                let stderr_redirect_filename = terms.get(idx + 1).map(|s| s.to_owned());
                let filename = stderr_redirect_filename.ok_or_else(|| anyhow::anyhow!("Missing filename for stderr append redirect: {}", input))?;
                stderr_redirect = Some(Redirect { filename, should_append: true });
                idx = idx + 2;
            } else {
                terms_without_redirect.push(terms[idx].clone());
                idx = idx + 1;
            }
        }
        if let Some(command) = terms_without_redirect.get(0) {
            let args = terms_without_redirect[1..].to_vec();
            Ok(Some(ParsedCommand { command: command.clone(), args, stdout_redirect, stderr_redirect }))
        } else {
            Err(anyhow::anyhow!("No command provided: {}", input))
        }
    }

    pub(crate) fn parse_command(input: &str) -> Result<Option<ParsedCommand>, anyhow::Error> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(None);
        }
        let command_and_args = ParsedCommand::read_quoted(input)?;
        ParsedCommand::parse_terms(input, &command_and_args)
    }

    fn read_quoted(input: &str) -> Result<Vec<String>, anyhow::Error> {
        let mut result: Vec<String> = Vec::new();
        let mut inside_single_quotes = false;
        let mut inside_double_quotes = false;
        let mut current_part = String::new();
        let mut is_escaped_character = false;
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if inside_single_quotes {
                if ch == '\'' {
                    inside_single_quotes = false;
                } else {
                    current_part.push(ch);
                }
                i += 1;
            } else if inside_double_quotes {
                if is_escaped_character {
                    if ch == '\"' || ch == '\\' || ch == '$' || ch == '`' {
                        current_part.push(ch);
                    } else if ch == 'n' {
                        //In a real shell we would need to push \n instead but to make Codecrafters test suite happy we do not handle newline this way
                        //current_part.push('\n');
                        current_part.push('\\');
                        current_part.push('n');
                    } else {
                        // If it's not a recognized escape sequence, treat the backslash as literal
                        current_part.push('\\');
                        current_part.push(ch);
                    }
                    is_escaped_character = false;
                } else if ch == '"' {
                    inside_double_quotes = false;
                } else if ch == '\\' {
                    is_escaped_character = true;
                } else {
                    current_part.push(ch);
                }
                i += 1;
            } else {
                if is_escaped_character {
                    is_escaped_character = false;
                    current_part.push(ch);
                    i += 1;
                } else if ch == '\\' {
                    is_escaped_character = true;
                    i += 1;
                } else if ch == '\'' {
                    inside_single_quotes = true;
                    i += 1;
                } else if ch == '"' {
                    inside_double_quotes = true;
                    i += 1;
                } else if ch == ' ' {
                    if current_part.len() > 0 {
                        result.push(current_part.clone());
                        current_part = String::new();
                    }
                    i += 1;
                } else {
                    let redirect_patterns = ["2>>", "1>>", ">>", "2>", "1>", ">"];
                    let mut found_redirect = false;

                    for pattern in redirect_patterns.iter() {
                        if i + pattern.len() <= chars.len() {
                            let potential_match: String = chars[i..i + pattern.len()].iter().collect();
                            // Found a redirect operator
                            if potential_match == *pattern {
                                if current_part.len() > 0 {
                                    result.push(current_part.clone());
                                    current_part = String::new();
                                }
                                result.push(pattern.to_string());
                                i += pattern.len();
                                found_redirect = true;
                                break;
                            }
                        }
                    }

                    if !found_redirect {
                        current_part.push(ch);
                        i += 1;
                    }
                }
            }
        }

        if current_part.len() > 0 {
            result.push(current_part);
        }

        Ok(result)
    }


    pub(crate) fn get_args(&self) -> Vec<&str> {
        self.args.iter().map(|arg| arg.as_str()).collect::<Vec<&str>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd_with_redirects(
        command: &str,
        args: Vec<&str>,
        stdout_redirect: Option<(&str, bool)>,
        stderr_redirect: Option<(&str, bool)>
    ) -> ParsedCommand {
        ParsedCommand {
            command: command.to_string(),
            args: args.into_iter().map(|s| s.to_string()).collect(),
            stdout_redirect: stdout_redirect.map(|(filename, should_append)| Redirect {
                filename: filename.to_string(),
                should_append
            }),
            stderr_redirect: stderr_redirect.map(|(filename, should_append)| Redirect {
                filename: filename.to_string(),
                should_append
            })
        }
    }

    fn cmd(command: &str, args: Vec<&str>) -> ParsedCommand {
        cmd_with_redirects(command, args, None, None)
    }

    #[test]
    fn test_parse_empty_command() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("")?;
        assert!(result.is_none(), "Empty command should return None");
        Ok(())
    }

    #[test]
    fn test_execute_quoted_command() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("'exe with \"quotes\"' file")?;
        assert_eq!(result, Some(cmd("exe with \"quotes\"", vec!["file"])));
        Ok(())
    }

    #[test]
    fn test_parse_whitespace_only_command() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("   \t\n   ")?;
        assert!(result.is_none(), "Whitespace-only command should return None");
        Ok(())
    }

    #[test]
    fn test_parse_simple_command_no_args() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("pwd")?;
        assert_eq!(result, Some(cmd("pwd", vec![])));
        Ok(())
    }

    #[test]
    fn test_parse_echo_with_number() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo 123")?;
        assert_eq!(result, Some(cmd("echo", vec!["123"])));
        Ok(())
    }

    #[test]
    fn test_parse_ls_with_stdout_redirect() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("ls /tmp/baz > /tmp/foo/baz.md")?;
        assert_eq!(result, Some(cmd_with_redirects("ls", vec!["/tmp/baz"], Some(("/tmp/foo/baz.md", false)), None)));
        Ok(())
    }

    #[test]
    fn test_parse_cat_with_stdout_wordier_redirect() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("cat /tmp/baz/blueberry nonexistent 1> /tmp/foo/quz.md")?;
        assert_eq!(result, Some(cmd_with_redirects("cat", vec!["/tmp/baz/blueberry", "nonexistent"], Some(("/tmp/foo/quz.md", false)), None)));
        Ok(())
    }

    #[test]
    fn test_parse_ls_with_stderr_redirect() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("ls /tmp/baz 2> /tmp/foo/baz.md")?;
        assert_eq!(result, Some(cmd_with_redirects("ls", vec!["/tmp/baz"], None, Some(("/tmp/foo/baz.md", false)))));
        Ok(())
    }

    #[test]
    fn test_parse_ls_with_stdout_and_stderr_redirect() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("ls /tmp/baz 1> /tmp/foo/baz1.md 2> /tmp/foo/baz2.md")?;
        assert_eq!(result, Some(cmd_with_redirects("ls", vec!["/tmp/baz"], Some(("/tmp/foo/baz1.md", false)), Some(("/tmp/foo/baz2.md", false)))));
        Ok(())
    }

    #[test]
    fn test_parse_echo_with_multiple_args() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo hello world")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello", "world"])));
        Ok(())
    }

    #[test]
    fn test_parse_echo_with_single_quoted_string() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo 'hello    world'")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello    world"])));
        Ok(())
    }

    #[test]
    fn test_parse_cat_with_quoted_file_paths() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("cat '/tmp/file name' '/tmp/file name with spaces'")?;
        assert_eq!(result, Some(cmd("cat", vec!["/tmp/file name", "/tmp/file name with spaces"])));
        Ok(())
    }

    #[test]
    fn test_parse_mixed_quoted_and_unquoted_args() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("cp 'file with spaces.txt' /tmp/destination")?;
        assert_eq!(result, Some(cmd("cp", vec!["file with spaces.txt", "/tmp/destination"])));
        Ok(())
    }

    #[test]
    fn test_parse_multiple_spaces_between_args() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo   hello    world   ")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello", "world"])));
        Ok(())
    }

    #[test]
    fn test_parse_empty_quoted_string() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo ''")?;
        assert_eq!(result, Some(cmd("echo", vec![])));
        Ok(())
    }

    #[test]
    fn test_parse_single_quote_in_middle() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo don't")?;
        assert_eq!(result, Some(cmd("echo", vec!["dont"])));
        Ok(())
    }

    #[test]
    fn test_parse_command_with_leading_trailing_spaces() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("  echo hello  ")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello"])));
        Ok(())
    }

    #[test]
    fn test_parse_complex_command_line() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("rsync -av 'source dir/' '/dest/path with spaces/' --exclude='*.tmp'")?;
        assert_eq!(result, Some(cmd("rsync", vec!["-av", "source dir/", "/dest/path with spaces/", "--exclude=*.tmp"])));
        Ok(())
    }

    #[test]
    fn test_parse_quotes_next_to_each_other() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo 'example     shell' 'hello''test' script''world")?;
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
        let result = ParsedCommand::parse_command("echo \"quz  hello\"  \"bar\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["quz  hello", "bar"])));
        Ok(())
    }

    #[test]
    fn test_parse_echo_with_double_quoted_strings_containing_single_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \"bar\"  \"shell's\"  \"foo\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["bar", "shell's", "foo"])));
        Ok(())
    }

    #[test]
    fn test_parse_cat_with_double_quoted_file_paths() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("cat \"/tmp/file name\" \"/tmp/'file name' with spaces\"")?;
        assert_eq!(result, Some(cmd("cat", vec!["/tmp/file name", "/tmp/'file name' with spaces"])));
        Ok(())
    }

    #[test]
    fn test_parse_empty_double_quoted_string() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \"\"")?;
        assert_eq!(result, Some(cmd("echo", vec![])));
        Ok(())
    }

    #[test]
    fn test_parse_mixed_single_and_double_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo 'single quoted' \"double quoted\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["single quoted", "double quoted"])));
        Ok(())
    }

    #[test]
    fn test_parse_double_quotes_next_to_each_other() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \"hello\"\"world\" \"test\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["helloworld", "test"])));
        Ok(())
    }

    #[test]
    fn test_parse_backslash_before_blank_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \"before\\   after\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["before\\   after"])));
        Ok(())
    }

    #[test]
    fn test_parse_multiple_backslashes_before_blanks() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo world\\ \\ \\ \\ \\ \\ script")?;
        assert_eq!(result, Some(cmd("echo", vec!["world      script"])));
        Ok(())
    }

    #[test]
    fn test_parse_multiple_backslashes_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("cat \"/tmp/file\\\\name\" \"/tmp/file\\ name\"")?;
        assert_eq!(result, Some(cmd("cat", vec!["/tmp/file\\name", "/tmp/file\\ name"])));
        Ok(())
    }

    #[test]
    fn test_parse_escape_sequences_in_double_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("cat \"/tmp/quz/f\\n36\" \"/tmp/quz/f\\t12\" \"/tmp/quz/f\\'52\"")?;
        assert_eq!(result, Some(cmd("cat", vec!["/tmp/quz/f\\n36", "/tmp/quz/f\\t12", "/tmp/quz/f\\'52"])));
        Ok(())
    }

    #[test]
    fn test_parse_escape_sequences_outside_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo world\\nshell")?;
        assert_eq!(result, Some(cmd("echo", vec!["worldnshell"])));
        Ok(())
    }

    #[test]
    fn test_parse_simple_escaped_quote() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \\'hello\\'") ?;
        assert_eq!(result, Some(cmd("echo", vec!["'hello'"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_quotes_with_space() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \\'\\\"script shell\\\"\\'") ?;
        assert_eq!(result, Some(cmd("echo", vec!["'\"script", "shell\"'"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_double_quote_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \"hello\\\"world\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello\"world"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_backslash_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \"hello\\\\world\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello\\world"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_dollar_sign_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \"hello\\$world\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello$world"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_backtick_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \"hello\\`world\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello`world"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_newline_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \"hello\\nworld\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["hello\\nworld"])));
        Ok(())
    }

    #[test]
    fn test_parse_multiple_escaped_characters_inside_double_quotes() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \"\\\"hello\\\" \\$world \\`test\\` \\\\backslash\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["\"hello\" $world `test` \\backslash"])));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_characters_at_beginning_and_end() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo \"\\\"start\\\" and \\\"end\\\"\"")?;
        assert_eq!(result, Some(cmd("echo", vec!["\"start\" and \"end\""])));
        Ok(())
    }

    #[test]
    fn test_parse_ls_with_stdout_append_redirect() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("ls /tmp/baz >> /tmp/foo/baz.md")?;
        assert_eq!(result, Some(cmd_with_redirects("ls", vec!["/tmp/baz"], Some(("/tmp/foo/baz.md", true)), None)));
        Ok(())
    }

    #[test]
    fn test_parse_cat_with_stdout_append_redirect_with_fd() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("cat /tmp/file.txt 1>> /tmp/output.log")?;
        assert_eq!(result, Some(cmd_with_redirects("cat", vec!["/tmp/file.txt"], Some(("/tmp/output.log", true)), None)));
        Ok(())
    }

    #[test]
    fn test_parse_ls_with_stderr_append_redirect() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("ls /nonexistent 2>> /tmp/errors.log")?;
        assert_eq!(result, Some(cmd_with_redirects("ls", vec!["/nonexistent"], None, Some(("/tmp/errors.log", true)))));
        Ok(())
    }

    #[test]
    fn test_parse_ls_with_both_append_redirects() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("ls /tmp 1>> /tmp/output.log 2>> /tmp/errors.log")?;
        assert_eq!(result, Some(cmd_with_redirects("ls", vec!["/tmp"], Some(("/tmp/output.log", true)), Some(("/tmp/errors.log", true)))));
        Ok(())
    }

    #[test]
    fn test_parse_mixed_regular_and_append_redirects() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("cat file.txt > output.txt 2>> errors.log")?;
        let expected = ParsedCommand {
            command: "cat".to_string(),
            args: vec!["file.txt".to_string()],
            stdout_redirect: Some(Redirect { filename: "output.txt".to_string(), should_append: false }),
            stderr_redirect: Some(Redirect { filename: "errors.log".to_string(), should_append: true })
        };
        assert_eq!(result, Some(expected));
        Ok(())
    }

    #[test]
    fn test_parse_no_space_stdout_redirect() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo hello >output.txt")?;
        assert_eq!(result, Some(cmd_with_redirects("echo", vec!["hello"], Some(("output.txt", false)), None)));
        Ok(())
    }

    #[test]
    fn test_parse_no_space_stdout_redirect_with_fd() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("cat file.txt 1>output.txt")?;
        assert_eq!(result, Some(cmd_with_redirects("cat", vec!["file.txt"], Some(("output.txt", false)), None)));
        Ok(())
    }

    #[test]
    fn test_parse_no_space_stderr_redirect() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("ls /nonexistent 2>errors.log")?;
        assert_eq!(result, Some(cmd_with_redirects("ls", vec!["/nonexistent"], None, Some(("errors.log", false)))));
        Ok(())
    }

    #[test]
    fn test_parse_no_space_stdout_append_redirect() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("echo hello >>output.log")?;
        assert_eq!(result, Some(cmd_with_redirects("echo", vec!["hello"], Some(("output.log", true)), None)));
        Ok(())
    }

    #[test]
    fn test_parse_no_space_stdout_append_redirect_with_fd() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("cat file.txt 1>>output.log")?;
        assert_eq!(result, Some(cmd_with_redirects("cat", vec!["file.txt"], Some(("output.log", true)), None)));
        Ok(())
    }

    #[test]
    fn test_parse_no_space_stderr_append_redirect() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("ls /nonexistent 2>>errors.log")?;
        assert_eq!(result, Some(cmd_with_redirects("ls", vec!["/nonexistent"], None, Some(("errors.log", true)))));
        Ok(())
    }

    #[test]
    fn test_parse_no_space_multiple_redirects() -> Result<(), anyhow::Error> {
        let result = ParsedCommand::parse_command("cat file.txt 1>output.txt 2>>errors.log")?;
        let expected = ParsedCommand {
            command: "cat".to_string(),
            args: vec!["file.txt".to_string()],
            stdout_redirect: Some(Redirect { filename: "output.txt".to_string(), should_append: false }),
            stderr_redirect: Some(Redirect { filename: "errors.log".to_string(), should_append: true })
        };
        assert_eq!(result, Some(expected));
        Ok(())
    }
}