use std::process;

pub(crate) fn run(args: &[&str]) -> Result<(), anyhow::Error> {
    let exit_code = parse_exit_code(args);
    if exit_code >= 0 {
        process::exit(exit_code);
    }
    Ok(())
}

fn parse_exit_code(args: &[&str]) -> i32 {
    let mut exit_code = 0;
    let joined = args.join(" ");
    let args: Vec<&str> = joined.split_whitespace().collect();
    if let Some(exit_code_arg) = args.get(0) {
        if let Some(code) = exit_code_arg.parse().ok() {
            exit_code = code;
        }
    }
    exit_code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exit_code_no_args() {
        let result = parse_exit_code(&[]);
        assert_eq!(result, 0, "Should default to exit code 0 when no arguments provided");
    }

    #[test] 
    fn test_parse_exit_code_empty_string() {
        let result = parse_exit_code(&[""]);
        assert_eq!(result, 0, "Should default to exit code 0 for empty string argument");
    }

    #[test]
    fn test_parse_exit_code_valid_zero() {
        let result = parse_exit_code(&["0"]);
        assert_eq!(result, 0, "Should parse '0' as exit code 0");
    }

    #[test]
    fn test_parse_exit_code_valid_positive() {
        let result = parse_exit_code(&["42"]);
        assert_eq!(result, 42, "Should parse '42' as exit code 42");
    }

    #[test]
    fn test_parse_exit_code_valid_negative() {
        let result = parse_exit_code(&["-1"]);
        assert_eq!(result, -1, "Should parse '-1' as exit code -1");
    }

    #[test]
    fn test_parse_exit_code_invalid_string() {
        let result = parse_exit_code(&["abc"]);
        assert_eq!(result, 0, "Should default to exit code 0 for non-numeric string");
    }

    #[test]
    fn test_parse_exit_code_invalid_mixed() {
        let result = parse_exit_code(&["123abc"]);
        assert_eq!(result, 0, "Should default to exit code 0 for mixed alphanumeric string");
    }

    #[test]
    fn test_parse_exit_code_whitespace_single_number() {
        let result = parse_exit_code(&["  42  "]);
        assert_eq!(result, 42, "Should parse whitespace-padded number correctly");
    }

    #[test]
    fn test_parse_exit_code_multiple_numbers_in_string() {
        let result = parse_exit_code(&["42 123"]);
        assert_eq!(result, 42, "Should parse first number when multiple numbers provided");
    }

    #[test]
    fn test_parse_exit_code_very_large_number() {
        let result = parse_exit_code(&["2147483647"]);
        assert_eq!(result, 2147483647, "Should handle maximum i32 value");
    }

    #[test]
    fn test_parse_exit_code_overflow_number() {
        let result = parse_exit_code(&["9999999999999999999"]);
        assert_eq!(result, 0, "Should default to 0 for numbers that overflow i32");
    }

    #[test]
    fn test_parse_exit_code_floating_point() {
        let result = parse_exit_code(&["3.14"]);
        assert_eq!(result, 0, "Should default to 0 for floating point numbers");
    }

    #[test]
    fn test_parse_exit_code_multiple_args() {
        let result = parse_exit_code(&["42", "ignored"]);
        assert_eq!(result, 42, "Should only consider first argument");
    }

    #[test]
    fn test_parse_exit_code_whitespace_only() {
        let result = parse_exit_code(&["   "]);
        assert_eq!(result, 0, "Should default to 0 for whitespace-only input");
    }
}