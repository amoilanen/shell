use std::process::Command;
use std::path::Path;
use std::io::Write;
use std::fs::OpenOptions;
use std::env;

#[derive(Debug, PartialEq)]
struct ExecutableInfo {
    pub name: String,
    pub directory: String,
}

pub(crate) fn run(args: &[&str], executable: &str, stdout_redirect_filename: Option<&str>) -> () {
    let exec_info = parse_executable_path(executable);
    let mut command = build_command(&exec_info, args);
    
    let output = command
        .output()
        .expect(&format!("Failed to execute process {}", executable));
    
    if let Some(stdout_redirect_filename) = stdout_redirect_filename {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(stdout_redirect_filename).unwrap();

        file.write_all(&output.stdout).unwrap();
    } else {
        let to_output = String::from_utf8_lossy(&output.stdout).to_string();
        print!("{}", to_output);
    }
    if !output.stderr.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stderr));
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}