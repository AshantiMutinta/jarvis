use std::net::UdpSocket;
use std::time::Duration;

#[derive(Debug)]
pub enum socket_setup_error {
    could_not_set_read_timeout,
    could_not_set_write_timeout,
    could_not_set_broadcast,
}

pub struct Channel<'a> {
    pub read_udp_socket: &'a UdpSocket,
    pub write_udp_socket: &'a UdpSocket,
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
