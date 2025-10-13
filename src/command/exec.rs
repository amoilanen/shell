use std::process::{Command, Stdio, Child, ChildStdout};
use std::path::Path;
use std::io::Write;
use std::fs::OpenOptions;
use crate::command::ParsedCommand;

#[derive(Debug, PartialEq)]
struct ExecutableInfo {
    pub name: String,
    pub directory: String,
}

pub(crate) fn run(parsed_command: &ParsedCommand) -> Result<(), anyhow::Error> {
    if let Some(piped_cmd) = &parsed_command.piped_command {
        run_pipeline(parsed_command, piped_cmd)
    } else {
        run_simple_command(parsed_command)
    }
}

fn run_simple_command(parsed_command: &ParsedCommand) -> Result<(), anyhow::Error> {
    let mut command = build_command_from_parsed(&parsed_command.command, &parsed_command.args);
    let output = command
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute process {}: {}", parsed_command.command, e))?;
    write_command_output(parsed_command, &output.stdout, &output.stderr)
}

fn run_pipeline(first_cmd: &ParsedCommand, second_cmd: &ParsedCommand) -> Result<(), anyhow::Error> {
    let (first_child, first_stdout) = spawn_first_pipeline_command(first_cmd)?;
    let second_child = spawn_second_pipeline_command(second_cmd, first_stdout)?;
    await_pipeline_completion(first_child, second_child, second_cmd)
}

fn spawn_first_pipeline_command(cmd: &ParsedCommand) -> Result<(Child, ChildStdout), anyhow::Error> {
    let mut command = build_command_from_parsed(&cmd.command, &cmd.args);
    command.stdout(Stdio::piped());

    let mut child = command.spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn first command: {}", e))?;

    let stdout = child.stdout.take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout from first command"))?;

    Ok((child, stdout))
}

fn spawn_second_pipeline_command(cmd: &ParsedCommand, stdin: ChildStdout) -> Result<Child, anyhow::Error> {
    let mut command = build_command_from_parsed(&cmd.command, &cmd.args);
    command.stdin(Stdio::from(stdin));

    configure_command_stdio(&mut command, cmd.stdout_redirect.is_some(), cmd.stderr_redirect.is_some());

    command.spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn second command: {}", e))
}

fn configure_command_stdio(command: &mut Command, capture_stdout: bool, capture_stderr: bool) {
    if capture_stdout {
        command.stdout(Stdio::piped());
    } else {
        command.stdout(Stdio::inherit());
    }

    if capture_stderr {
        command.stderr(Stdio::piped());
    } else {
        command.stderr(Stdio::inherit());
    }
}

fn await_pipeline_completion(
    mut first_child: Child,
    mut second_child: Child,
    second_cmd: &ParsedCommand
) -> Result<(), anyhow::Error> {
    let needs_output_capture = second_cmd.stdout_redirect.is_some() || second_cmd.stderr_redirect.is_some();

    if needs_output_capture {
        let output = second_child.wait_with_output()
            .map_err(|e| anyhow::anyhow!("Failed to wait for second command: {}", e))?;
        let _ = first_child.wait();
        write_command_output(second_cmd, &output.stdout, &output.stderr)?;
    } else {
        second_child.wait()
            .map_err(|e| anyhow::anyhow!("Failed to wait for second command: {}", e))?;
        let _ = first_child.wait();
    }

    Ok(())
}

fn build_command_from_parsed(command_name: &str, args: &[String]) -> Command {
    let exec_info = parse_executable_path(command_name);
    build_command(&exec_info, args.iter().map(|a| a.as_str()).collect::<Vec<&str>>().as_slice())
}

fn write_command_output(parsed_command: &ParsedCommand, stdout: &[u8], stderr: &[u8]) -> Result<(), anyhow::Error> {
    write_output(&parsed_command.stdout_redirect.as_ref().map(|r| (r.filename.as_str(), r.should_append)), stdout)?;
    write_output(&parsed_command.stderr_redirect.as_ref().map(|r| (r.filename.as_str(), r.should_append)), stderr)?;
    Ok(())
}

fn parse_executable_path(executable: &str) -> ExecutableInfo {
    let path = Path::new(executable);
    let executable_name = path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(executable)
        .to_string();
    let executable_dir = ".".to_string();
    ExecutableInfo {
        name: executable_name,
        directory: executable_dir,
    }
}

fn build_command(exec_info: &ExecutableInfo, args: &[&str]) -> Command {
    let mut command = Command::new(&exec_info.name);
    command.args(args);
    command.current_dir(&exec_info.directory);
    command
}

