use std::env;

pub(crate) fn run(_: &[&str]) -> Result<(), anyhow::Error> {
    let current_directory = env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
    let path_str = current_directory.to_str()
        .ok_or_else(|| anyhow::anyhow!("Path contains invalid Unicode"))?;
    println!("\r{}", path_str);
    Ok(())
}