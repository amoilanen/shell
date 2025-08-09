pub(crate) fn run(command_and_args: &[&str]) -> () {
    if let Some(args_input) = command_and_args.get(1) {
        print!("{}", args_input);
    }
}