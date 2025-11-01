use std::io::{self, Write, Read};
use termios::{Termios, tcsetattr, TCSANOW, ECHO, ICANON, IEXTEN, ISIG, VMIN, VTIME};
use crate::input::autocompletion::AutoCompletion;
use crate::history::History;

pub mod autocompletion;

// Terminal control sequences
const BACKSPACE_ERASE_SEQUENCE: &str = "\x08 \x08";
const PROMPT: &str = "$ ";

// Special characters
const NEWLINE: char = '\n';
const CARRIAGE_RETURN: char = '\r';
const TAB: char = '\t';
const BACKSPACE: char = '\u{7f}';
const DELETE: char = '\u{0008}';
const BEEP: char = '\x07';
const CTRL_C: char = '\u{0003}';
const ESC: char = '\u{001b}';

pub fn read_line_with_completion(autocomplete: &AutoCompletion, history: &History) -> Result<String, anyhow::Error> {
    let raw_mode = RawMode::enable()?;
    let mut input = String::new();
    let mut stdin = io::stdin();
    let mut buffer = [0; 1];
    let mut last_tab_input: Option<String> = None;
    let mut history_index: Option<usize> = None;

    loop {
        stdin.read_exact(&mut buffer)?;
        let ch = buffer[0] as char;

        match ch {
            NEWLINE | CARRIAGE_RETURN => {
                println!();
                drop(raw_mode);
                return Ok(input);
            }
            CTRL_C => {
                println!("^C");
                drop(raw_mode);
                std::process::exit(0);
            }
            BACKSPACE | DELETE => {
                last_tab_input = None;
                history_index = None;
                handle_backspace(&mut input)?;
            }
            TAB => {
                history_index = None;
                handle_tab_completion(&mut input, autocomplete, &mut last_tab_input)?;
            }
            ESC => {
                if let Some(arrow) = read_arrow_sequence(&mut stdin)? {
                    match arrow {
                        ArrowKey::Up => {
                            handle_history_up(&mut input, &mut history_index, history)?;
                        }
                        ArrowKey::Down => {
                            handle_history_down(&mut input, &mut history_index, history)?;
                        }
                    }
                }
            }
            _ => {
                last_tab_input = None;
                history_index = None;
                handle_regular_char(&mut input, ch)?;
            }
        }
    }
}

fn handle_backspace(input: &mut String) -> Result<(), anyhow::Error> {
    if !input.is_empty() {
        input.pop();
        print_and_flush(BACKSPACE_ERASE_SEQUENCE)?;
    }
    Ok(())
}

fn handle_regular_char(input: &mut String, ch: char) -> Result<(), anyhow::Error> {
    print_and_flush(&ch.to_string())?;
    input.push(ch);
    Ok(())
}

fn handle_tab_completion(input: &mut String, autocomplete: &AutoCompletion, last_tab_input: &mut Option<String>) -> Result<(), anyhow::Error> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if let Some(last_word) = words.last() {
        let last_word = last_word.to_string();
        let matches = autocomplete.complete(&last_word);
        process_completion_matches(input, &last_word, matches, autocomplete, last_tab_input)?;
    }
    Ok(())
}

fn process_completion_matches(
    input: &mut String,
    last_word: &str,
    matches: Vec<String>,
    autocomplete: &AutoCompletion,
    last_tab_input: &mut Option<String>,
) -> Result<(), anyhow::Error> {
    match matches.len() {
        0 => {
            print_and_flush(format!("{}", BEEP).as_str())?;
        },
        1 => handle_single_completion(input, last_word, &matches[0])?,
        _ => handle_multiple_completions(input, last_word, matches, autocomplete, last_tab_input)?,
    }
    Ok(())
}

fn handle_single_completion(input: &mut String, last_word: &str, completion: &str) -> Result<(), anyhow::Error> {
    if completion.len() > last_word.len() {
        let to_add = &completion[last_word.len()..];
        let to_add_padded = format!("{} ", to_add);
        print_and_flush(&to_add_padded)?;
        input.push_str(&to_add_padded);
    }
    Ok(())
}

