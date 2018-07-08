extern crate Jarvis;
extern crate colored;

use colored::*;
use std::io::Write;
use Jarvis::Communication::Channel;
use Jarvis::Device::Command;
use Jarvis::Device::Device;

enum message_level {
    info,
    warning,
    error,
    success,
}

fn post_message(message: &str, level: message_level) {
    match level {
        message_level::warning => {
            println!("{}", message.yellow());
        }
        message_level::error => {
            println!("{}", message.red());
        }
        message_level::success => println!("{}", message.green()),
        _ => {
            println!("{}", message);
        }
    }
}
fn listen_to_commands<'a>(com_channel: &'a Channel::Channel) {
    post_message("Enter COMMAND", message_level::info);
    std::io::stdout().flush();
    let mut command: String = String::new();
    loop {
        command = String::new();
        std::io::stdin().read_line(&mut command);
        match Command::parse_command(&command) {
            Ok(exec) => {
                //exec.execute();
            }
            Err(_) => {
                post_message("invalid command", message_level::error);
            }
        }
    }
}

fn main() {
    post_message("Starting JARVIS", message_level::success);
    println!("Checking For Devices");
    let mut com_channel = Channel::Channel::new("0.0.0.0:61000", "0.0.0.0:62345");
    match Device::set_up_devices(&com_channel) {
        Ok(devices) => {
            println!("Set up devices : devices {:?}", devices);
            listen_to_commands(&com_channel);
        }
        Err(err) => {
            post_message("JARVIS COULD NOT START", message_level::error);
        }
    };
}
