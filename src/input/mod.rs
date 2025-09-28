use std::io::{self, Write, Read};
use termios::{Termios, tcsetattr, TCSANOW, ECHO, ICANON, IEXTEN, ISIG, VMIN, VTIME};
use crate::input::autocompletion::AutoCompletion;

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

pub fn read_line_with_completion(autocomplete: &AutoCompletion) -> Result<String, anyhow::Error> {
    let raw_mode = RawMode::enable()?;
    let mut input = String::new();
    let mut stdin = io::stdin();
    let mut buffer = [0; 1];

    loop {
        stdin.read_exact(&mut buffer)?;
        let ch = buffer[0] as char;

        match ch {
            NEWLINE | CARRIAGE_RETURN => {
                println!();
                // Explicitly drop raw mode before returning to ensure terminal is restored
                drop(raw_mode);
                return Ok(input);
            }
            BACKSPACE | DELETE => {
                handle_backspace(&mut input)?;
            }
            TAB => {
                handle_tab_completion(&mut input, autocomplete)?;
            }
            _ => {
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

fn handle_tab_completion(input: &mut String, autocomplete: &AutoCompletion) -> Result<(), anyhow::Error> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if let Some(last_word) = words.last() {
        let last_word = last_word.to_string();
        let matches = autocomplete.complete(&last_word);
        process_completion_matches(input, &last_word, matches, autocomplete)?;
    }
    Ok(())
}

fn process_completion_matches(
    input: &mut String,
    last_word: &str,
    matches: Vec<String>,
    autocomplete: &AutoCompletion,
) -> Result<(), anyhow::Error> {
    match matches.len() {
        0 => {},
        1 => handle_single_completion(input, last_word, &matches[0])?,
        _ => handle_multiple_completions(input, last_word, matches, autocomplete)?,
    }
    Ok(())
}

fn handle_single_completion(input: &mut String, last_word: &str, completion: &str) -> Result<(), anyhow::Error> {
    if completion.len() > last_word.len() {
        let to_add = &completion[last_word.len()..];
        print_and_flush(to_add)?;
        input.push_str(to_add);
    }
    Ok(())
}

fn handle_multiple_completions(
    input: &mut String,
    last_word: &str,
    matches: Vec<String>,
    autocomplete: &AutoCompletion,
) -> Result<(), anyhow::Error> {
    if let Some(common_prefix) = autocomplete.find_common_prefix(last_word) {
        if common_prefix.len() > last_word.len() {
            let to_add = &common_prefix[last_word.len()..];
            print_and_flush(to_add)?;
            input.push_str(to_add);
        } else {
            display_matches_and_reprompt(input, &matches)?;
        }
    } else {
        display_matches_and_reprompt(input, &matches)?;
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

        // This would normally print to stdout, but we'll test the logic
        let result = handle_single_completion(&mut input, "ec", completion);
        assert!(result.is_ok());
        assert_eq!(input, "echo");
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

        let result = process_completion_matches(&mut input, "xyz", matches, &autocomplete);
        assert!(result.is_ok());
        assert_eq!(input, "xyz"); // Should remain unchanged
    }

    #[test]
    fn test_process_completion_matches_single_match() {
        let mut input = String::from("ec");
        let autocomplete = create_test_autocomplete();
        let matches = vec!["echo".to_string()];

        let result = process_completion_matches(&mut input, "ec", matches, &autocomplete);
        assert!(result.is_ok());
        assert_eq!(input, "echo");
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

        let result = handle_tab_completion(&mut input, &autocomplete);
        assert!(result.is_ok());
        assert_eq!(input, ""); // Should remain empty when no words to complete
    }

    #[test]
    fn test_handle_tab_completion_with_partial_word() {
        let mut input = String::from("ec");
        let autocomplete = create_test_autocomplete();

        let result = handle_tab_completion(&mut input, &autocomplete);
        assert!(result.is_ok());
        assert_eq!(input, "echo"); // Should complete to "echo"
    }

    #[test]
    fn test_handle_tab_completion_with_multiple_words() {
        let mut input = String::from("echo hello ec");
        let autocomplete = create_test_autocomplete();

        let result = handle_tab_completion(&mut input, &autocomplete);
        assert!(result.is_ok());
        assert_eq!(input, "echo hello echo"); // Should complete the last word
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
    }

    #[test]
    fn test_multiple_completions_with_common_prefix() {
        let mut input = String::from("e");
        let autocomplete = create_test_autocomplete();
        let matches = vec!["echo".to_string(), "exit".to_string(), "export".to_string()];

        let result = handle_multiple_completions(&mut input, "e", matches, &autocomplete);
        assert!(result.is_ok());
        assert_eq!(input, "e"); // Should remain "e" since that's the only common prefix
    }

    #[test]
    fn test_multiple_completions_extending_prefix() {
        let mut input = String::from("ex");
        let autocomplete = create_test_autocomplete();
        let matches = vec!["exit".to_string(), "export".to_string()];

        let result = handle_multiple_completions(&mut input, "ex", matches, &autocomplete);
        assert!(result.is_ok());
        // Should extend to common prefix "ex" (no further extension possible)
        assert_eq!(input, "ex");
    }
}