use std::net::UdpSocket;
use std::time::Duration;
extern crate time;
extern crate checksum;
use checksum::crc32;

#[derive(Debug)]
enum Status {
    on,
    off,
}


#[derive(Debug)]
enum socket_setup_error{
    could_not_set_read_timeout,
    could_not_set_write_timeout,
    could_not_set_broadcast,
}

#[derive(Debug)]
enum data_transmittion_error{
    could_not_send_data
}



struct Device
{
    ip_address : String,
    mac_address : String,
    status : Status
}


fn main() 
{
    println!("Starting Jarvis");
    println!("Checking For Devices");
    let udp_socket = UdpSocket::bind("127.0.0.1:34254").expect("COULD NOT BIND TO UDP PACKET");
    set_up_socket(&udp_socket).expect("could not set up socket");
    let devices = set_up_devices(&udp_socket);
    
}

fn set_up_socket(udp_socket : &UdpSocket) -> Result<(),socket_setup_error>
{
    match udp_socket.set_write_timeout(Some(Duration::new(3, 0)))
    {
        Ok(_) =>
        {
            match udp_socket.set_read_timeout(Some(Duration::new(3, 0)))
            {
                Ok(_) =>
                {
                    match udp_socket.set_broadcast(true)
                    {
                        Ok(_) =>{Ok(())},
                        Err(_)=>{Err(socket_setup_error::could_not_set_broadcast)}
                    }
                },
                Err(_) => {Err(socket_setup_error::could_not_set_write_timeout)}
            }
        },
        Err(_) => {Err(socket_setup_error::could_not_set_write_timeout)}
    }
}

fn set_up_devices(udp_socket : &UdpSocket) -> Vec<Device>
{
    match udp_socket.connect("255.255.255.255:62345")
    {
        Ok(result) =>
        {
            let current_time = time::now();
            let set_up_command = set_up_command_broadcast();
            match udp_socket.send(&set_up_command)
            {
                Ok(_) =>
                {
                    retrieve_devices(udp_socket)
                },
                Err(_)=>
                {
                    vec![]
                }
            }
        },
        Err(_) =>{
            vec![]
        }
    }

}

fn set_up_command_broadcast() -> [u8;9]
{
    let current_time = time::now();
    let command_without_checksum = [165,5,0,0,current_time.tm_hour as u8,current_time.tm_min as u8,current_time.tm_sec as u8];
    let checksum = get_checksum(&command_without_checksum);
    [165,5,0,0,current_time.tm_hour as u8,current_time.tm_min as u8,current_time.tm_sec as u8,(checksum>>8) as u8,
    (checksum | 0x00ff) as u8]


}

fn get_checksum(bytes : &[u8]) -> u32
{
    let mut checksum = crc32::Crc32::new();
    checksum.checksum(bytes)
}




fn retrieve_devices(udp_socket : &UdpSocket) -> Vec<Device>
{
     unimplemented!("Retrieve Devices From LAN")
}






