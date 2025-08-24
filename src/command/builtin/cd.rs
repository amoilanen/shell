use std::env;
use std::path::{Path, PathBuf};

pub(crate) fn run(args: &[&str]) -> () {
    let destination = determine_destination(args);
    match env::set_current_dir(destination.clone()) {
        Ok(_) => (),
        Err(_) =>
            println!("cd: {}: No such file or directory", destination.to_string_lossy())
    }
}

fn determine_destination(args: &[&str]) -> PathBuf {
    let home_directory = Path::new(&env::var("HOME").unwrap_or_else(|_| "/".to_string())).to_path_buf();
    
    if args.is_empty() {
        return home_directory;
    }

    let destination = args[0].trim().to_string();
    let destination_parts = destination.split('/');
    let mut current_directory = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    
    for (idx, destination_part) in destination_parts.enumerate() {
        match destination_part {
            "." => {
                // Do nothing, already in the correct directory (current directory)
            }
            ".." => {
                current_directory.pop();
            }
            "" if idx == 0 => {
                current_directory = Path::new("/").to_path_buf();
            }
            "~" if idx == 0 => {
                current_directory = home_directory.clone();
            }
            _ => {
                current_directory.push(destination_part);
            }
        }
    }
    
    current_directory
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_home_directory() -> PathBuf {
        Path::new(&env::var("HOME").unwrap_or_else(|_| "/".to_string())).to_path_buf()
    }

    fn get_current_directory() -> PathBuf {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
    }

    #[test]
    fn test_cd_no_args_goes_to_home() {
        let home = get_home_directory();
        let result = determine_destination(&[]);
        assert_eq!(result, PathBuf::from(home));
    }

    #[test]
    fn test_cd_current_directory() {
        let current = get_current_directory();
        let result = determine_destination(&["."]);
        assert_eq!(result, current);
    }

    #[test]
    fn test_cd_parent_directory() {
        let mut expected = get_current_directory();
        expected.pop();
        let result = determine_destination(&[".."]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cd_absolute_path() {
        let result = determine_destination(&["/usr/bin"]);
        assert_eq!(result, PathBuf::from("/usr/bin"));
    }

    #[test]
    fn test_cd_absolute_root() {
        let result = determine_destination(&["/"]);
        assert_eq!(result, PathBuf::from("/"));
    }

    #[test]
    fn test_cd_tilde_expansion() {
        let home = get_home_directory();
        let result = determine_destination(&["~"]);
        assert_eq!(result, PathBuf::from(home));
    }

    #[test]
    fn test_cd_tilde_with_path() {
        let home = get_home_directory();
        let mut expected = PathBuf::from(home);
        expected.push("Documents");
        let result = determine_destination(&["~/Documents"]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cd_relative_path() {
        let mut expected = get_current_directory();
        expected.push("child_directory");
        let result = determine_destination(&["child_directory"]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cd_complex_relative_path() {
        let mut expected = get_current_directory();
        expected.push("src");
        expected.pop();
        expected.push("target");
        let result = determine_destination(&["src/../target"]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cd_multiple_parent_directories() {
        let mut expected = get_current_directory();
        expected.pop();
        expected.pop();
        let result = determine_destination(&["../.."]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cd_with_current_and_parent_mix() {
        let mut expected = get_current_directory();
        expected.push("src");
        expected.pop();
        expected.push("target");
        expected.push("debug");
        let result = determine_destination(&["src/../target/./debug"]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cd_empty_string_components() {
        let result = determine_destination(&["//usr//bin//"]);
        assert_eq!(result, PathBuf::from("/usr/bin"));
    }

    #[test]
    fn test_cd_whitespace_trimming() {
        // Test that arguments are trimmed
        let result = determine_destination(&["  /usr/bin  "]);
        assert_eq!(result, PathBuf::from("/usr/bin"));
    }
}