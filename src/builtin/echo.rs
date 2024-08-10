pub(crate) fn command(command_and_args: &Vec<&str>) {
    if let Some(args_input) = command_and_args.get(1) {
        print!("{}", args_input);
    }
}