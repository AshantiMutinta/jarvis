use std::io;
use std::str;
use Communication::Channel;
use Device::Command::{command_execution_error, parse_command, CommandExecution, CommandListen};

#[derive(Clone)]
pub struct NetworkInput {}

impl NetworkInput {
    pub fn new() -> NetworkInput {
        NetworkInput {}
    }
}

impl CommandListen for NetworkInput {
    fn listen(
        &self,
        com_channel: &Channel::Channel,
    ) -> Result<Box<dyn CommandExecution>, command_execution_error> {
        let mut buffer = [0; 256];
        match com_channel.read_udp_socket.recv_from(&mut buffer) {
            Ok(_) => match str::from_utf8(&buffer) {
                Ok(str_command) => parse_command(&String::from(str_command)),
                Err(_) => Err(command_execution_error::invalid_command),
            },
            Err(_) => Err(command_execution_error::invalid_command),
        }
    }
}
