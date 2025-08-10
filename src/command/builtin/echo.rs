pub(crate) fn run(command_and_args: &[&str]) -> () {
    let args_input = &command_and_args[1..];

    print!("{}", args_input.join(" "));
}