use std::{ fs::File, io::{BufRead, BufReader, BufWriter, Write}, path::PathBuf };
use anyhow::Error;

pub(crate) struct History {
    commands: Vec<String>
}

impl History {
    pub(crate) fn new() -> Self {
        History { commands: Vec::new() }
    }

    pub(crate) fn read_from_file(&mut self, path: &PathBuf) -> Result<(), Error> {
        let file = File::open(path)?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            if !line.is_empty() {
                self.append(&line);
            }
        }
        Ok(())
    }

    pub(crate) fn write_to_file(&mut self, path: &PathBuf) -> Result<(), Error> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for command in self.commands.iter() {
            writer.write(command.as_bytes())?;
            writer.write(&[b'\n'])?;
        }
        Ok(())
    }

    pub(crate) fn append(&mut self, command: &str) -> &Self {
        self.commands.push(command.to_string());
        self
    }

    pub(crate) fn show(&self, limit: Option<usize>) -> String {
        let mut result = String::new();
        let starting_index = match limit {
            Some(n) => {
                self.commands.len() - n.min(self.commands.len())
            },
            None => 0,
        };
        for (idx, command) in self.commands.iter().skip(starting_index).enumerate() {
            result.push_str(&format!("{}  {}\n", starting_index + idx + 1, command));
        }
        result
    }

    pub(crate) fn get_last_command_by_idx(&self, index_from_end: usize) -> Option<&str> {
        if index_from_end >= self.commands.len() {
            None
        } else {
            let index = self.commands.len() - 1 - index_from_end;
            Some(&self.commands[index])
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.commands.len()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_history_shows_executed_commands() {
        let mut history = History::new();
        history.append("echo hello");
        history.append("echo world");
        history.append("invalid_command");
        assert_eq!(history.show(None), "1  echo hello\n2  echo world\n3  invalid_command\n")
    }

    #[test]
    fn test_history_respects_limit_argument() {
        let mut history = History::new();
        history.append("echo hello");
        history.append("echo world");
        history.append("invalid_command");
        assert_eq!(history.show(Some(2)), "2  echo world\n3  invalid_command\n")
    }

    #[test]
    fn test_history_with_too_large_limit_works() {
        let mut history = History::new();
        history.append("echo hello");
        history.append("echo world");
        history.append("invalid_command");
        assert_eq!(history.show(Some(5)), "1  echo hello\n2  echo world\n3  invalid_command\n")
    }

    #[test]
    fn test_get_reverse_most_recent() {
        let mut history = History::new();
        history.append("echo hello");
        history.append("echo world");
        history.append("invalid_command");
        assert_eq!(history.get_last_command_by_idx(0), Some("invalid_command"));
    }

    #[test]
    fn test_get_reverse_second_most_recent() {
        let mut history = History::new();
        history.append("echo hello");
        history.append("echo world");
        history.append("invalid_command");
        assert_eq!(history.get_last_command_by_idx(1), Some("echo world"));
    }

    #[test]
    fn test_get_reverse_oldest() {
        let mut history = History::new();
        history.append("echo hello");
        history.append("echo world");
        history.append("invalid_command");
        assert_eq!(history.get_last_command_by_idx(2), Some("echo hello"));
    }

    #[test]
    fn test_get_reverse_out_of_bounds() {
        let mut history = History::new();
        history.append("echo hello");
        assert_eq!(history.get_last_command_by_idx(1), None);
        assert_eq!(history.get_last_command_by_idx(100), None);
    }

    #[test]
    fn test_len() {
        let mut history = History::new();
        assert_eq!(history.len(), 0);
        history.append("echo hello");
        assert_eq!(history.len(), 1);
        history.append("echo world");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_read_from_file_basic() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "echo hello")?;
        writeln!(temp_file, "echo world")?;
        writeln!(temp_file, "pwd")?;
        temp_file.flush()?;

        let path = PathBuf::from(temp_file.path());
        let mut history = History::new();
        history.read_from_file(&path)?;

        assert_eq!(history.len(), 3);
        assert_eq!(history.get_last_command_by_idx(2), Some("echo hello"));
        assert_eq!(history.get_last_command_by_idx(1), Some("echo world"));
        assert_eq!(history.get_last_command_by_idx(0), Some("pwd"));
        Ok(())
    }

    #[test]
    fn test_read_from_file_skips_empty_lines() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "echo hello")?;
        writeln!(temp_file, "")?;
        writeln!(temp_file, "echo world")?;
        writeln!(temp_file, "")?;
        writeln!(temp_file, "pwd")?;
        temp_file.flush()?;

        let path = PathBuf::from(temp_file.path());
        let mut history = History::new();
        history.read_from_file(&path)?;

        assert_eq!(history.len(), 3);
        assert_eq!(history.show(None), "1  echo hello\n2  echo world\n3  pwd\n");
        Ok(())
    }

    #[test]
    fn test_read_from_file_empty_file() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.flush()?;

        let path = PathBuf::from(temp_file.path());
        let mut history = History::new();
        history.read_from_file(&path)?;

        assert_eq!(history.len(), 0);
        Ok(())
    }

    #[test]
    fn test_read_from_file_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/path/to/file.txt");
        let mut history = History::new();
        let result = history.read_from_file(&path);

        assert!(result.is_err());
    }

    #[test]
    fn test_read_from_file_preserves_existing_commands() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "file_cmd1")?;
        writeln!(temp_file, "file_cmd2")?;
        temp_file.flush()?;

        let path = PathBuf::from(temp_file.path());
        let mut history = History::new();
        history.append("existing_cmd");

        history.read_from_file(&path)?;

        assert_eq!(history.len(), 3);
        assert_eq!(history.get_last_command_by_idx(2), Some("existing_cmd"));
        assert_eq!(history.get_last_command_by_idx(1), Some("file_cmd1"));
        assert_eq!(history.get_last_command_by_idx(0), Some("file_cmd2"));
        Ok(())
    }

    #[test]
    fn test_read_from_file_with_special_characters() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "echo 'hello world'")?;
        writeln!(temp_file, "ls -la /tmp")?;
        writeln!(temp_file, "echo \"test\" | grep test")?;
        temp_file.flush()?;

        let path = PathBuf::from(temp_file.path());
        let mut history = History::new();
        history.read_from_file(&path)?;

        assert_eq!(history.len(), 3);
        assert_eq!(history.get_last_command_by_idx(2), Some("echo 'hello world'"));
        assert_eq!(history.get_last_command_by_idx(1), Some("ls -la /tmp"));
        assert_eq!(history.get_last_command_by_idx(0), Some("echo \"test\" | grep test"));
        Ok(())
    }
}