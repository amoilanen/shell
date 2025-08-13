use std::process;

pub(crate) fn run(args: &[&str]) -> () {
    let mut exit_code = 0;
    if let Some(args_input) =  args.get(0) {
        let args: Vec<&str> = args_input.split_whitespace().collect();
        if let Some(exit_code_arg) = args.get(0) {
            if let Some(code) = exit_code_arg.parse().ok() {
                exit_code = code;
            }
        }
    }
    if exit_code >= 0 {
        process::exit(exit_code);
    }
}