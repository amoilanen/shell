use crate::command::ShellCommand;

pub(crate) struct Echo {}

impl ShellCommand for Echo {
    fn run(&self, command_and_args: &Vec<&str>) -> () {
        if let Some(args_input) = command_and_args.get(1) {
            print!("{}", args_input);
        }
    }
    fn name(&self) -> String {
        "echo".to_string()
    }
}