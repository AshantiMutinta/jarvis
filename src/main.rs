#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate Jarvis;
extern crate num_cpus;
extern crate termcolor;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Write};
use std::sync::{
    mpsc, mpsc::{Receiver, Sender}, Arc, Mutex,
};
use std::thread;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use Jarvis::Communication::Channel;
use Jarvis::Device::Command::{CommandExecution, CommandListen};
use Jarvis::Device::Device;
use Jarvis::Device::{Command, NetworkCommand, TextCommand};

enum MessageLevel {
    info,
    warning,
    error,
    success,
}

fn post_message(message: &str, level: MessageLevel) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let log_message = match level {
        MessageLevel::warning => ("Warning", Color::Yellow),
        MessageLevel::error => ("Error", Color::Red),
        MessageLevel::success => ("OK", Color::Green),
        _ => ("Info", Color::White),
    };
    stdout.set_color(ColorSpec::new().set_fg(Some(log_message.1)));
    writeln!(&mut stdout, "{}", [log_message.0, ":", message].join(""));
}
fn listen_to_commands(com_channel: Channel::TransportLayerChannel) {
    post_message("ENTER OR VOICE COMMAND", MessageLevel::info);
    std::io::stdout().flush();
    let channel = mpsc::channel();
    let thread_data = Arc::new(com_channel);
    let th = thread_data.clone();
    let send_clone_channel = channel.0.clone();
    let results = COMMAND_LISTENERS
        .iter()
        .map(move |execution| {
            let th = th.clone();
            let send_clone_channel = send_clone_channel.clone();
            Some(thread::spawn(move || -> () {
                loop {
                    let exec = execution.listen(&th);
                    match exec {
                        Ok(command) => match send_clone_channel.send(command) {
                            Ok(com) => {}
                            Err(_) => {
                                post_message("CHANNEL COULD NOT BE SEND", MessageLevel::error);
                            }
                        },
                        Err(Command::command_execution_error::timeout) => {
                            ();
                        }
                        Err(_) => {
                            post_message("COULD NOT EXECUTE COMMAND", MessageLevel::error);
                        }
                    }
                }
            }))
        })
        .collect::<Vec<_>>();

    let tcount = thread_data.clone();
    let loop_result = thread::spawn(move || -> () {
        let thr = tcount.clone();
        loop {
            match channel.1.recv() {
                Ok(exec) => {
                    post_message("EXECUTING COMMAND", MessageLevel::info);
                    exec.execute(&thr);
                }
                Err(_) => {
                    post_message("COULD NOT RECIEVE COMMAND", MessageLevel::error);
                }
            }
        }
    });

    results
        .into_iter()
        .map(|f| match f {
            Some(res) => {
                res.join();
            }
            _ => (),
        })
        .collect::<Vec<_>>();
    loop_result.join();
}

fn read_from_file(file_name: &str) -> Result<String, Error> {
    match File::open(file_name) {
        Ok(mut opened_file) => {
            let mut contents = String::new();
            match opened_file.read_to_string(&mut contents) {
                Ok(bytes_read) => Ok(contents),
                Err(_) => Err(Error::new(
                    ErrorKind::InvalidData,
                    "Could not read from file",
                )),
            }
        }
        Err(_) => Err(Error::new(ErrorKind::InvalidInput, "Could not find file")),
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TCPConfiguration {
    readIO: u32,
    writeIO: u32,
    timeout: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct DataLinkConfiguration {
    NetworkCards: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Configuration {
    TCP: TCPConfiguration,
    DataLink: DataLinkConfiguration,
}

fn get_configuration(conf: &String) -> Result<Configuration, Error> {
    match serde_json::from_str::<Configuration>(&conf) {
        Ok(config) => Ok(config),
        Err(_) => Err(Error::new(
            ErrorKind::InvalidData,
            "COULD NOT DESERIALIZE CONFIGURATION",
        )),
    }
}
fn main() {
    post_message("Starting JARVIS", MessageLevel::info);
    post_message("Checking for devices", MessageLevel::info);
    println!("=====================");
    match env::args().next() {
        Some(file_name) => match get_configuration(&file_name) {
            Ok(config) => {
                let mut com_channel = Channel::TransportLayerChannel::new(
                    &vec!["0.0.0.0", &(*config.TCP.readIO.to_string())].join(""),
                    &vec!["0.0.0.0", &(*config.TCP.writeIO.to_string())].join(""),
                );
                match Device::set_up_devices(&mut com_channel) {
                    Ok(devices) => {
                        println!("Set up devices : devices {:?}", devices);
                    }
                    Err(err) => {
                        post_message("JARVIS COULD NOT SET UP DEVICES", MessageLevel::error);
                    }
                };

                listen_to_commands(com_channel);
                println!("=====================");
            }
            Err(_) => {
                post_message("INVALID DATA IN CONFIGURATION FILE", MessageLevel::error);
            }
        },
        None => {
            post_message("COULD NOT FIND CONFIGURATION FILE", MessageLevel::error);
        }
    }
}

#[test]
fn test_configuration() {
    let test_configuration = String::from("{\"TCP\":{\"readIO\":12345,\"writeIO\":444,\"timeout\":30},\"DataLink\":{\"NetworkCards\":[\"Example1\"]}}");
    let config =
        get_configuration(&test_configuration).expect("test failed, was supposed to serialize");
    assert_eq!(config.TCP.readIO, 12345);
    assert_eq!(config.TCP.writeIO, 444);
    assert_eq!(config.TCP.timeout, 30);
    assert_eq!(config.DataLink.NetworkCards[0], "Example1");
}

lazy_static! {
    static ref COMMAND_LISTENERS: Vec<Box<CommandListen>> = {
        vec![
            (Box::new(TextCommand::TextInput::new(""))),
            (Box::new(NetworkCommand::NetworkInput::new())),
        ]
    };
}
