extern crate crc;
extern crate pnet;

use self::pnet::datalink::{self, NetworkInterface,Channel::Ethernet,Config};
use self::pnet::packet::{Packet, MutablePacket,ethernet::{EthernetPacket,MutableEthernetPacket}};
use self::pnet::util::MacAddr;
use self::crc::{crc32, Hasher32};
use std::io;
use std::io::prelude::*;
use std::net::UdpSocket;
use std::time::Duration;


#[derive(Debug)]
pub enum socket_setup_error {
    could_not_set_read_timeout,
    could_not_set_write_timeout,
    could_not_set_broadcast,
}

pub struct TransportLayerChannel {
    pub read_udp_socket: UdpSocket,
    pub write_udp_socket: UdpSocket,
}

pub enum DataLinkError{
    interface_error_not_available
}

pub struct DataLinkLayerChannel 
{
    datalink : NetworkInterface,
    source_addr : MacAddr,
    destination_addr : MacAddr
}

impl DataLinkLayerChannel
{
    pub fn new(datalink_name:&str,source_mac_addr : (u8,u8,u8,u8,u8,u8),dest_mac_addr : (u8,u8,u8,u8,u8,u8)) -> Result<DataLinkLayerChannel,DataLinkError>
    {
        match datalink::interfaces().into_iter()
              .filter(|net|{
                  net.name == datalink_name
              }).next()
        {
            Some(link)=>
            {
                Ok(DataLinkLayerChannel
                {
                    datalink : link,
                    source_addr : MacAddr::new(source_mac_addr.0,source_mac_addr.1,source_mac_addr.2,source_mac_addr.3,source_mac_addr.4,source_mac_addr.5),
                    destination_addr : MacAddr::new(dest_mac_addr.0,dest_mac_addr.1,dest_mac_addr.2,dest_mac_addr.3,dest_mac_addr.4,dest_mac_addr.5)
                })
            },
            None =>
            {
                Err(DataLinkError::interface_error_not_available)
            }
        }
    }
}

impl TransportLayerChannel {
    pub fn new(read_IP: &str, write_IP: &str) -> TransportLayerChannel {
        let com_TransportLayerChannel = TransportLayerChannel {
            read_udp_socket: UdpSocket::bind(read_IP).expect("COULD NOT BIND TO UDP PACKET"),
            write_udp_socket: UdpSocket::bind(write_IP).expect("COULD NOT BIND SEND SOCKET"),
        };

        set_up_socket(&com_TransportLayerChannel.read_udp_socket)
            .expect("could not set up read socket");
        set_up_socket(&com_TransportLayerChannel.write_udp_socket)
            .expect("could not set up send socket");
        com_TransportLayerChannel
    }
}

impl io::Read for DataLinkLayerChannel
{
    fn read(&mut self, mut buffer: &mut [u8]) ->Result<usize, io::Error>
    {
        let mut read_data_command = [165u8,2u8,01];
        match datalink::channel(&self.datalink,Default::default())
        {
            Ok(Ethernet(mut tx, rx)) =>
            {
                match MutableEthernetPacket::new(&mut read_data_command.clone())
                {
                    Some(mut new_packet) =>
                    {
                            // Switch the source and destination
                            new_packet.set_source(self.source_addr);
                            new_packet.set_destination(self.destination_addr);

                            match tx.send_to(new_packet.packet(),None)
                            {
                                Some(_) =>
                                {
                                    let mut bytes_read = read_data_command.len();
                                    Ok(bytes_read)
                                },
                                None =>
                                {
                                    Err(io::Error::new(
                                    io::ErrorKind::BrokenPipe,
                                    "COULD NOT SEND READ DATA",
                        )) 
                                }
                            }
                    },
                    None =>
                    {
                        Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "COULD NOT CREATE PACKET",
                    ))
                    }
                   
                   
                }
                
            },
            Ok(_) =>
            {
                Err(io::Error::new(
                                io::ErrorKind::NotConnected,
                                "INVALID CHANNEL TYPE",
                    ))
                     
            },
            Err(_) =>
            {
                Err(io::Error::new(
                                io::ErrorKind::NotConnected,
                                "COULD NOT CREATE CHANNEL",
                    ))
            }
        }
    }
}

impl io::Read for TransportLayerChannel {
    fn read(&mut self, mut buffer: &mut [u8]) -> Result<usize, io::Error> {
        match self.read_udp_socket.recv_from(&mut buffer) {
            Ok(read_result) => match buffer.first() {
                Some(first) => {
                    if (*first == 165u8) {
                        match validate_checksum(buffer) {
                            Ok(validated_buffer) => Ok(buffer.len()),
                            Err(_) => Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "MISMATCHED CHECKSUM",
                            )),
                        }
                    } else {
                        Err(io::Error::new(io::ErrorKind::InvalidData, "PROTOCOL ERROR"))
                    }
                }
                None => Err(io::Error::new(io::ErrorKind::NotFound, "EMPTY BUFFER")),
            },
            Err(_) => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "COULD NOT READ BUFFER",
            )),
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
