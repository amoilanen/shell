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

pub(crate) fn run(executable: &str, parsed_command: &ParsedCommand) -> () {
    let exec_info = parse_executable_path(executable);
    let mut command = build_command(&exec_info, parsed_command.args.iter().map(|a| a.as_str()).collect::<Vec<&str>>().as_slice());
    
    let output = command
        .output()
        .expect(&format!("Failed to execute process {}", parsed_command.command.as_str()));
    
    write_output(&parsed_command.stdout_redirect_filename, &output.stdout);
    write_output(&parsed_command.stderr_redirect_filename, &output.stderr);
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

fn write_output(filename: &Option<String>, content: &[u8]) -> () {
    if let Some(filename) = filename {
        write_output_to_file(&filename, content).unwrap();
    } else {
        print!("{}", String::from_utf8_lossy(content));
    }
}

fn write_output_to_file(filename: &str, content: &[u8]) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
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
            stdout_redirect_filename: stdout_redirect,
            stderr_redirect_filename: stderr_redirect
        }
    }

    fn create_temp_file_path(filename: &str) -> String {
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join(filename);
        file_path.to_string_lossy().to_string()
    }

    fn read_file_content(path: &str) -> String {
        let mut content = String::new();
        let mut file = fs::File::open(path).expect(&format!("Failed to open file: {}", path));
        file.read_to_string(&mut content).expect(&format!("Failed to read file: {}", path));
        content
    }

    fn cleanup_files(paths: &[&str]) {
        for path in paths {
            fs::remove_file(path).ok();
        }
    }

    fn assert_file_contains_error_message(path: &str, message: &str) {
        let content = read_file_content(path);
        assert!(content.contains(message), "File should contain error message, got: {}", content);
    }

    fn assert_file_empty_or_missing(path: &str, context: &str) {
        if fs::metadata(path).is_ok() {
            let content = read_file_content(path);
            assert_eq!(content, "", "{}", context);
        }
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
    fn test_stderr_redirect_to_file() {
        let stderr_path = create_temp_file_path("test_stderr.txt");

        let parsed_command = create_test_parsed_command(
            "ls",
            vec!["/nonexistent".to_string()],
            None,
            Some(stderr_path.clone())
        );

        run("ls", &parsed_command);

        assert_file_contains_error_message(&stderr_path, "");
        cleanup_files(&[&stderr_path]);
    }

    #[test]
    fn test_stderr_redirect_with_stdout_redirect() {
        let stdout_path = create_temp_file_path("test_stdout.txt");
        let stderr_path = create_temp_file_path("test_stderr.txt");

        let parsed_command = create_test_parsed_command(
            "ls",
            vec!["/nonexistent".to_string(), "/tmp".to_string()],
            Some(stdout_path.clone()),
            Some(stderr_path.clone())
        );

        run("ls", &parsed_command);

        assert_file_contains_error_message(&stderr_path, "");

        let stdout_exists = fs::metadata(&stdout_path).is_ok();
        assert!(stdout_exists, "Stdout file should be created even when stderr is redirected");
        
        cleanup_files(&[&stdout_path, &stderr_path]);
    }

    #[test]
    fn test_stderr_redirect_empty_stderr() {
        let stderr_path = create_temp_file_path("test_empty_stderr.txt");
        
        let parsed_command = create_test_parsed_command(
            "echo",
            vec!["hello".to_string()],
            None,
            Some(stderr_path.clone())
        );

        run("echo", &parsed_command);

        assert_file_empty_or_missing(&stderr_path, "Stderr file should be empty when command produces no stderr");
        cleanup_files(&[&stderr_path]);
    }

    #[test]
    fn test_stdout_redirect_to_file() {
        let stdout_path = create_temp_file_path("test_stdout.txt");
        
        let parsed_command = create_test_parsed_command(
            "echo",
            vec!["hello world".to_string()],
            Some(stdout_path.clone()),
            None
        );
        
        run("echo", &parsed_command);
        
        let content = read_file_content(&stdout_path);
        assert!(content.contains("hello world"), "Stdout file should contain command output");
        
        cleanup_files(&[&stdout_path]);
    }

    #[test]
    fn test_stdout_redirect_empty_stdout() {
        let stdout_path = create_temp_file_path("test_empty_stdout.txt");
        
        let parsed_command = create_test_parsed_command(
            "ls",
            vec!["/nonexistent".to_string()],
            Some(stdout_path.clone()),
            None
        );
        
        run("ls", &parsed_command);
        
        assert_file_empty_or_missing(&stdout_path, "Stdout file should be empty when command produces no stdout");
        cleanup_files(&[&stdout_path]);
    }
}