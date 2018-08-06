extern crate crc;

use std::io;
use std::io::prelude::*;
use std::net::UdpSocket;
use std::time::Duration;
use self::crc::{crc32, Hasher32};

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
            read_udp_socket: UdpSocket::bind(read_IP).expect("COULD NOT BIND TO UDP PACKET"),
            write_udp_socket: UdpSocket::bind(write_IP).expect("COULD NOT BIND SEND SOCKET"),
        };

        set_up_socket(&com_channel.read_udp_socket).expect("could not set up read socket");
        set_up_socket(&com_channel.write_udp_socket).expect("could not set up send socket");
        com_channel
    }
}



impl io::Read for Channel {
    fn read(&mut self, mut buffer: &mut [u8]) -> Result<usize,io::Error> {
        match self.read_udp_socket.recv_from(&mut buffer) 
        {
            Ok(read_result) => match buffer.first() {
                Some(first) => {
                    if (*first == 165u8) {
                        match validate_checksum(buffer) 
                        {
                            Ok(validated_buffer) => Ok(buffer.len()),
                            Err(_) =>
                            {
                                  Err(io::Error::new(io::ErrorKind::InvalidData,"MISMATCHED CHECKSUM"))
                            }
                        }
                    } 
                    else 
                    {
                        Err(io::Error::new(io::ErrorKind::InvalidData,"PROTOCOL ERROR"))
                    }
                }
                None => Err(io::Error::new(io::ErrorKind::NotFound,"EMPTY BUFFER")),
            },
            Err(_) => {

                Err(io::Error::new(io::ErrorKind::BrokenPipe,"COULD NOT READ BUFFER"))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum checksum_error {
    mismatch,
}

fn validate_checksum(buffer: &[u8]) -> Result<(), checksum_error> {
    if (buffer.len() > 4) {
        let buffer_without_trailing_zeros = buffer.split_at((buffer[1] + 2) as usize).0;
        let buffer_without_checksum =
            buffer_without_trailing_zeros.split_at(buffer_without_trailing_zeros.len() - 4);
        let calculated_read_buffer_checksum = get_checksum(buffer_without_checksum.0);
        let checksum = ((buffer_without_checksum.1[0] as u32) << 24)
            | ((buffer_without_checksum.1[1] as u32) << 16)
            | ((buffer_without_checksum.1[2] as u32) << 8)
            | ((buffer_without_checksum.1[3] as u32));

        if (checksum == calculated_read_buffer_checksum) {
            Ok(())
        } else {
            Err(checksum_error::mismatch)
        }
    } else {
        Err(checksum_error::mismatch)
    }
}

enum com_error {
    read_error,
    write_error,
}

fn get_checksum(bytes: &[u8]) -> u32 {
    crc32::checksum_ieee(bytes)
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
