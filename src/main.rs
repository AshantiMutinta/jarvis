#[macro_use]
extern crate lazy_static;
extern crate Jarvis;
extern crate num_cpus;
extern crate termcolor;

use std::io::Write;
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

enum execution_order {
    sync,
    async,
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
fn listen_to_commands(com_channel: Channel::Channel) {
    post_message("ENTER OR VOICE COMMAND", MessageLevel::info);
    std::io::stdout().flush();
    let io_execution: Vec<(Box<CommandListen>, execution_order)> = vec![
        (
            Box::new(TextCommand::TextInput::new("")),
            execution_order::async,
        ),
        (
            Box::new(NetworkCommand::NetworkInput::new()),
            execution_order::async,
        ),
    ];

    //use tuple of (io,execution_order) to determine execution of application
    let channel = mpsc::channel();
    let thread_data = Arc::new(com_channel);
    let th = thread_data.clone();
    let send_clone_channel = channel.0.clone();
    let results = COMMAND_LISTENERS
        .iter()
        .map(move |execution| {
            let th = th.clone();
            let send_clone_channel = send_clone_channel.clone();
            match execution.1 {
                execution_order::async => Some(thread::spawn(move || -> () {
                    loop {
                        let exec = execution.0.listen(&th);
                        match exec {
                            Ok(command) => match send_clone_channel.send(command) {
                                Ok(com) => {}
                                Err(_) => {
                                    post_message("CHANNEL COULD NOT BE SEND", MessageLevel::error);
                                }
                            },
                            Err(_) => {
                                post_message("COULD NOT EXECUTE COMMAND", MessageLevel::error);
                            }
                        }
                    }
                })),
                _ => None,
            }
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

fn main() {
    post_message("Starting JARVIS", MessageLevel::info);
    post_message("Checking for devices", MessageLevel::info);
    println!("=====================");
    let mut com_channel = Channel::Channel::new("0.0.0.0:61000", "0.0.0.0:62345");
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

lazy_static! {
    static ref COMMAND_LISTENERS: Vec<(Box<CommandListen>, execution_order)> = {
        vec![
            (
                Box::new(TextCommand::TextInput::new("")),
                execution_order::async,
            ),
            (
                Box::new(NetworkCommand::NetworkInput::new()),
                execution_order::async,
            ),
        ]
    };
}
