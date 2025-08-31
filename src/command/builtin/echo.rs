use std::fs;

pub(crate) fn run(args: &[&str], stdout_redirect_filename: Option<&str>) -> () {
    let to_output = args.join(" ");
    if let Some(stdout_redirect_filename) = stdout_redirect_filename {
        fs::write(stdout_redirect_filename, to_output).unwrap();
    } else {
        print!("{}\n", to_output);
    }
}