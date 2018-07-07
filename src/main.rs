use std::net::UdpSocket;
use std::time::Duration;
use std::io::Write;
extern crate time;
extern crate crc;
use crc::{crc32, Hasher32};

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

#[derive(Debug,PartialEq)]
enum data_recieve_error{
    protocol_error,
    could_not_recieve_data,
    possible_corrupted_data,
    empty_buffer,
    mismatch_checksum

}

#[derive(Debug,PartialEq)]
enum device_error{
    could_not_setup_devices,
    could_not_send_setup_command,
    could_not_unserialize_device_packet,
    could_not_recieve_device_packet

}
#[derive(Debug,PartialEq)]
enum checksum_error
{
    mismatch
}

#[derive(Debug,PartialEq)]
enum parent_command
{
    status,
    help,
    exit,
    no_parent_command
}

enum command_execution_error
{
    empty_command
}

trait CommandExecution
{
    fn execute(&self) ->Result<(),command_execution_error>;
}


#[derive(Debug)]
struct Device
{
    device_id : u8,
    status : Status
}

fn listen_to_commands()
{
    println!("Enter COMMAND");
    std::io::stdout().flush();
    let mut command: String = String::new();
    loop
    {
        command = String::new();
        std::io::stdin().read_line(&mut command);
        match parse_command(&command)
        {
            Ok(exec) =>
            {
                exec.execute();
            },
            Err(_)=>
            {
               println!("invalid command");
            }
        }
        
    }
}

struct stub
{

}


impl CommandExecution for stub
{
    fn execute(&self)->Result<(),command_execution_error>
    {
        println!("Stub");
        Ok(())
    }
}

fn find_parent_command(command : &str) -> parent_command
{
     println!("{:?} was entered",command);
      match &*command.to_uppercase()
      {
          "STATUS" =>
          {
              parent_command::status
          },
          "HELP" =>
          {
              parent_command::help
          },
          "EXIT" =>
          {
              parent_command::exit
          },
          _ =>
          {
              parent_command::no_parent_command
          }
      }
}

fn evaluate_status_command(tokenized_command : impl Iterator ) ->Result<impl CommandExecution,command_execution_error>
{
    Ok(stub{}) 
}

fn parse_command<'a>(command : &'a String) ->Result<impl CommandExecution + 'a,command_execution_error>
{
    let mut tokenized_command = command.split_whitespace();
    match tokenized_command.next()
    {
        Some(first_command) =>
        {
            let parent_command = find_parent_command(first_command);
            match parent_command
            {
                parent_command::status =>
                {
                     evaluate_status_command(tokenized_command)
                },
                _ =>
                {
                    unimplemented!("have not implemented default case")
                }
            }
        },
        None=>
        {
            Err(command_execution_error::empty_command)
        }
    }

}

fn main() 
{
    println!("Starting Jarvis");
    println!("Checking For Devices");
    let read_udp_socket = UdpSocket::bind("0.0.0.0:61000").expect("COULD NOT BIND TO UDP PACKET");
    let send_udp_socket = UdpSocket::bind("0.0.0.0:62345").expect("COULD NOT BIND SEND SOCKET");
    set_up_socket(&read_udp_socket).expect("could not set up read socket");
    set_up_socket(&send_udp_socket).expect("could not set up send socket");
    match set_up_devices(&read_udp_socket,&send_udp_socket)
    {
        Ok(devices)=>
        {
            println!("Set up devices : devices {:?}",devices);
            listen_to_commands();

        },
        Err(err) =>
        {
            println!("JARVIS COULD NOT START: {:?}",err);
        }
    };

    listen_to_commands();

 

    
}


