use std::env;
use std::path::{Path, PathBuf};

pub(crate) fn run(args: &[&str]) -> Result<(), anyhow::Error> {
    let destination = determine_destination(args)?;
    env::set_current_dir(destination.clone())
        .map_err(|_| anyhow::anyhow!("cd: {}: No such file or directory", destination.to_string_lossy()))?;
    Ok(())
}

fn determine_destination(args: &[&str]) -> Result<PathBuf, anyhow::Error> {
    let home_directory = Path::new(&env::var("HOME").unwrap_or_else(|_| "/".to_string())).to_path_buf();

    if args.is_empty() {
        return Ok(home_directory);
    }

    let destination = args[0].trim().to_string();
    let destination_parts = destination.split('/');
    let mut current_directory = env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
    
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

    Ok(current_directory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_home_directory() -> Result<PathBuf, anyhow::Error> {
        let home_var = env::var("HOME").unwrap_or_else(|_| "/".to_string());
        Ok(Path::new(&home_var).to_path_buf())
    }

    fn get_current_directory() -> Result<PathBuf, anyhow::Error> {
        env::current_dir().map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))
    }

    #[test]
    fn test_cd_no_args_goes_to_home() -> Result<(), anyhow::Error> {
        let home = get_home_directory()?;
        let result = determine_destination(&[])?;
        assert_eq!(result, PathBuf::from(home));
        Ok(())
    }

    #[test]
    fn test_cd_current_directory() -> Result<(), anyhow::Error> {
        let current = get_current_directory()?;
        let result = determine_destination(&["."])?;
        assert_eq!(result, current);
        Ok(())
    }

    #[test]
    fn test_cd_parent_directory() -> Result<(), anyhow::Error> {
        let mut expected = get_current_directory()?;
        expected.pop();
        let result = determine_destination(&[".."])?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_cd_absolute_path() -> Result<(), anyhow::Error> {
        let result = determine_destination(&["/usr/bin"])?;
        assert_eq!(result, PathBuf::from("/usr/bin"));
        Ok(())
    }

    #[test]
    fn test_cd_absolute_root() -> Result<(), anyhow::Error> {
        let result = determine_destination(&["/"])?;
        assert_eq!(result, PathBuf::from("/"));
        Ok(())
    }

    #[test]
    fn test_cd_tilde_expansion() -> Result<(), anyhow::Error> {
        let home = get_home_directory()?;
        let result = determine_destination(&["~"])?;
        assert_eq!(result, PathBuf::from(home));
        Ok(())
    }

    #[test]
    fn test_cd_tilde_with_path() -> Result<(), anyhow::Error> {
        let home = get_home_directory()?;
        let mut expected = PathBuf::from(home);
        expected.push("Documents");
        let result = determine_destination(&["~/Documents"])?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_cd_relative_path() -> Result<(), anyhow::Error> {
        let mut expected = get_current_directory()?;
        expected.push("child_directory");
        let result = determine_destination(&["child_directory"])?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_cd_complex_relative_path() -> Result<(), anyhow::Error> {
        let mut expected = get_current_directory()?;
        expected.push("src");
        expected.pop();
        expected.push("target");
        let result = determine_destination(&["src/../target"])?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_cd_multiple_parent_directories() -> Result<(), anyhow::Error> {
        let mut expected = get_current_directory()?;
        expected.pop();
        expected.pop();
        let result = determine_destination(&["../.."])?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_cd_with_current_and_parent_mix() -> Result<(), anyhow::Error> {
        let mut expected = get_current_directory()?;
        expected.push("src");
        expected.pop();
        expected.push("target");
        expected.push("debug");
        let result = determine_destination(&["src/../target/./debug"])?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_cd_empty_string_components() -> Result<(), anyhow::Error> {
        let result = determine_destination(&["//usr//bin//"])?;
        assert_eq!(result, PathBuf::from("/usr/bin"));
        Ok(())
    }

    #[test]
    fn test_cd_whitespace_trimming() -> Result<(), anyhow::Error> {
        // Test that arguments are trimmed
        let result = determine_destination(&["  /usr/bin  "])?;
        assert_eq!(result, PathBuf::from("/usr/bin"));
        Ok(())
    }
}