fn handle_multiple_completions(
    input: &mut String,
    last_word: &str,
    matches: Vec<String>,
    autocomplete: &AutoCompletion,
    last_tab_input: &mut Option<String>,
) -> Result<(), anyhow::Error> {
    let is_consecutive_tab = last_tab_input.as_ref() == Some(input);
    
    if let Some(common_prefix) = autocomplete.find_common_prefix(last_word) {
        if common_prefix.len() > last_word.len() {
            let to_add = &common_prefix[last_word.len()..];
            print_and_flush(to_add)?;
            input.push_str(to_add);
            *last_tab_input = Some(input.clone());
        } else if is_consecutive_tab {
            display_matches_and_reprompt(input, &matches)?;
            *last_tab_input = None;
        } else {
            print_and_flush(format!("{}", BEEP).as_str())?;
            *last_tab_input = Some(input.clone());
        }
    } else if is_consecutive_tab {
        display_matches_and_reprompt(input, &matches)?;
        *last_tab_input = None;
    } else {
        print_and_flush(format!("{}", BEEP).as_str())?;
        *last_tab_input = Some(input.clone());
    }
    Ok(())
}

fn display_matches_and_reprompt(input: &str, matches: &[String]) -> Result<(), anyhow::Error> {
    println!();
    for match_str in matches {
        print!("{}  ", match_str);
    }
    print!("\n\r{}{}", PROMPT, input);
    io::stdout().flush()?;
    Ok(())
}

fn print_and_flush(text: &str) -> Result<(), anyhow::Error> {
    print!("{}", text);
    io::stdout().flush()?;
    Ok(())
}

enum ArrowKey {
    Up,
    Down,
}

fn read_arrow_sequence(stdin: &mut io::Stdin) -> Result<Option<ArrowKey>, anyhow::Error> {
    let mut buffer = [0; 1];
    stdin.read_exact(&mut buffer)?;
    if buffer[0] as char != '[' {
        return Ok(None);
    }
    stdin.read_exact(&mut buffer)?;
    match buffer[0] as char {
        'A' => Ok(Some(ArrowKey::Up)),
        'B' => Ok(Some(ArrowKey::Down)),
        _ => Ok(None),
    }
}

fn clear_line(input: &str) -> Result<(), anyhow::Error> {
    // Move cursor to beginning of input (after prompt)
    for _ in 0..input.len() {
        print!("\x08");
    }
    // Clear from cursor to end of line
    print!("\x1b[K");
    io::stdout().flush()?;
    Ok(())
}

fn handle_history_up(input: &mut String, history_index: &mut Option<usize>, history: &History) -> Result<(), anyhow::Error> {
    if history.len() == 0 {
        return Ok(());
    }

    let new_index = match history_index {
        None => Some(0),
        Some(idx) => {
            if *idx + 1 < history.len() {
                Some(*idx + 1)
            } else {
                None
            }
        }
    };
    match new_index {
        None => (),
        Some(new_index) => {
            if let Some(cmd) = history.get_last_command_by_idx(new_index) {
                clear_line(input)?;
                input.clear();
                input.push_str(cmd);
                print_and_flush(cmd)?;
                *history_index = Some(new_index);
            }
        }
    }

    Ok(())
}

fn handle_history_down(input: &mut String, history_index: &mut Option<usize>, history: &History) -> Result<(), anyhow::Error> {
    let new_index = match history_index {
        None => return Ok(()), // Not navigating history, do nothing
        Some(idx) => {
            if *idx == 0 {
                None // Go back to empty line
            } else {
                Some(*idx - 1) // Navigate to newer command
            }
        }
    };

    match new_index {
        None => {
            // Back to empty line
            clear_line(input)?;
            input.clear();
            *history_index = None;
        }
        Some(idx) => {
            if let Some(cmd) = history.get_last_command_by_idx(idx) {
                clear_line(input)?;
                input.clear();
                input.push_str(cmd);
                print_and_flush(cmd)?;
                *history_index = Some(idx);
            }
        }
    }

    Ok(())
}

struct RawMode {
    original: Termios,
}

