pub mod builtin;
pub mod exec;

pub(crate) trait ShellCommand {
    fn run(&self, command_and_args: &Vec<&str>) -> ();
    fn name(&self) -> String;
}