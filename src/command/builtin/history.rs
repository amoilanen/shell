use std::io::{self, Write};

pub(crate) fn generate_output(_: &[&str]) -> Result<Vec<u8>, anyhow::Error> {
    //TODO: Implement
    Ok(Vec::new())
}

pub(crate) fn run(args: &[&str]) -> Result<(), anyhow::Error> {
    let output = generate_output(args)?;
    print!("{}", String::from_utf8_lossy(&output));
    io::stdout().flush()?;
    Ok(())
}