fn set_up_socket(udp_socket : &UdpSocket) -> Result<(),socket_setup_error>
{
    match udp_socket.set_write_timeout(Some(Duration::new(30, 0)))
    {
        Ok(_) =>
        {
            match udp_socket.set_read_timeout(Some(Duration::new(30, 0)))
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

fn set_up_devices(read_udp_socket : &UdpSocket,send_udp_socket : &UdpSocket) -> Result<Vec<Device>,device_error>
{
    match send_udp_socket.connect("255.255.255.255:62344")
    {
        Ok(result) =>
        {
            let current_time = time::now();
            let set_up_command = set_up_command_broadcast();
            match send_udp_socket.send(&set_up_command)
            {
                Ok(_) =>
                {
                    retrieve_devices(read_udp_socket)
                },
                Err(_)=>
                {
                    Err(device_error::could_not_setup_devices)
                }
            }
        },
        Err(_) =>{
            Err(device_error::could_not_send_setup_command)
        }
    }

}

fn set_up_command_broadcast() -> [u8;11]
{
    let current_time = time::now();
    let command_without_checksum = [165,5,0,0,current_time.tm_hour as u8,current_time.tm_min as u8,current_time.tm_sec as u8];
    let checksum = get_checksum(&command_without_checksum);
    [165,9,0,0,current_time.tm_hour as u8,current_time.tm_min as u8,current_time.tm_sec as u8, 
    (checksum>>24) as u8,
    (checksum>>16) as u8,
    (checksum>>8) as u8,
    (checksum | 0x00ff) as u8]


}

fn get_checksum(bytes : &[u8]) -> u32
{
    crc32::checksum_ieee(bytes)
}




fn retrieve_devices(udp_socket : &UdpSocket) -> Result<Vec<Device>,device_error>
{
    let mut devices : Vec<Device> = vec![];

    let mut buffer = [0;256];
    //udp_socket.connect("0.0.0.0:56000").expect("Could not bind to 62344");
    match udp_socket.recv_from(&mut buffer)
    {
        Ok(success) =>
        {
            match get_device_from_bytes(&buffer)
            {
                Ok(device_from_buffer) =>
                {
                    devices.push(device_from_buffer);
                    match retrieve_devices(udp_socket)
                    {
                        Ok(recursive_device) =>
                        {

                            devices.extend(recursive_device);
                            Ok(devices)
                        },
                        Err(_) =>
                        {
                            Ok(devices)
                        }
                    }
                },
                Err(e)=>
                {
                    Err(device_error::could_not_unserialize_device_packet)
                }
            }
            
        },
        Err(e) =>
        {
            Err(device_error::could_not_recieve_device_packet)
        }
    }
    

}


fn get_device_from_bytes(buffer:&[u8]) -> Result<Device,data_recieve_error>
{
    match buffer.first()
    {
        Some(first) =>
        {
            if(*first==165u8)
            {
                match validate_checksum(buffer)
                {
                    Ok(validated_buffer) =>
                    {
                        match create_buffer_from_device(buffer)
                        {
                            Some(result) => Ok(result),
                            None => Err(data_recieve_error::possible_corrupted_data)
                        }

                    },
                    Err(_) =>{Err(data_recieve_error::mismatch_checksum)}
                }
            }
            else
            {
                Err(data_recieve_error::protocol_error)
            }
        },
        None =>Err(data_recieve_error::empty_buffer)
    }
    

}

fn validate_checksum(buffer: &[u8]) -> Result<(),checksum_error>
{
    if(buffer.len()>4)
    {
        let buffer_without_trailing_zeros = buffer.split_at((buffer[1]+2) as usize).0;
        let buffer_without_checksum = buffer_without_trailing_zeros.split_at(buffer_without_trailing_zeros.len()-4);
        let calculated_read_buffer_checksum = get_checksum(buffer_without_checksum.0);
        let checksum = ((buffer_without_checksum.1[0] as u32) <<24) |
                    ((buffer_without_checksum.1[1] as u32)  <<16) |
                    ((buffer_without_checksum.1[2] as u32)  <<8) |
                    ((buffer_without_checksum.1[3] as u32) ) ;
      
        if(checksum==calculated_read_buffer_checksum)
        {
            Ok(())
        }
        else
        {
            Err(checksum_error::mismatch)
        }
    }
    else
    {
        Err(checksum_error::mismatch)
    }
    
    

}

fn create_buffer_from_device(buffer:&[u8]) -> Option<Device>
{
    if(buffer.len()>2)
    {
        Some(Device
        {
             device_id : buffer[2],
             status : Status::on
        })
    
    }
    else
    {
        None
    }
    
}   

#[test]
fn test_validate_checksum()
{
    assert_eq!(validate_checksum(&[165,9,15,1,0,0,0,0x95,0x1c,0x82,0xcb]),Ok(()));
    assert_eq!(validate_checksum(&[165,9,15,1,0,0,0,0x95,0x1c,0x82,0xcb,0,0,0,0,23]),Ok(()));
    assert_eq!(validate_checksum(&[165,9,19,1,0,0,0,0x95,0x1C,0x82,0xCB]).expect_err("Matching checksum"),checksum_error::mismatch);

}

#[test]
fn test_get_device_from_bytes()
{
    
    assert_eq!(get_device_from_bytes(&[]).expect_err("should expect protocol error"),data_recieve_error::empty_buffer);
    assert_eq!(get_device_from_bytes(&[00,9,15,1,0,0,0,0x95,0x1C,0x82,0xCB]).expect_err("should expect protocol error"),data_recieve_error::protocol_error);
    assert_eq!(get_device_from_bytes(&[165,9,15,1,0,0,0,0,0,0,0]).expect_err("expected corrupted data"),data_recieve_error::mismatch_checksum);
    assert_eq!(get_device_from_bytes(&[165,9,15,1,0,0,0,0x95,0x1C,0x82,0xCB]).expect("expected device").device_id,15);
    

}


#[test]
fn test_parent_status_match()
{
    assert_eq!(find_parent_command("sTatus"),parent_command::status);
    assert_eq!(find_parent_command("eXit"),parent_command::exit);
    assert_eq!(find_parent_command("hELp"),parent_command::help);
}



