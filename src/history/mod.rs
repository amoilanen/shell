use std::usize::MAX;

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
        let mut result: String = String::new();
        for (idx, command) in self.commands.iter().take(limit.unwrap_or(MAX)).enumerate() {
            result.push_str(&format!("{}  {}\n", (idx + 1), command));
        }
        result
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
        assert_eq!(history.show(Some(2)), "1  echo hello\n2  echo world\n")
    }
}