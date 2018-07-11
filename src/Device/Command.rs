use std::io;
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
    mut tokenized_command: impl Iterator<Item = &'a str>,
) -> Result<impl CommandExecution<'a>, command_execution_error> {
    let first_token = tokenized_command.next();
    match tokenized_command.next() {
        Some(device) => Ok(StatusExec {
            device_name: device,
        }),
        None => Err(command_execution_error::device_name_not_provided),
    }
}

pub fn parse_command<'a>(
    command: &'a String,
) -> Result<impl CommandExecution + 'a, command_execution_error> {
    let mut tokenized_command = command.split_whitespace();
    match tokenized_command.next() {
        Some(first_command) => {
            let parent_command = find_parent_command(first_command);
            match parent_command {
                parent_command::status => evaluate_status_command(tokenized_command),
                _ => unimplemented!("have not implemented default case"),
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

trait CommandListen<'a> {
    fn listen(&self, com_channel: &'a Channel::Channel);
}

struct TextInput {}
struct VoiceInput {}

impl<'a> CommandListen<'a> for TextInput {
    fn listen(&self, com_channel: &'a Channel::Channel) {
        let mut command: String = String::new();
        loop {
            command = String::new();
            io::stdin().read_line(&mut command);
            match parse_command(&command) {
                Ok(exec) => {
                    exec.execute(&com_channel);
                }
                Err(_) => {
                    ();
                }
            }
        }
    }
}

#[test]
fn test_parent_status_match() {
    assert_eq!(find_parent_command("sTatus"), parent_command::status);
    assert_eq!(find_parent_command("eXit"), parent_command::exit);
    assert_eq!(find_parent_command("hELp"), parent_command::help);
}
