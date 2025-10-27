use std::io::{self, Write};
use crate::history::History;

pub(crate) fn generate_output(_: &[&str], history: &History) -> Result<Vec<u8>, anyhow::Error> {
    Ok(history.show().into_bytes())
}

pub(crate) fn run(args: &[&str], history: &History) -> Result<(), anyhow::Error> {
    let output = generate_output(args, history)?;
    print!("{}", String::from_utf8_lossy(&output));
    io::stdout().flush()?;
    Ok(())
}