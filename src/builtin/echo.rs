use super::BuiltinCommand;

pub(crate) struct Echo {}
impl BuiltinCommand for Echo {
    fn command(&self, command_and_args: &Vec<&str>) -> () {
        if let Some(args_input) = command_and_args.get(1) {
            print!("{}", args_input);
        }
    }
    fn name(&self) -> String {
        "echo".to_string()
    }
}