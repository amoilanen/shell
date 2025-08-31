use std::fs;

pub(crate) fn run(args: &[&str], stdout_redirect_filename: Option<&str>) -> () {
    let to_output = format!("{}\n", args.join(" "));
    if let Some(stdout_redirect_filename) = stdout_redirect_filename {
        fs::write(stdout_redirect_filename, to_output).unwrap();
    } else {
        print!("{}", to_output);
    }
}