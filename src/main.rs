extern crate Jarvis;
extern crate termcolor;

use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use Jarvis::Communication::Channel;
use Jarvis::Device::Command;
use Jarvis::Device::Command::CommandExecution;
use Jarvis::Device::Device;

enum message_level {
    info,
    warning,
    error,
    success,
}

fn post_message(message: &str, level: message_level) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let term_color = match level {
        message_level::warning => Color::Yellow,
        message_level::error => Color::Red,
        message_level::success => Color::Green,
        _ => Color::White,
    };
    stdout.set_color(ColorSpec::new().set_fg(Some(term_color)));
    writeln!(&mut stdout, "{}", message);
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
                exec.execute(&com_channel);
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
