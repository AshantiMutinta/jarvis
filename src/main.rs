extern crate Jarvis;
extern crate termcolor;

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use Jarvis::Communication::Channel;
use Jarvis::Device::Command;
use Jarvis::Device::Command::{CommandExecution, CommandListen};
use Jarvis::Device::Device;

enum message_level {
    info,
    warning,
    error,
    success,
}

fn post_message(message: &str, level: message_level) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let log_message = match level {
        message_level::warning => ("Warning", Color::Yellow),
        message_level::error => ("Error", Color::Red),
        message_level::success => ("OK", Color::Green),
        _ => ("Info", Color::White),
    };
    stdout.set_color(ColorSpec::new().set_fg(Some(log_message.1)));
    writeln!(&mut stdout, "{}", [log_message.0, ":", message].join(""));
}
fn listen_to_commands(com_channel: Channel::Channel) {
    post_message("ENTER OR VOICE COMMAND", message_level::info);
    std::io::stdout().flush();
    let thread_data = Arc::new(Mutex::new(com_channel));
    let result = thread::spawn(move || -> () {
        loop {
            let thread_com_channel = &*thread_data.lock().unwrap();
            let mut text_io = Command::TextInput::new("");
            let exec = text_io.listen(&*thread_com_channel);
            match exec
            {
                Ok(command) =>
                {
                    command.execute(&*thread_com_channel);
                }
                Err(_)=>
                {
                    post_message("COULD NOT EXECUTE COMMAND",message_level::error);
                }
            }
        }
    });

    result.join();
}

fn main() {
    post_message("Starting JARVIS", message_level::info);
    post_message("Checking for devices", message_level::info);
    println!("=====================");
    let com_channel = Channel::Channel::new("0.0.0.0:61000", "0.0.0.0:62345");
    match Device::set_up_devices(&com_channel) {
        Ok(devices) => {
            println!("Set up devices : devices {:?}", devices);
            listen_to_commands(com_channel);
        }
        Err(err) => {
            post_message("JARVIS COULD NOT START", message_level::error);
            listen_to_commands(com_channel);
        }
    };
    println!("=====================");
}
