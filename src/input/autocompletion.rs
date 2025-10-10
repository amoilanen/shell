use std::collections::HashSet;

pub struct AutoCompletion {
    candidates: HashSet<String>,
    dynamic_completion: Box<dyn Fn(&str) -> Vec<String>>,
}

impl AutoCompletion {

    #[cfg(test)]
    pub fn new(candidates: Vec<&str>) -> Self {
        Self::new_with_dynamic_completion(candidates, Box::new(|_| Vec::new()))
    }

    pub fn new_with_dynamic_completion(
        candidates: Vec<&str>,
        dynamic_completion: Box<dyn Fn(&str) -> Vec<String>>,
    ) -> Self {
        Self {
            candidates: candidates.into_iter().map(|s| s.to_string()).collect(),
            dynamic_completion
        }
    }

    pub fn complete(&self, partial: &str) -> Vec<String> {
        if partial.is_empty() {
            return Vec::new();
        }

        let mut seen = HashSet::new();
        let mut matches: Vec<String> = self
            .candidates
            .iter()
            .filter(|candidate| candidate.starts_with(partial))
            .filter(|candidate| seen.insert((*candidate).clone()))
            .cloned()
            .collect();

        let dynamic_matches = (self.dynamic_completion)(partial);
        for m in dynamic_matches {
            if seen.insert(m.clone()) {
                matches.push(m);
            }
        }

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

    #[test]
    fn test_dynamic_completion_no_matches() {
        let autocomplete = AutoCompletion::new_with_dynamic_completion(
            vec!["echo", "exit"],
            Box::new(|partial: &str| -> Vec<String> {
                if partial == "xyz" {
                    vec![]
                } else {
                    vec!["dynamic1".to_string(), "dynamic2".to_string()]
                }
            })
        );

        assert_eq!(autocomplete.complete("xyz"), Vec::<String>::new());
    }

    #[test]
    fn test_dynamic_completion_with_matches() {
        let autocomplete = AutoCompletion::new_with_dynamic_completion(
            vec!["echo", "exit"],
            Box::new(|partial: &str| -> Vec<String> {
                if partial == "d" {
                    vec!["docker".to_string(), "dotnet".to_string()]
                } else {
                    vec![]
                }
            })
        );

        let result = autocomplete.complete("d");
        assert_eq!(result, vec!["docker", "dotnet"]);
    }

    #[test]
    fn test_dynamic_completion_combined_with_static() {
        let autocomplete = AutoCompletion::new_with_dynamic_completion(
            vec!["echo", "exit"],
            Box::new(|partial: &str| -> Vec<String> {
                if partial == "e" {
                    vec!["env".to_string(), "emacs".to_string()]
                } else {
                    vec![]
                }
            })
        );

        let result = autocomplete.complete("e");
        assert_eq!(result, vec!["echo", "emacs", "env", "exit"]);
    }

    #[test]
    fn test_dynamic_completion_deduplication() {
        let autocomplete = AutoCompletion::new_with_dynamic_completion(
            vec!["echo", "exit"],
            Box::new(|partial: &str| -> Vec<String> {
                if partial == "e" {
                    vec!["echo".to_string(), "env".to_string()]
                } else {
                    vec![]
                }
            })
        );

        let result = autocomplete.complete("e");
        assert_eq!(result, vec!["echo", "env", "exit"]);
    }

    #[test]
    fn test_find_common_prefix_with_dynamic_completion() {
        let autocomplete = AutoCompletion::new_with_dynamic_completion(
            vec!["echo", "exit", "export"],
            Box::new(|partial: &str| -> Vec<String> {
                if partial == "ex" {
                    vec!["expr".to_string()]
                } else {
                    vec![]
                }
            })
        );
        assert_eq!(autocomplete.find_common_prefix("ex"), Some("ex".to_string()));
    }

    #[test]
    fn test_dynamic_completion_empty_partial() {
        let autocomplete = AutoCompletion::new_with_dynamic_completion(
            vec!["echo"],
            Box::new(|_partial: &str| -> Vec<String> {
                vec!["should_not_appear".to_string()]
            })
        );
        assert_eq!(autocomplete.complete(""), Vec::<String>::new());
    }
}