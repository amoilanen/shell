pub(crate) struct History {
    commands: Vec<String>
}

impl History {
    pub(crate) fn new() -> Self {
        History { commands: Vec::new() }
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
}