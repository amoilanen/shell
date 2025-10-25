use std::path;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use crate::command::{self, ParsedCommand};

#[derive(Debug, Clone)]
pub(crate) struct Path {
    directories: Vec<String>
}

impl Path {
    pub(crate) fn parse(input: &str) -> Result<Path, anyhow::Error> {
        Ok(Path {
            directories: input.split(":").map(|p| p.to_string()).collect()
        })
    }

    pub(crate) fn resolve_piped_commands(
        &self,
        cmd: &mut ParsedCommand
    ) -> Result<(), String> {
        if let Some(piped_cmd) = &mut cmd.piped_command {
            if !command::builtin::is_builtin(&piped_cmd.command) {
                if let Some(found_piped_executable) = self.find_command(&piped_cmd.command) {
                    piped_cmd.command = found_piped_executable;
                } else {
                    return Err(piped_cmd.command.clone());
                }
            }
            self.resolve_piped_commands(piped_cmd)?;
        }
        Ok(())
    }

    pub(crate) fn find_command(&self, command_name: &str) -> Option<String> {
        self.find_executable_path(command_name)
    }

    pub(crate) fn find_matching_executables(&self, partial: &str) -> Vec<String> {
        if partial.is_empty() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for directory in &self.directories {
            let dir_path = path::Path::new(directory);
            if let Ok(entries) = fs::read_dir(dir_path) {
                for entry in entries.flatten() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        if file_name.starts_with(partial) {
                            let full_path = entry.path();
                            if self.is_executable(&full_path).unwrap_or(false) {
                                if seen.insert(file_name.clone()) {
                                    matches.push(file_name);
                                }
                            }
                        }
                    }
                }
            }
        }
        matches.sort();
        matches
    }

    fn find_executable_path(&self, command_name: &str) -> Option<String> {
        for directory in &self.directories {
            let path_to_command = path::Path::new(directory).join(command_name);
            if self.is_executable(&path_to_command).unwrap_or(false) {
                return path_to_command.to_str().map(|x| x.to_string());
            }
        }
        None
    }

    fn is_executable(&self, path: &PathBuf) -> std::io::Result<bool> {
        let metadata = fs::metadata(path)?;
        let permissions = metadata.permissions();

        if !metadata.is_file() {
            return Ok(false);
        }

        let mode = permissions.mode();
        Ok(mode & 0o111 != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    fn create_test_directory() -> tempfile::TempDir {
        tempfile::tempdir().expect("Failed to create temp directory")
    }

    fn create_executable_file(dir: &path::Path, name: &str) -> PathBuf {
        let file_path = dir.join(name);
        let mut file = File::create(&file_path).expect("Failed to create file");
        writeln!(file, "#!/bin/sh\necho test").expect("Failed to write to file");

        let mut perms = fs::metadata(&file_path).expect("Failed to get metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&file_path, perms).expect("Failed to set permissions");

        file_path
    }

    fn create_non_executable_file(dir: &path::Path, name: &str) -> PathBuf {
        let file_path = dir.join(name);
        let mut file = File::create(&file_path).expect("Failed to create file");
        writeln!(file, "not executable").expect("Failed to write to file");
        file_path
    }

    #[test]
    fn test_find_matching_executables_empty_partial() {
        let temp_dir = create_test_directory();
        create_executable_file(temp_dir.path(), "test1");

        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let matches = path.find_matching_executables("");
        assert_eq!(matches, Vec::<String>::new());
    }

    #[test]
    fn test_find_matching_executables_single_match() {
        let temp_dir = create_test_directory();
        create_executable_file(temp_dir.path(), "cat");
        create_executable_file(temp_dir.path(), "ls");

        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let matches = path.find_matching_executables("ca");
        assert_eq!(matches, vec!["cat"]);
    }

    #[test]
    fn test_find_matching_executables_multiple_matches() {
        let temp_dir = create_test_directory();
        create_executable_file(temp_dir.path(), "cat");
        create_executable_file(temp_dir.path(), "cargo");
        create_executable_file(temp_dir.path(), "cal");
        create_executable_file(temp_dir.path(), "ls");

        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let matches = path.find_matching_executables("ca");
        assert_eq!(matches, vec!["cal", "cargo", "cat"]);
    }

    #[test]
    fn test_find_matching_executables_no_matches() {
        let temp_dir = create_test_directory();
        create_executable_file(temp_dir.path(), "cat");
        create_executable_file(temp_dir.path(), "ls");

        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let matches = path.find_matching_executables("xyz");
        assert_eq!(matches, Vec::<String>::new());
    }

    #[test]
    fn test_find_matching_executables_ignores_non_executables() {
        let temp_dir = create_test_directory();
        create_executable_file(temp_dir.path(), "cat");
        create_non_executable_file(temp_dir.path(), "cargo");

        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let matches = path.find_matching_executables("ca");
        assert_eq!(matches, vec!["cat"]);
    }

    #[test]
    fn test_find_matching_executables_deduplicates_across_directories() {
        let temp_dir1 = create_test_directory();
        let temp_dir2 = create_test_directory();

        create_executable_file(temp_dir1.path(), "cat");
        create_executable_file(temp_dir1.path(), "cargo");
        create_executable_file(temp_dir2.path(), "cat");

        let path = Path {
            directories: vec![
                temp_dir1.path().to_str().unwrap().to_string(),
                temp_dir2.path().to_str().unwrap().to_string()
            ]
        };

        let matches = path.find_matching_executables("ca");
        assert_eq!(matches, vec!["cargo", "cat"]);
    }

    #[test]
    fn test_find_matching_executables_invalid_directory() {
        let path = Path {
            directories: vec!["/nonexistent/directory".to_string()]
        };

        let matches = path.find_matching_executables("ca");
        assert_eq!(matches, Vec::<String>::new());
    }

    #[test]
    fn test_find_matching_executables_exact_match() {
        let temp_dir = create_test_directory();
        create_executable_file(temp_dir.path(), "cat");

        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let matches = path.find_matching_executables("cat");
        assert_eq!(matches, vec!["cat"]);
    }

    #[test]
    fn test_resolve_piped_commands_no_pipe() {
        let temp_dir = create_test_directory();
        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let mut cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: None,
        };

        let result = path.resolve_piped_commands(&mut cmd);
        assert!(result.is_ok());
        assert_eq!(cmd.command, "echo");
    }

    #[test]
    fn test_resolve_piped_commands_single_builtin() {
        let temp_dir = create_test_directory();
        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let mut cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(ParsedCommand {
                command: "pwd".to_string(),
                args: vec![],
                stdout_redirect: None,
                stderr_redirect: None,
                piped_command: None,
            })),
        };

        let result = path.resolve_piped_commands(&mut cmd);
        assert!(result.is_ok());
        assert_eq!(cmd.piped_command.as_ref().unwrap().command, "pwd");
    }

    #[test]
    fn test_resolve_piped_commands_single_external() {
        let temp_dir = create_test_directory();
        let cat_path = create_executable_file(temp_dir.path(), "cat");

        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let mut cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(ParsedCommand {
                command: "cat".to_string(),
                args: vec![],
                stdout_redirect: None,
                stderr_redirect: None,
                piped_command: None,
            })),
        };

        let result = path.resolve_piped_commands(&mut cmd);
        assert!(result.is_ok());
        assert_eq!(cmd.piped_command.as_ref().unwrap().command, cat_path.to_str().unwrap());
    }

    #[test]
    fn test_resolve_piped_commands_external_not_found() {
        let temp_dir = create_test_directory();
        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let mut cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(ParsedCommand {
                command: "nonexistent".to_string(),
                args: vec![],
                stdout_redirect: None,
                stderr_redirect: None,
                piped_command: None,
            })),
        };

        let result = path.resolve_piped_commands(&mut cmd);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "nonexistent");
    }

    #[test]
    fn test_resolve_piped_commands_multiple_externals() {
        let temp_dir = create_test_directory();
        let cat_path = create_executable_file(temp_dir.path(), "cat");
        let grep_path = create_executable_file(temp_dir.path(), "grep");

        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let mut cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(ParsedCommand {
                command: "cat".to_string(),
                args: vec![],
                stdout_redirect: None,
                stderr_redirect: None,
                piped_command: Some(Box::new(ParsedCommand {
                    command: "grep".to_string(),
                    args: vec!["hello".to_string()],
                    stdout_redirect: None,
                    stderr_redirect: None,
                    piped_command: None,
                })),
            })),
        };

        let result = path.resolve_piped_commands(&mut cmd);
        assert!(result.is_ok());
        assert_eq!(cmd.piped_command.as_ref().unwrap().command, cat_path.to_str().unwrap());
        assert_eq!(
            cmd.piped_command.as_ref().unwrap().piped_command.as_ref().unwrap().command,
            grep_path.to_str().unwrap()
        );
    }

    #[test]
    fn test_resolve_piped_commands_mixed_builtin_and_external() {
        let temp_dir = create_test_directory();
        let cat_path = create_executable_file(temp_dir.path(), "cat");

        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let mut cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(ParsedCommand {
                command: "cat".to_string(),
                args: vec![],
                stdout_redirect: None,
                stderr_redirect: None,
                piped_command: Some(Box::new(ParsedCommand {
                    command: "pwd".to_string(),
                    args: vec![],
                    stdout_redirect: None,
                    stderr_redirect: None,
                    piped_command: None,
                })),
            })),
        };

        let result = path.resolve_piped_commands(&mut cmd);
        assert!(result.is_ok());
        assert_eq!(cmd.piped_command.as_ref().unwrap().command, cat_path.to_str().unwrap());
        assert_eq!(
            cmd.piped_command.as_ref().unwrap().piped_command.as_ref().unwrap().command,
            "pwd"
        );
    }

    #[test]
    fn test_resolve_piped_commands_error_in_middle_of_chain() {
        let temp_dir = create_test_directory();
        let _cat_path = create_executable_file(temp_dir.path(), "cat");

        let path = Path {
            directories: vec![temp_dir.path().to_str().unwrap().to_string()]
        };

        let mut cmd = ParsedCommand {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            stdout_redirect: None,
            stderr_redirect: None,
            piped_command: Some(Box::new(ParsedCommand {
                command: "cat".to_string(),
                args: vec![],
                stdout_redirect: None,
                stderr_redirect: None,
                piped_command: Some(Box::new(ParsedCommand {
                    command: "nonexistent".to_string(),
                    args: vec![],
                    stdout_redirect: None,
                    stderr_redirect: None,
                    piped_command: None,
                })),
            })),
        };

        let result = path.resolve_piped_commands(&mut cmd);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "nonexistent");
    }
}