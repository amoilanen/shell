use std::env;

pub(crate) fn generate_output() -> Result<Vec<u8>, anyhow::Error> {
    let current_directory = env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
    let path_str = current_directory.to_str()
        .ok_or_else(|| anyhow::anyhow!("Path contains invalid Unicode"))?;
    Ok(format!("{}\n", path_str).into_bytes())
}

pub(crate) fn run(_: &[&str]) -> Result<(), anyhow::Error> {
    let output = generate_output()?;
    print!("\r{}", String::from_utf8_lossy(&output));
    Ok(())
}