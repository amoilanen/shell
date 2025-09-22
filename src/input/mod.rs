use std::io::{self, Write, Read};
use crate::input::autocompletion::AutoCompletion;

pub mod autocompletion;

pub fn read_line_with_completion(autocomplete: &AutoCompletion) -> Result<String, anyhow::Error> {
    let mut input = String::new();
    let mut stdin = io::stdin();

    loop {
        let mut buffer = [0; 1];
        stdin.read_exact(&mut buffer)?;
        let ch = buffer[0] as char;

        match ch {
            '\n' => {
                println!();
                break;
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
                                println!();
                                print!("$ {}", input);
                                io::stdout().flush()?;
                            }
                        } else {
                            // No common prefix found - show all matches
                            println!();
                            for match_str in matches {
                                print!("{}  ", match_str);
                            }
                            println!();
                            print!("$ {}", input);
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

    Ok(input)
}