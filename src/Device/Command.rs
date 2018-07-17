use std::io;
use std::str::SplitWhitespace;

use Communication::Channel;

#[derive(Debug, PartialEq)]
enum parent_command {
    status,
    help,
    exit,
    no_parent_command,
}

pub enum command_execution_error {
    empty_command,
    device_name_not_provided,
    invalid_command,
}

pub trait CommandExecution<'a> {
    fn execute(&self, &'a Channel::Channel) -> Result<(), command_execution_error>;
}

struct StatusExec<'a> {
    device_name: &'a str,
}

impl<'a> CommandExecution<'a> for StatusExec<'a> {
    fn execute(&self, com_channel: &'a Channel::Channel) -> Result<(), command_execution_error> {
        unimplemented!("Have not implemented Status");
    }
}

struct stub {}

impl<'a> CommandExecution<'a> for stub {
    fn execute(&self, com_channel: &'a Channel::Channel) -> Result<(), command_execution_error> {
        println!("Stub");
        Ok(())
    }
}

fn evaluate_status_command<'a>(
    mut tokenized_command: SplitWhitespace<'a>,
) -> Result<Box<dyn CommandExecution<'a> + 'a>, command_execution_error> {
    let first_token = tokenized_command.next();
    match tokenized_command.next() {
        Some(device) => {
            let comm = StatusExec {
                device_name: device,
            };

            Ok(Box::new(comm))
        }
        None => Err(command_execution_error::device_name_not_provided),
    }
}

pub fn parse_command<'a>(
    command: &'a String,
) -> Result<Box<CommandExecution<'a> + 'a>, command_execution_error> {
    let mut tokenized_command = command.split_whitespace();
    match tokenized_command.next() {
        Some(first_command) => {
            let parent_command = find_parent_command(first_command);
            match parent_command {
                parent_command::status => evaluate_status_command(tokenized_command),
                _ => Err(command_execution_error::invalid_command),
            }
        }
        None => Err(command_execution_error::empty_command),
    }
}

fn find_parent_command(command: &str) -> parent_command {
    match &*command.to_uppercase() {
        "STATUS" => parent_command::status,
        "HELP" => parent_command::help,
        "EXIT" => parent_command::exit,
        _ => parent_command::no_parent_command,
    }
}
struct CommandExecutionWrapper<'a, T: 'a>
where
    T: CommandExecution<'a>,
{
    exec: &'a T,
}
pub trait CommandListen<'a> {
    fn listen(
        &'a mut self,
        com_channel: &'a Channel::Channel,
    ) -> Result<Box<dyn CommandExecution<'a> + 'a>, command_execution_error>;
}

pub struct TextInput {
    command: String,
}
impl TextInput {
    pub fn new(com: &str) -> TextInput {
        TextInput {
            command: String::from(com),
        }
    }
}
struct VoiceInput {}

impl<'a> CommandListen<'a> for TextInput {
    fn listen(
        &'a mut self,
        com_channel: &'a Channel::Channel,
    ) -> Result<Box<dyn CommandExecution<'a> + 'a>, command_execution_error> {
        self.command = String::new();
        io::stdin().read_line(&mut self.command);
        parse_command(&self.command)
    }
}

#[test]
fn test_parent_status_match() {
    assert_eq!(find_parent_command("sTatus"), parent_command::status);
    assert_eq!(find_parent_command("eXit"), parent_command::exit);
    assert_eq!(find_parent_command("hELp"), parent_command::help);
}
