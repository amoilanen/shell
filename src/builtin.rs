pub(crate) mod exit;
pub(crate) mod echo;
pub(crate) mod type_;

pub(crate) trait BuiltinCommand {
    fn command(&self, command_and_args: &Vec<&str>) -> ();
    fn name(&self) -> String;
}