use std::env;

pub(crate) fn run(_: &[&str]) -> () {
    let current_directory = env::current_dir().unwrap();
    println!("{}", current_directory.to_str().unwrap());
}