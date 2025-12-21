use std::io::{self, Write};
use crate::history::History;
use anyhow::anyhow;

pub(crate) fn generate_output(args: &[&str], history: &History) -> Result<Vec<u8>, anyhow::Error> {
    let limit = args.get(0)
        .map(|s| s.trim().parse::<usize>())
        .transpose()?;
    Ok(history.show(limit).into_bytes())
}

pub(crate) fn run(args: &[&str], history: &mut History) -> Result<(), anyhow::Error> {
    if let Some(r_option_position) = args.iter().position(|&arg| arg == "-r") {
        if let Some(history_file_path) = args.get(r_option_position + 1) {
            history.read_from_file(&history_file_path.into())
        } else {
            Err(anyhow!("Expected file option, found nothing, args: {:?}", args))
        }
    } else if let Some(w_option_position) = args.iter().position(|&arg| arg == "-w") {
        if let Some(history_file_path) = args.get(w_option_position + 1) {
            history.write_to_file(&history_file_path.into())
        } else {
            Err(anyhow!("Expected file option, found nothing, args: {:?}", args))
        }
    } else if let Some(w_option_position) = args.iter().position(|&arg| arg == "-a") {
        if let Some(history_file_path) = args.get(w_option_position + 1) {
            history.append_to_file(&history_file_path.into())
        } else {
            Err(anyhow!("Expected file option, found nothing, args: {:?}", args))
        }
    } else {
        let output = generate_output(args, history)?;
        print!("{}", String::from_utf8_lossy(&output));
        io::stdout().flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_generate_output_no_args() -> Result<(), anyhow::Error> {
        let mut history = History::new();
        history.append("echo hello");
        history.append("pwd");
        history.append("ls");

        let output = generate_output(&[], &history)?;
        let output_str = String::from_utf8(output)?;

        assert_eq!(output_str, "1  echo hello\n2  pwd\n3  ls\n");
        Ok(())
    }

    #[test]
    fn test_generate_output_with_limit() -> Result<(), anyhow::Error> {
        let mut history = History::new();
        history.append("echo hello");
        history.append("pwd");
        history.append("ls");

        let output = generate_output(&["2"], &history)?;
        let output_str = String::from_utf8(output)?;

        assert_eq!(output_str, "2  pwd\n3  ls\n");
        Ok(())
    }

    #[test]
    fn test_generate_output_invalid_limit() {
        let mut history = History::new();
        history.append("echo hello");

        let result = generate_output(&["not_a_number"], &history);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_history_from_file() -> Result<(), anyhow::Error> {
        let mut history = History::new();
        let mut file = NamedTempFile::new()?;
        writeln!(file, "echo first")?;
        writeln!(file, "echo second")?;
        writeln!(file, "pwd")?;

        run(&["-r", file.path().to_string_lossy().as_ref()], &mut history)?;

        assert_eq!(history.len(), 3);
        let output = generate_output(&[], &history)?;
        let output_str = String::from_utf8(output)?;
        assert_eq!(output_str, "1  echo first\n2  echo second\n3  pwd\n");

        Ok(())
    }

    #[test]
    fn test_write_history_to_file() -> Result<(), anyhow::Error> {
        let mut history = History::new();
        let file = NamedTempFile::new()?;
        history.append("ls");
        history.append("cat README.md");
        history.append("pwd");

        run(&["-w", file.path().to_string_lossy().as_ref()], &mut history)?;

        let written_history = fs::read_to_string(file)?;

        assert_eq!(written_history, "ls\ncat README.md\npwd\n");
        Ok(())
    }

    #[test]
    fn test_write_empty_history_to_file() -> Result<(), anyhow::Error> {
        let mut history = History::new();
        let file = NamedTempFile::new()?;

        run(&["-w", file.path().to_string_lossy().as_ref()], &mut history)?;

        let written_history = fs::read_to_string(file)?;

        assert_eq!(written_history, "");
        Ok(())
    }

    #[test]
    fn test_run_r_flag_without_path() -> Result<(), anyhow::Error> {
        let mut history = History::new();
        history.append("echo hello");

       let err = run(&["-r"], &mut history).unwrap_err();

        assert_eq!(history.len(), 1);
        assert!(format!("{}", err).contains("Expected file option"));
        Ok(())
    }

    #[test]
    fn test_append_history_to_file() -> Result<(), anyhow::Error> {
        let mut history = History::new();
        let file = NamedTempFile::new()?;
        history.append("ls");
        history.append("cat README.md");
        history.append("pwd");

        run(&["-a", file.path().to_string_lossy().as_ref()], &mut history)?;

        let written_history = fs::read_to_string(file)?;

        assert_eq!(written_history, "ls\ncat README.md\npwd\n");
        Ok(())
    }

    #[test]
    fn test_multiple_calls_to_append_history() -> Result<(), anyhow::Error> {
        let mut history = History::new();
        let file = NamedTempFile::new()?;
        history.append("ls");
        history.append("cat README.md");
        history.append("pwd");

        for _ in 1..5 {
            run(&["-a", file.path().to_string_lossy().as_ref()], &mut history)?;
        }

        let written_history = fs::read_to_string(file)?;

        assert_eq!(written_history, "ls\ncat README.md\npwd\n");
        Ok(())
    }

    #[test]
    fn test_append_empty_history_to_file() -> Result<(), anyhow::Error> {
        let mut history = History::new();
        let file = NamedTempFile::new()?;

        run(&["-a", file.path().to_string_lossy().as_ref()], &mut history)?;

        let written_history = fs::read_to_string(file)?;

        assert_eq!(written_history, "");
        Ok(())
    }

    #[test]
    fn test_append_empty_history_to_file_after_written_to_file() -> Result<(), anyhow::Error> {
        let mut history = History::new();
        let file = NamedTempFile::new()?;
        let file_path = file.path().to_string_lossy();
        history.append("ls");
        history.append("cat README.md");
        history.append("pwd");

        run(&["-w", file_path.as_ref()], &mut history)?;

        let mut written_history = fs::read_to_string(&file)?;
        assert_eq!(written_history, "ls\ncat README.md\npwd\n");

        history.append("echo abc");
        history.append("echo def");

        run(&["-a", file_path.as_ref()], &mut history)?;

        written_history = fs::read_to_string(&file)?;
        assert_eq!(written_history, "ls\ncat README.md\npwd\necho abc\necho def\n");

        Ok(())
    }

    #[test]
    fn test_append_empty_history_to_file_after_appended_to_file() -> Result<(), anyhow::Error> {
        let mut history = History::new();
        let file = NamedTempFile::new()?;
        let file_path = file.path().to_string_lossy();
        history.append("ls");
        history.append("cat README.md");
        history.append("pwd");

        run(&["-a", file_path.as_ref()], &mut history)?;

        let mut written_history = fs::read_to_string(&file)?;
        assert_eq!(written_history, "ls\ncat README.md\npwd\n");

        history.append("echo abc");
        history.append("echo def");

        run(&["-a", file_path.as_ref()], &mut history)?;

        written_history = fs::read_to_string(&file)?;
        assert_eq!(written_history, "ls\ncat README.md\npwd\necho abc\necho def\n");

        Ok(())
    }
}
