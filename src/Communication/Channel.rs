use std::net::UdpSocket;
use std::time::Duration;

#[derive(Debug)]
pub enum socket_setup_error {
    could_not_set_read_timeout,
    could_not_set_write_timeout,
    could_not_set_broadcast,
}

pub struct Channel {
    pub read_udp_socket: UdpSocket,
    pub write_udp_socket: UdpSocket,
}

impl Channel {
    pub fn new(read_IP: &str, write_IP: &str) -> Channel {
        let com_channel = Channel {
            read_udp_socket: UdpSocket::bind("0.0.0.0:61000")
                .expect("COULD NOT BIND TO UDP PACKET"),
            write_udp_socket: UdpSocket::bind("0.0.0.0:62345").expect("COULD NOT BIND SEND SOCKET"),
        };

        set_up_socket(&com_channel.read_udp_socket).expect("could not set up read socket");
        set_up_socket(&com_channel.write_udp_socket).expect("could not set up send socket");
        com_channel
    }
}

enum com_error {
    read_error,
    write_error,
}

pub fn set_up_socket(udp_socket: &UdpSocket) -> Result<(), socket_setup_error> {
    match udp_socket.set_write_timeout(Some(Duration::new(30, 0))) {
        Ok(_) => match udp_socket.set_read_timeout(Some(Duration::new(30, 0))) {
            Ok(_) => match udp_socket.set_broadcast(true) {
                Ok(_) => Ok(()),
                Err(_) => Err(socket_setup_error::could_not_set_broadcast),
            },
            Err(_) => Err(socket_setup_error::could_not_set_write_timeout),
        },
        Err(_) => Err(socket_setup_error::could_not_set_write_timeout),
    }
}
