use std::io;
use Communication::Channel;
use Device::Command::{command_execution_error, parse_command, CommandExecution, CommandListen};

#[derive(Clone)]
pub struct TextInput {}

impl TextInput {
    pub fn new(com: &str) -> TextInput {
        TextInput {}
    }
}

impl CommandListen for TextInput {
    fn listen(
        &self,
        com_channel: &Channel::Channel,
    ) -> Result<Box<dyn CommandExecution>, command_execution_error> {
        let mut command = String::new();
        io::stdin().read_line(&mut command);
        parse_command(&command)
    }
}
