pub(crate) fn read_option(option_short_name: &str, args: &[&str]) -> Option<String> {
    let mut result = None;
    if let Some(option_position) = args.iter().position(|&arg| arg == format!("-{}", option_short_name)) {
        if let Some(option_value) = args.get(option_position + 1) {
            result = Some(option_value.to_string())
        }
    }
    result
}

#[cfg(test)]
mod tests {

    use super::read_option;

    #[test]
    fn test_read_option_no_option() {
        let result = read_option("w", &vec!["a", "b", "c"]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_option_no_value() {
        let result = read_option("w", &vec!["a", "-w"]);
        assert_eq!(result, None);
    }

        #[test]
    fn test_read_option() {
        let result = read_option("w", &vec!["-w", "w_value"]);
        assert_eq!(result, Some("w_value".to_owned()));
    }
}