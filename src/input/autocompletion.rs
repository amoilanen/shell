use std::collections::HashSet;

pub struct AutoCompletion {
    candidates: HashSet<String>,
}

impl AutoCompletion {
    pub fn new(candidates: Vec<&str>) -> Self {
        Self {
            candidates: candidates.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn complete(&self, partial: &str) -> Vec<String> {
        if partial.is_empty() {
            return Vec::new();
        }

        let mut matches: Vec<String> = self
            .candidates
            .iter()
            .filter(|candidate| candidate.starts_with(partial))
            .cloned()
            .collect();

        matches.sort();
        matches
    }

    pub fn find_common_prefix(&self, partial: &str) -> Option<String> {
        let matches = self.complete(partial);
        let common_len = self.find_common_length(&matches);
        
        common_len.into_iter().flat_map(|len| {
            if len >= partial.len() {
                matches.first().map(|s| s[..len].to_string())
            } else {
                None
            }
        }).next()
    }

    fn find_common_length(&self, matches: &Vec<String>) -> Option<usize> {
        if matches.is_empty() {
            return None;
        }
        if matches.len() == 1 {
            return matches.first().map(|s| s.len());
        }
        let first = &matches[0];
        let mut common_len = first.len();
        for match_str in &matches[1..] {
            common_len = common_len.min(self.find_common_length_of_two_strings(first, match_str));
        }
        Some(common_len)
    }

    fn find_common_length_of_two_strings(&self, s1: &str, s2: &str) -> usize {
        let mut len = 0;
        for (c1, c2) in s1.chars().zip(s2.chars()) {
            if c1 == c2 {
                len += c1.len_utf8();
            } else {
                break;
            }
        }
        return len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_builtin_commands() {
        let autocomplete = AutoCompletion::new(vec!["echo", "cd", "pwd", "exit", "type"]);

        assert_eq!(autocomplete.complete("e"), vec!["echo", "exit"]);
        assert_eq!(autocomplete.complete("ec"), vec!["echo"]);
        assert_eq!(autocomplete.complete("p"), vec!["pwd"]);
        assert_eq!(autocomplete.complete("pw"), vec!["pwd"]);
        assert_eq!(autocomplete.complete("pwd"), vec!["pwd"]);
        assert_eq!(autocomplete.complete("xyz"), Vec::<String>::new());
    }

    #[test]
    fn test_find_common_prefix() {
        let autocomplete = AutoCompletion::new(vec!["echo", "exit", "export"]);

        assert_eq!(autocomplete.find_common_prefix("e"), Some("e".to_string()));
        assert_eq!(autocomplete.find_common_prefix("ec"), Some("echo".to_string()));
        assert_eq!(autocomplete.find_common_prefix("ex"), Some("ex".to_string()));
        assert_eq!(autocomplete.find_common_prefix("xyz"), None);
    }
}