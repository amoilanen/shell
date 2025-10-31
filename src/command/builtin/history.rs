use std::io::{self, Write};
use crate::history::History;

pub(crate) fn generate_output(args: &[&str], history: &History) -> Result<Vec<u8>, anyhow::Error> {
    let limit = args.get(0)
        .map(|s| s.trim().parse::<usize>())
        .transpose()?;
    Ok(history.show(limit).into_bytes())
}

pub(crate) fn run(args: &[&str], history: &History) -> Result<(), anyhow::Error> {
    let output = generate_output(args, history)?;
    print!("{}", String::from_utf8_lossy(&output));
    io::stdout().flush()?;
    Ok(())
}
