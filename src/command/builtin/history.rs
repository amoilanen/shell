use std::io::{self, Write};
use crate::history::History;

pub(crate) fn generate_output(args: &[&str], history: &History) -> Result<Vec<u8>, anyhow::Error> {
    let limit = args.get(0)
        .map(|s| s.trim().parse::<usize>())
        .transpose()?;
    Ok(history.show(limit).into_bytes())
}

pub(crate) fn run(args: &[&str], history: &mut History) -> Result<(), anyhow::Error> {
    if let Some(r_option_position) = args.iter().position(|arg| arg.to_string() == "-r") {
        if let Some(history_file_path) = args.get(r_option_position + 1) {
            history.read_from_file(&history_file_path.into())
        } else {
            Ok(())
        }
    } else {
        let output = generate_output(args, history)?;
        print!("{}", String::from_utf8_lossy(&output));
        io::stdout().flush()?;
        Ok(())
    }
}