impl RawMode {
    fn enable() -> Result<Self, anyhow::Error> {
        let stdin_file_descriptor = 0;
        let original = Termios::from_fd(stdin_file_descriptor)?;
        let mut raw = original.clone();
        // Disable canonical mode, echo, signals, and special chars
        raw.c_lflag &= !(ICANON | ECHO | IEXTEN | ISIG);
        // Ensure reads return as soon as 1 byte is available
        raw.c_cc[VMIN] = 1;
        raw.c_cc[VTIME] = 0;
        tcsetattr(stdin_file_descriptor, TCSANOW, &raw)?;
        Ok(Self { original })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let stdin_file_descriptor = 0;
        let _ = tcsetattr(stdin_file_descriptor, TCSANOW, &self.original);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_autocomplete() -> AutoCompletion {
        AutoCompletion::new(vec!["echo", "exit", "export", "cd", "cat", "cargo"])
    }

    #[test]
    fn test_handle_single_completion() {
        let mut input = String::from("ec");
        let completion = "echo";

        let result = handle_single_completion(&mut input, "ec", completion);
        assert!(result.is_ok());
        assert_eq!(input, "echo ");
    }

    #[test]
    fn test_handle_single_completion_no_extension() {
        let mut input = String::from("echo");
        let completion = "echo";

        let result = handle_single_completion(&mut input, "echo", completion);
        assert!(result.is_ok());
        assert_eq!(input, "echo"); // Should remain unchanged
    }

    #[test]
    fn test_process_completion_matches_no_matches() {
        let mut input = String::from("xyz");
        let autocomplete = create_test_autocomplete();
        let matches = Vec::new();
        let mut last_tab_input = None;

        let result = process_completion_matches(&mut input, "xyz", matches, &autocomplete, &mut last_tab_input);
        assert!(result.is_ok());
        assert_eq!(input, "xyz");
    }

    #[test]
    fn test_process_completion_matches_single_match() {
        let mut input = String::from("ec");
        let autocomplete = create_test_autocomplete();
        let matches = vec!["echo".to_string()];
        let mut last_tab_input = None;

        let result = process_completion_matches(&mut input, "ec", matches, &autocomplete, &mut last_tab_input);
        assert!(result.is_ok());
        assert_eq!(input, "echo ");
    }

    #[test]
    fn test_handle_backspace_with_content() {
        let mut input = String::from("hello");

        // We can't easily test the terminal output, but we can test the string manipulation
        let result = handle_backspace(&mut input);
        assert!(result.is_ok());
        assert_eq!(input, "hell");
    }

    #[test]
    fn test_handle_backspace_empty_input() {
        let mut input = String::new();

        let result = handle_backspace(&mut input);
        assert!(result.is_ok());
        assert_eq!(input, ""); // Should remain empty
    }

    #[test]
    fn test_handle_regular_char() {
        let mut input = String::from("hell");

        let result = handle_regular_char(&mut input, 'o');
        assert!(result.is_ok());
        assert_eq!(input, "hello");
    }

    #[test]
    fn test_handle_tab_completion_no_words() {
        let mut input = String::new();
        let autocomplete = create_test_autocomplete();
        let mut last_tab_input = None;

        let result = handle_tab_completion(&mut input, &autocomplete, &mut last_tab_input);
        assert!(result.is_ok());
        assert_eq!(input, ""); // Should remain empty when no words to complete
    }

    #[test]
    fn test_handle_tab_completion_with_partial_word() {
        let mut input = String::from("ec");
        let autocomplete = create_test_autocomplete();
        let mut last_tab_input = None;

        let result = handle_tab_completion(&mut input, &autocomplete, &mut last_tab_input);
        assert!(result.is_ok());
        assert_eq!(input, "echo "); // Should complete to "echo"
    }

    #[test]
    fn test_handle_tab_completion_with_multiple_words() {
        let mut input = String::from("echo hello ec");
        let autocomplete = create_test_autocomplete();
        let mut last_tab_input = None;

        let result = handle_tab_completion(&mut input, &autocomplete, &mut last_tab_input);
        assert!(result.is_ok());
        assert_eq!(input, "echo hello echo "); // Should complete the last word
    }

    #[test]
    fn test_constants() {
        assert_eq!(BACKSPACE_ERASE_SEQUENCE, "\x08 \x08");
        assert_eq!(PROMPT, "$ ");
        assert_eq!(NEWLINE, '\n');
        assert_eq!(CARRIAGE_RETURN, '\r');
        assert_eq!(TAB, '\t');
        assert_eq!(BACKSPACE, '\u{7f}');
        assert_eq!(DELETE, '\u{0008}');
        assert_eq!(CTRL_C, '\u{0003}');
    }

    #[test]
    fn test_multiple_completions_with_common_prefix() {
        let mut input = String::from("e");
        let autocomplete = create_test_autocomplete();
        let matches = vec!["echo".to_string(), "exit".to_string(), "export".to_string()];
        let mut last_tab_input = None;

        let result = handle_multiple_completions(&mut input, "e", matches, &autocomplete, &mut last_tab_input);
        assert!(result.is_ok());
        assert_eq!(input, "e"); // Should remain "e" since that's the only common prefix
    }

    #[test]
    fn test_multiple_completions_extending_prefix() {
        let mut input = String::from("ex");
        let autocomplete = create_test_autocomplete();
        let matches = vec!["exit".to_string(), "export".to_string()];
        let mut last_tab_input = None;

        let result = handle_multiple_completions(&mut input, "ex", matches, &autocomplete, &mut last_tab_input);
        assert!(result.is_ok());
        // Should extend to common prefix "ex" (no further extension possible)
        assert_eq!(input, "ex");
    }

    #[test]
    fn test_pressing_tab_twice_with_multiple_completions() {
        let mut input = String::from("e");
        let autocomplete = create_test_autocomplete();
        let matches = vec!["echo".to_string(), "exit".to_string(), "export".to_string()];
        let mut last_tab_input = None;

        // First tab press - should set last_tab_input since no common prefix extension
        let result = handle_multiple_completions(&mut input, "e", matches.clone(), &autocomplete, &mut last_tab_input);
        assert!(result.is_ok());
        assert_eq!(input, "e");
        assert_eq!(last_tab_input, Some(String::from("e")));

        // Second tab press (consecutive) - should trigger display of matches and clear last_tab_input
        let result = handle_multiple_completions(&mut input, "e", matches, &autocomplete, &mut last_tab_input);
        assert!(result.is_ok());
        assert_eq!(input, "e");
        assert_eq!(last_tab_input, None);
    }

    #[test]
    fn test_handle_history_up_empty_history() {
        use crate::history::History;
        let history = History::new();
        let mut input = String::new();
        let mut history_index = None;

        let result = handle_history_up(&mut input, &mut history_index, &history);
        assert!(result.is_ok());
        assert_eq!(input, "");
        assert_eq!(history_index, None);
    }

    #[test]
    fn test_handle_history_up_first_command() {
        use crate::history::History;
        let mut history = History::new();
        history.append("echo hello");
        history.append("echo world");

        let mut input = String::new();
        let mut history_index = None;

        let result = handle_history_up(&mut input, &mut history_index, &history);
        assert!(result.is_ok());
        assert_eq!(input, "echo world");
        assert_eq!(history_index, Some(0));
    }

    #[test]
    fn test_handle_history_up_second_command() {
        use crate::history::History;
        let mut history = History::new();
        history.append("echo hello");
        history.append("echo world");

        let mut input = String::new();
        let mut history_index = None;

        let mut result = handle_history_up(&mut input, &mut history_index, &history);
        assert!(result.is_ok());
        result = handle_history_up(&mut input, &mut history_index, &history);
        assert!(result.is_ok());
        assert_eq!(input, "echo hello");
        assert_eq!(history_index, Some(1));
    }

    #[test]
    fn test_handle_history_up_at_oldest() {
        use crate::history::History;
        let mut history = History::new();
        history.append("echo world");
        history.append("echo hello");

        let mut input = String::from("echo world");
        let mut history_index = Some(1);

        // Should not change when already at oldest
        let result = handle_history_up(&mut input, &mut history_index, &history);
        assert!(result.is_ok());
        assert_eq!(input, "echo world");
        assert_eq!(history_index, Some(1));
    }

    #[test]
    fn test_handle_history_down_not_navigating() {
        use crate::history::History;
        let mut history = History::new();
        history.append("echo world");
        history.append("echo hello");

        let mut input = String::from("test");
        let mut history_index = None;

        let result = handle_history_down(&mut input, &mut history_index, &history);
        assert!(result.is_ok());
        assert_eq!(input, "test");
        assert_eq!(history_index, None);
    }

    #[test]
    fn test_handle_history_down_to_empty() {
        use crate::history::History;
        let mut history = History::new();
        history.append("echo hello");

        let mut input = String::from("echo hello");
        let mut history_index = Some(0);

        let result = handle_history_down(&mut input, &mut history_index, &history);
        assert!(result.is_ok());
        assert_eq!(input, "");
        assert_eq!(history_index, None);
    }

    #[test]
    fn test_handle_history_down_to_newer() {
        use crate::history::History;
        let mut history = History::new();
        history.append("echo hello");
        history.append("echo world");

        let mut input = String::from("echo hello");
        let mut history_index = Some(1);

        let result = handle_history_down(&mut input, &mut history_index, &history);
        assert!(result.is_ok());
        assert_eq!(input, "echo world");
        assert_eq!(history_index, Some(0));
    }

    #[test]
    fn test_clear_line_empty_input() {
        let input = "";
        let result = clear_line(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_line_with_content() {
        let input = "echo hello";
        let result = clear_line(input);
        assert!(result.is_ok());
    }
}