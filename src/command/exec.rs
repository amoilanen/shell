use std::process::Command;
use std::path::Path;
use std::io::Write;
use std::fs::OpenOptions;
use crate::command::ParsedCommand;

#[derive(Debug, PartialEq)]
struct ExecutableInfo {
    pub name: String,
    pub directory: String,
}

pub(crate) fn run(executable: &str, parsed_command: &ParsedCommand) -> Result<(), anyhow::Error> {
    let exec_info = parse_executable_path(executable);
    let mut command = build_command(&exec_info, parsed_command.args.iter().map(|a| a.as_str()).collect::<Vec<&str>>().as_slice());

    let output = command
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute process {}: {}", parsed_command.command.as_str(), e))?;

    write_output(&parsed_command.stdout_redirect.as_ref().map(|r| (r.filename.as_str(), r.should_append)), &output.stdout)?;
    write_output(&parsed_command.stderr_redirect.as_ref().map(|r| (r.filename.as_str(), r.should_append)), &output.stderr)?;
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
        print!("{}", String::from_utf8_lossy(content));
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
            stderr_redirect: stderr_redirect.map(|filename| crate::command::Redirect { filename, should_append: false })
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

        run("ls", &parsed_command)?;

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

        run("ls", &parsed_command)?;

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

        run("echo", &parsed_command)?;

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

        run("echo", &parsed_command)?;

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

        run("ls", &parsed_command)?;

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
            stderr_redirect: None
        };

        run("echo", &parsed_command)?;

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
            })
        };

        run("ls", &parsed_command)?;

        let content = read_file_content(&stderr_path)?;
        assert!(content.contains("initial error"), "File should contain initial content");
        // The exact error message may vary, but there should be some error content
        assert!(content.len() > initial_content.len(), "File should have more content after append");

        cleanup_files(&[&stderr_path]);
        Ok(())
    }
}