fn write_output(filename_and_append: &Option<(&str, bool)>, content: &[u8]) -> Result<(), anyhow::Error> {
    if let Some((filename, should_append)) = filename_and_append {
        write_output_to_file(&filename, content, *should_append)
            .map_err(|e| anyhow::anyhow!("Failed to write output to file '{}': {}", filename, e))?;
    } else {
        print!("\r{}", String::from_utf8_lossy(content));
    }
    Ok(())
}

fn write_output_to_file(filename: &str, content: &[u8], should_append: bool) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(should_append)
        .truncate(!should_append)
        .open(filename)?;
    file.write_all(content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::io::Read;

    fn create_test_parsed_command(command: &str, args: Vec<String>, stdout_redirect: Option<String>, stderr_redirect: Option<String>) -> ParsedCommand {
        ParsedCommand {
            command: command.to_string(),
            args,
            stdout_redirect: stdout_redirect.map(|filename| crate::command::Redirect { filename, should_append: false }),
            stderr_redirect: stderr_redirect.map(|filename| crate::command::Redirect { filename, should_append: false }),
            piped_command: None
        }
    }

    fn create_temp_file_path(filename: &str) -> String {
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join(filename);
        file_path.to_string_lossy().to_string()
    }

    fn read_file_content(path: &str) -> Result<String, anyhow::Error> {
        let mut content = String::new();
        let mut file = fs::File::open(path)
            .map_err(|e| anyhow::anyhow!("Failed to open file '{}': {}", path, e))?;
        file.read_to_string(&mut content)
            .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", path, e))?;
        Ok(content)
    }

    fn cleanup_files(paths: &[&str]) {
        for path in paths {
            fs::remove_file(path).ok();
        }
    }

    fn assert_file_contains_error_message(path: &str, message: &str) -> Result<(), anyhow::Error> {
        let content = read_file_content(path)?;
        assert!(content.contains(message), "File should contain error message, got: {}", content);
        Ok(())
    }

    fn assert_file_empty_or_missing(path: &str, context: &str) -> Result<(), anyhow::Error> {
        if fs::metadata(path).is_ok() {
            let content = read_file_content(path)?;
            assert_eq!(content, "", "{}", context);
        }
        Ok(())
    }

    #[test]
    fn test_parse_executable_path_simple_command() {
        let result = parse_executable_path("ls");
        assert_eq!(result, ExecutableInfo {
            name: "ls".to_string(),
            directory: ".".to_string(),
        });
    }

    #[test]
    fn test_parse_executable_path_relative_path() {
        let result = parse_executable_path("./bin/my_program");
        assert_eq!(result, ExecutableInfo {
            name: "my_program".to_string(),
            directory: ".".to_string(),
        });
    }

    #[test]
    fn test_parse_executable_path_absolute_path() {
        let result = parse_executable_path("/usr/bin/python3.9");
        assert_eq!(result, ExecutableInfo {
            name: "python3.9".to_string(),
            directory: ".".to_string(),
        });
    }

    #[test]
    fn test_parse_executable_path_root_executable() {
        let result = parse_executable_path("/program");
        assert_eq!(result, ExecutableInfo {
            name: "program".to_string(),
            directory: ".".to_string(),
        });
    }

    #[test]
    fn test_parse_executable_path_current_dir_executable() {
        let result = parse_executable_path("./program");
        assert_eq!(result, ExecutableInfo {
            name: "program".to_string(),
            directory: ".".to_string(),
        });
    }

    #[test]
    fn test_parse_executable_path_edge_cases() {
        let result = parse_executable_path("");
        assert_eq!(result, ExecutableInfo {
            name: "".to_string(),
            directory: ".".to_string(),
        });
        let result = parse_executable_path(".");
        assert_eq!(result, ExecutableInfo {
            name: ".".to_string(),
            directory: ".".to_string(),
        });
        let result = parse_executable_path("/bin//ls");
        assert_eq!(result, ExecutableInfo {
            name: "ls".to_string(),
            directory: ".".to_string(),
        });
    }

    #[test]
    fn test_parse_executable_path_special_characters() {
        let result = parse_executable_path("/bin/my-special_program.v2");
        assert_eq!(result, ExecutableInfo {
            name: "my-special_program.v2".to_string(),
            directory: ".".to_string(),
        });
    }

    #[test]
    fn test_build_command_simple() {
        let exec_info = ExecutableInfo {
            name: "ls".to_string(),
            directory: ".".to_string(),
        };
        let command = build_command(&exec_info, &["--help"]);

        let debug_str = format!("{:?}", command);
        assert_eq!(debug_str, "cd \".\" && \"ls\" \"--help\"");
    }

    #[test]
    fn test_build_command_with_directory() {
        let exec_info = ExecutableInfo {
            name: "ls".to_string(),
            directory: "/bin".to_string(),
        };
        let command = build_command(&exec_info, &["-la"]);
        
        let debug_str = format!("{:?}", command);
        assert_eq!(debug_str, "cd \"/bin\" && \"ls\" \"-la\"");
    }

    #[test]
    fn test_build_command_multiple_args() {
        let exec_info = ExecutableInfo {
            name: "grep".to_string(),
            directory: ".".to_string(),
        };
        let command = build_command(&exec_info, &["-r", "pattern", "."]);
        
        let debug_str = format!("{:?}", command);
        assert_eq!(debug_str, "cd \".\" && \"grep\" \"-r\" \"pattern\" \".\"");
    }

    #[test]
    fn test_build_command_no_args() {
        let exec_info = ExecutableInfo {
            name: "pwd".to_string(),
            directory: ".".to_string(),
        };
        let command = build_command(&exec_info, &[]);
        
        let debug_str = format!("{:?}", command);
        assert_eq!(debug_str, ("cd \".\" && \"pwd\""));
    }

    #[test]
    fn test_parse_and_build_workflow() {
        let executable = "/bin/ls";
        let args = &["-la", "/tmp"];
        
        let exec_info = parse_executable_path(executable);
        let command = build_command(&exec_info, args);
        
        assert_eq!(exec_info.name, "ls");
        assert_eq!(exec_info.directory, ".");
        
        let debug_str = format!("{:?}", command);
        assert_eq!(debug_str, "cd \".\" && \"ls\" \"-la\" \"/tmp\"");
    }

    #[test]
    fn test_stderr_redirect_to_file() -> Result<(), anyhow::Error> {
        let stderr_path = create_temp_file_path("test_stderr.txt");

        let parsed_command = create_test_parsed_command(
            "ls",
            vec!["/nonexistent".to_string()],
            None,
            Some(stderr_path.clone())
        );

        run(&parsed_command)?;

        assert_file_contains_error_message(&stderr_path, "")?;
        cleanup_files(&[&stderr_path]);
        Ok(())
    }

    #[test]
    fn test_stderr_redirect_with_stdout_redirect() -> Result<(), anyhow::Error> {
        let stdout_path = create_temp_file_path("test_stdout.txt");
        let stderr_path = create_temp_file_path("test_stderr.txt");

        let parsed_command = create_test_parsed_command(
            "ls",
            vec!["/nonexistent".to_string(), "/tmp".to_string()],
            Some(stdout_path.clone()),
            Some(stderr_path.clone())
        );

        run(&parsed_command)?;

        assert_file_contains_error_message(&stderr_path, "")?;

        let stdout_exists = fs::metadata(&stdout_path).is_ok();
        assert!(stdout_exists, "Stdout file should be created even when stderr is redirected");

        cleanup_files(&[&stdout_path, &stderr_path]);
        Ok(())
    }

    #[test]
    fn test_stderr_redirect_empty_stderr() -> Result<(), anyhow::Error> {
        let stderr_path = create_temp_file_path("test_empty_stderr.txt");

        let parsed_command = create_test_parsed_command(
            "echo",
            vec!["hello".to_string()],
            None,
            Some(stderr_path.clone())
        );

        run(&parsed_command)?;

        assert_file_empty_or_missing(&stderr_path, "Stderr file should be empty when command produces no stderr")?;
        cleanup_files(&[&stderr_path]);
        Ok(())
    }

    #[test]
    fn test_stdout_redirect_to_file() -> Result<(), anyhow::Error> {
        let stdout_path = create_temp_file_path("test_stdout.txt");

        let parsed_command = create_test_parsed_command(
            "echo",
            vec!["hello world".to_string()],
            Some(stdout_path.clone()),
            None
        );

        run(&parsed_command)?;

        let content = read_file_content(&stdout_path)?;
        assert!(content.contains("hello world"), "Stdout file should contain command output");

        cleanup_files(&[&stdout_path]);
        Ok(())
    }

    #[test]
    fn test_stdout_redirect_empty_stdout() -> Result<(), anyhow::Error> {
        let stdout_path = create_temp_file_path("test_empty_stdout.txt");

        let parsed_command = create_test_parsed_command(
            "ls",
            vec!["/nonexistent".to_string()],
            Some(stdout_path.clone()),
            None
        );

        run(&parsed_command)?;

        assert_file_empty_or_missing(&stdout_path, "Stdout file should be empty when command produces no stdout")?;
        cleanup_files(&[&stdout_path]);
        Ok(())
    }

    #[test]
    fn test_stdout_redirect_append_mode() -> Result<(), anyhow::Error> {
        let stdout_path = create_temp_file_path("test_stdout_append.txt");

        let initial_content = "initial content\n";
        write_output_to_file(&stdout_path, initial_content.as_bytes(), false)
            .map_err(|e| anyhow::anyhow!("Failed to write initial content: {}", e))?;

        let parsed_command = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["appended content".to_string()],
            stdout_redirect: Some(crate::command::Redirect {
                filename: stdout_path.clone(),
                should_append: true
            }),
            stderr_redirect: None,
            piped_command: None
        };

        run(&parsed_command)?;

        let content = read_file_content(&stdout_path)?;
        assert!(content.contains("initial content\nappended content\n"), "File should contain all content");

        cleanup_files(&[&stdout_path]);
        Ok(())
    }

    #[test]
    fn test_stderr_redirect_append_mode() -> Result<(), anyhow::Error> {
        let stderr_path = create_temp_file_path("test_stderr_append.txt");

        let initial_content = "initial error\n";
        write_output_to_file(&stderr_path, initial_content.as_bytes(), false)
            .map_err(|e| anyhow::anyhow!("Failed to write initial content: {}", e))?;

        let parsed_command = ParsedCommand {
            command: "ls".to_string(),
            args: vec!["/nonexistent_directory".to_string()],
            stdout_redirect: None,
            stderr_redirect: Some(crate::command::Redirect {
                filename: stderr_path.clone(),
                should_append: true
            }),
            piped_command: None
        };

        run(&parsed_command)?;

        let content = read_file_content(&stderr_path)?;
        assert!(content.contains("initial error"), "File should contain initial content");
        // The exact error message may vary, but there should be some error content
        assert!(content.len() > initial_content.len(), "File should have more content after append");

        cleanup_files(&[&stderr_path]);
        Ok(())
    }

    #[test]
    fn test_pipeline_simple_echo_to_cat() -> Result<(), anyhow::Error> {
        let stdout_path = create_temp_file_path("test_pipe_echo_cat.txt");

        let second_cmd = ParsedCommand {
            command: "cat".to_string(),
            args: vec![],
            stdout_redirect: Some(crate::command::Redirect {
                filename: stdout_path.clone(),
                should_append: false
            }),
            stderr_redirect: None,
            piped_command: None
        };

        let first_cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["hello world".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(second_cmd))
        };

        run(&first_cmd)?;

        let content = read_file_content(&stdout_path)?;
        assert!(content.contains("hello world"), "Pipeline output should contain 'hello world', got: {}", content);

        cleanup_files(&[&stdout_path]);
        Ok(())
    }

    #[test]
    fn test_pipeline_with_grep() -> Result<(), anyhow::Error> {
        let stdout_path = create_temp_file_path("test_pipe_grep.txt");

        let second_cmd = ParsedCommand {
            command: "grep".to_string(),
            args: vec!["ba".to_string()],
            stdout_redirect: Some(crate::command::Redirect {
                filename: stdout_path.clone(),
                should_append: false
            }),
            stderr_redirect: None,
            piped_command: None
        };

        let first_cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["-e".to_string(), "foo\nbar\nbaz".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(second_cmd))
        };

        run(&first_cmd)?;

        let content = read_file_content(&stdout_path)?;
        assert!(content.contains("bar"), "Pipeline should filter and contain 'bar', got: {}", content);
        assert!(content.contains("baz"), "Pipeline should filter and contain 'baz', got: {}", content);
        assert!(!content.contains("foo\n"), "Pipeline should not contain 'foo' on its own line");

        cleanup_files(&[&stdout_path]);
        Ok(())
    }

    #[test]
    fn test_pipeline_head_with_args() -> Result<(), anyhow::Error> {
        let stdout_path = create_temp_file_path("test_pipe_head.txt");

        let second_cmd = ParsedCommand {
            command: "head".to_string(),
            args: vec!["-n".to_string(), "2".to_string()],
            stdout_redirect: Some(crate::command::Redirect {
                filename: stdout_path.clone(),
                should_append: false
            }),
            stderr_redirect: None,
            piped_command: None
        };

        let first_cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["-e".to_string(), "line1\nline2\nline3".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(second_cmd))
        };

        run(&first_cmd)?;

        let content = read_file_content(&stdout_path)?;
        assert!(content.contains("line1"), "Pipeline should contain line1");
        assert!(content.contains("line2"), "Pipeline should contain line2");

        cleanup_files(&[&stdout_path]);
        Ok(())
    }

    #[test]
    fn test_pipeline_with_wc() -> Result<(), anyhow::Error> {
        let stdout_path = create_temp_file_path("test_pipe_wc.txt");

        let second_cmd = ParsedCommand {
            command: "wc".to_string(),
            args: vec!["-w".to_string()],
            stdout_redirect: Some(crate::command::Redirect {
                filename: stdout_path.clone(),
                should_append: false
            }),
            stderr_redirect: None,
            piped_command: None
        };

        let first_cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["hello world".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(second_cmd))
        };

        run(&first_cmd)?;

        let content = read_file_content(&stdout_path)?;
        let trimmed = content.trim();
        assert_eq!(trimmed, "2", "Word count should be 2, got: {}", trimmed);

        cleanup_files(&[&stdout_path]);
        Ok(())
    }

    #[test]
    fn test_pipeline_stderr_redirect_on_second_command() -> Result<(), anyhow::Error> {
        let stdout_path = create_temp_file_path("test_pipe_stderr_out.txt");
        let stderr_path = create_temp_file_path("test_pipe_stderr_err.txt");

        let second_cmd = ParsedCommand {
            command: "grep".to_string(),
            args: vec!["nonexistent_pattern_xyz".to_string()],
            stdout_redirect: Some(crate::command::Redirect {
                filename: stdout_path.clone(),
                should_append: false
            }),
            stderr_redirect: Some(crate::command::Redirect {
                filename: stderr_path.clone(),
                should_append: false
            }),
            piped_command: None
        };

        let first_cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(second_cmd))
        };

        run(&first_cmd)?;

        assert_file_empty_or_missing(&stdout_path, "Stdout should be empty when grep finds no matches")?;

        cleanup_files(&[&stdout_path, &stderr_path]);
        Ok(())
    }

    #[test]
    fn test_pipeline_append_mode() -> Result<(), anyhow::Error> {
        let stdout_path = create_temp_file_path("test_pipe_append.txt");
        write_output_to_file(&stdout_path, "initial\n".as_bytes(), false)?;

        let second_cmd = ParsedCommand {
            command: "cat".to_string(),
            args: vec![],
            stdout_redirect: Some(crate::command::Redirect {
                filename: stdout_path.clone(),
                should_append: true
            }),
            stderr_redirect: None,
            piped_command: None
        };

        let first_cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["appended".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(second_cmd))
        };

        run(&first_cmd)?;

        let content = read_file_content(&stdout_path)?;
        assert!(content.contains("initial"), "File should contain initial content");
        assert!(content.contains("appended"), "File should contain appended content");

        cleanup_files(&[&stdout_path]);
        Ok(())
    }

    #[test]
    fn test_pipeline_tail_with_head_limited_input() -> Result<(), anyhow::Error> {
        let input_file = create_temp_file_path("test_tail_head_input.txt");
        let stdout_path = create_temp_file_path("test_tail_head_output.txt");

        write_output_to_file(&input_file, "1. banana strawberry\n2. apple pear\n3. orange mango\n".as_bytes(), false)?;

        let second_cmd = ParsedCommand {
            command: "head".to_string(),
            args: vec!["-n".to_string(), "5".to_string()],
            stdout_redirect: Some(crate::command::Redirect {
                filename: stdout_path.clone(),
                should_append: false
            }),
            stderr_redirect: None,
            piped_command: None
        };

        let first_cmd = ParsedCommand {
            command: "tail".to_string(),
            args: vec![input_file.clone()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(second_cmd))
        };

        run(&first_cmd)?;

        let content = read_file_content(&stdout_path)?;
        assert!(content.contains("1. banana strawberry"), "Output should contain first line, got: {}", content);
        assert!(content.contains("2. apple pear"), "Output should contain second line, got: {}", content);
        assert!(content.contains("3. orange mango"), "Output should contain third line, got: {}", content);

        cleanup_files(&[&input_file, &stdout_path]);
        Ok(())
    }
}