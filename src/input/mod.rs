use std::io::{self, Write, Read};
use termios::{Termios, tcsetattr, TCSANOW, ECHO, ICANON, IEXTEN, ISIG, IXON, ICRNL, OPOST, VMIN, VTIME};
use crate::input::autocompletion::AutoCompletion;

pub mod autocompletion;

pub fn read_line_with_completion(autocomplete: &AutoCompletion) -> Result<String, anyhow::Error> {
    // Enable raw mode so we can read a byte at a time (including Tab)
    let raw_mode = RawMode::enable()?;
    let mut input = String::new();
    let mut stdin = io::stdin();

    loop {
        let mut buffer = [0; 1];
        stdin.read_exact(&mut buffer)?;
        let ch = buffer[0] as char;

        match ch {
            '\n' | '\r' => {
                println!();
                // Explicitly drop raw mode before returning to ensure terminal is restored
                drop(raw_mode);
                return Ok(input);
            }
            // Backspace/Delete
            '\u{7f}' | '\u{0008}' => {
                if !input.is_empty() {
                    input.pop();
                    // Move cursor back, erase char, move back again
                    print!("\x08 \x08");
                    io::stdout().flush()?;
                }
            }
            '\t' => {
                let words: Vec<&str> = input.split_whitespace().collect();
                if let Some(last_word) = words.last() {
                    let matches = autocomplete.complete(last_word);

                    if matches.is_empty() {
                        // No completions available
                    } else if matches.len() == 1 {
                        // Single match - complete it fully
                        let completion = &matches[0];
                        if completion.len() > last_word.len() {
                            let to_add = &completion[last_word.len()..];
                            print!("{}", to_add);
                            io::stdout().flush()?;
                            input.push_str(to_add);
                        }
                    } else {
                        // Multiple matches - try to complete common prefix first
                        if let Some(common_prefix) = autocomplete.find_common_prefix(last_word) {
                            if common_prefix.len() > last_word.len() {
                                // We can extend with common prefix
                                let to_add = &common_prefix[last_word.len()..];
                                print!("{}", to_add);
                                io::stdout().flush()?;
                                input.push_str(to_add);
                            } else {
                                // No more common prefix - show all matches
                                println!();
                                for match_str in matches {
                                    print!("{}  ", match_str);
                                }
                                print!("\n\r$ {}", input);
                                io::stdout().flush()?;
                            }
                        } else {
                            // No common prefix found - show all matches
                            println!();
                            for match_str in matches {
                                print!("{}  ", match_str);
                            }
                            println!();
                            print!("\r$ {}", input);
                            io::stdout().flush()?;
                        }
                    }
                }
            }
            _ => {
                print!("{}", ch);
                io::stdout().flush()?;
                input.push(ch);
            }
        }
    }
}

struct RawMode {
    original: Termios,
}

impl RawMode {
    fn enable() -> Result<Self, anyhow::Error> {
        let fd = 0; // stdin
        let original = Termios::from_fd(fd)?;
        let mut raw = original.clone();

        // Disable canonical mode, echo, signals, and special chars
        raw.c_lflag &= !(ICANON | ECHO | IEXTEN | ISIG);
        // Disable output processing
        raw.c_oflag &= !OPOST;
        // Disable CR-to-NL translation and XON/XOFF
        raw.c_iflag &= !(ICRNL | IXON);
        // Ensure reads return as soon as 1 byte is available
        raw.c_cc[VMIN] = 1;
        raw.c_cc[VTIME] = 0;

        tcsetattr(fd, TCSANOW, &raw)?;
        Ok(Self { original })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let fd = 0; // stdin
        let _ = tcsetattr(fd, TCSANOW, &self.original);
    }
}