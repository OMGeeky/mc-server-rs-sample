pub mod protocols;
pub mod types;
pub mod utils;

use crate::protocols::handle;
use crate::types::string::McString;
use crate::types::var_int::VarInt;
use crate::types::{McRead, McRustRepr};
use crate::utils::RWStreamWithLimit;
use num_derive::FromPrimitive;
use num_traits::{FromPrimitive, ToPrimitive};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn main() {
    println!("Hello, world!");
    let listener = TcpListener::bind("127.0.0.1:25565").unwrap();
    println!("Listening started.");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    println!("===============START=====================");

                    stream
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    stream
                        .set_write_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    println!(
                        "Timeout for connection: {:?}/{:?}",
                        stream.read_timeout(),
                        stream.write_timeout()
                    );
                    handle_connection(stream);
                    println!("===============DONE======================");
                });
            }
            Err(err) => {
                dbg!(err);
            }
        }
    }
}
#[derive(FromPrimitive, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
enum ConnectionState {
    NotConnected = 0,
    Status = 1,
    Login = 2,
    Transfer = 3,
    Closed = -1,
}
struct Connection {
    connection_state: ConnectionState,
    tcp_stream: TcpStream,
    compression_active: bool,
}
impl Connection {
    fn handle(&mut self) -> Result<(), String> {
        while self.connection_state != ConnectionState::Closed {
            let x = self.tcp_stream.peek(&mut [0]); //see if we have at least one byte available
            match x {
                Ok(size) => {
                    println!("we should have 1 here: {size}");
                    if size == 0 {
                        println!("Reached end of stream.");
                        self.connection_state = ConnectionState::Closed;
                        continue;
                    }
                }
                Err(_) => {
                    println!("could not peek if we reached the end of the stream.");
                }
            }

            let length = VarInt::read_stream(&mut self.tcp_stream)?;
            println!("packet length: {}", length.as_rs());
            let bytes_left_in_package = length.to_rs();

            let mut package_stream = RWStreamWithLimit::new(
                &mut self.tcp_stream,
                bytes_left_in_package.to_usize().unwrap(),
            );
            let result = Self::handle_package(
                &mut package_stream,
                self.connection_state,
                self.compression_active,
            );
            match result {
                Ok(new_connection_state) => {
                    assert_eq!(
                        package_stream.get_read_left(),
                        0,
                        "The not failed package did not use up all its bytes or used to much!"
                    );
                    self.connection_state = new_connection_state;
                }
                Err(e) => {
                    //discard rest of package for failed ones
                    discard_read(&mut self.tcp_stream, bytes_left_in_package.to_u8().unwrap())
                        .map_err(|x| x.to_string())?;

                    println!("Got an error during package handling: {e}");
                }
            }
        }

        Ok(())
    }
    fn handshake<T: Read + Write>(
        stream: &mut T,
        _compression: bool,
        // bytes_left_in_package: &mut i32,
    ) -> Result<ConnectionState, String> {
        // println!("bytes left:{}", bytes_left_in_package);
        let protocol_version = VarInt::read_stream(stream)?;
        // *bytes_left_in_package -= read as i32;
        println!("protocol version: {}", protocol_version.as_rs());
        // println!("bytes left:{}", bytes_left_in_package);
        let address =
            McString::read_stream(stream).map_err(|_| "Could not read string".to_string())?;
        // *bytes_left_in_package -= read as i32;
        println!("address: '{}'", address.as_rs());
        stream.read_exact(&mut [0, 2]).unwrap(); //server port. Unused
                                                 // *bytes_left_in_package -= 2;
        let next_state_id = VarInt::read_stream(stream)?;
        // *bytes_left_in_package -= read as i32;
        println!("next state: {}", next_state_id.as_rs());
        // println!("bytes left:{}", bytes_left_in_package);
        let next_state = FromPrimitive::from_i32(next_state_id.to_rs());
        match next_state {
            Some(next_state) => Ok(next_state),
            None => Err(format!(
                "Got an unknown next state: {}",
                next_state_id.as_rs()
            )),
        }
    }
    fn handle_package<T: Read + Write>(
        stream: &mut RWStreamWithLimit<T>,
        connection_state: ConnectionState,
        compression: bool,
        // bytes_left_in_package: usize,
    ) -> Result<ConnectionState, String> {
        // let mut stream = RWStreamWithLimit::new(stream, bytes_left_in_package);
        // let stream = &mut stream;
        let packet_id = VarInt::read_stream(stream)?;
        // *bytes_left_in_package = i32::max(*bytes_left_in_package - read as i32, 0);

        println!("id: {:0>2x}", packet_id.as_rs());
        if connection_state == ConnectionState::NotConnected && packet_id.to_rs() == 0x00 {
            return Self::handshake(stream, compression);
        }
        match FromPrimitive::from_i32(packet_id.to_rs()) {
            Some(protocol) => {
                // println!("bytes left:{}", bytes_left_in_package);
                let res = handle(protocol, stream);
                // println!("bytes left:{}", bytes_left_in_package);
                match res {
                    Ok(_) => {
                        // println!("bytes left:{}", bytes_left_in_package);
                        println!("Success!");
                    }
                    Err(_) => {
                        stream.discard_unread().map_err(|x| x.to_string())?;
                        // println!("bytes left:{}", bytes_left_in_package);
                        // *bytes_left_in_package -= discard_read(stream, *bytes_left_in_package as u8)
                        //     as i32;
                        println!("Failure :(");
                    }
                }
            }
            None => {
                stream.discard_unread().map_err(|x| x.to_string())?;
                // *bytes_left_in_package -= discard_read(stream, *bytes_left_in_package as u8)
                //     .map_err(|x| x.to_string())? as i32;
                println!("I don't know this protocol yet, so Im gonna ignore it...");
            }
        }
        Ok(connection_state)
    }
}
fn handle_connection(stream: TcpStream) {
    let mut connection = Connection {
        connection_state: ConnectionState::NotConnected,
        tcp_stream: stream,
        compression_active: false,
    };
    let result = connection.handle();
    if let Err(e) = result {
        dbg!(e);
    }
}
fn discard_read<T: Read>(stream: &mut T, bytes: u8) -> Result<usize, std::io::Error> {
    stream.read_exact(&mut [0, bytes])?;
    Ok(bytes as usize)
}
