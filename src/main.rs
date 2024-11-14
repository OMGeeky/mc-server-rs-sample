#![allow(unused)]
pub mod protocols;
pub mod types;
pub mod utils;

use crate::types::string::McString;
use crate::types::var_int::VarInt;
use crate::types::{McRead, McRustRepr};
use crate::utils::{MyAsyncReadExt, RWStreamWithLimit};
use num_derive::FromPrimitive;
use num_traits::{FromPrimitive, ToPrimitive};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), ()> {
    println!("Hello, world!");
    // let listener = TcpListener::bind("127.0.0.1:25565").unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:25565")
        .await
        .unwrap();
    println!("Listening started.");
    loop {
        let (stream, socket) = listener.accept().await.map_err(|x| {
            dbg!(x);
        })?;

        tokio::spawn(async move {
            println!("===============START=====================");
            dbg!(&socket);
            handle_connection(stream).await;
            println!("===============DONE======================");
        });
    }
}
#[derive(FromPrimitive, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
enum ConnectionState {
    NotConnected = 0,
    Status = 1,
    Login = 2,
    Transfer = 3,
    ///Internal use
    Closed = -1,
}
struct Connection {
    connection_state: ConnectionState,
    tcp_stream: TcpStream,
    compression_active: bool,
}
impl Connection {
    async fn handle(&mut self) -> Result<(), String> {
        while self.connection_state != ConnectionState::Closed {
            let x = self.tcp_stream.peek(&mut [0]).await; //see if we have at least one byte available
            match x {
                Ok(size) => {
                    if size == 0 {
                        println!("Reached end of stream.");
                        self.connection_state = ConnectionState::Closed;
                        continue;
                    }
                    // else if size == 0xFE {
                    //   //  Legacy Ping (see https://wiki.vg/Server_List_Ping#1.6)
                    // handle_legacy_ping(&mut self.tcp_stream).await?;
                    // }
                }
                Err(_) => {
                    println!("could not peek if we reached the end of the stream.");
                }
            }

            let length = VarInt::read_stream(&mut self.tcp_stream).await?;
            if *length == 0xFE {
                //Legacy Ping (see https://wiki.vg/Server_List_Ping#1.6)
                let x = handle_legacy_ping(&mut self.tcp_stream).await;
                self.connection_state = ConnectionState::Closed;
                self.tcp_stream.shutdown().await.map_err(|e| {
                    dbg!(e);
                    "?"
                })?;
                continue;
            }
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
            )
            .await;
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
                    self.connection_state = ConnectionState::Closed;
                    dbg!(&self.tcp_stream.shutdown().await);
                    println!("Got an error during package handling: {e}");
                }
            }
        }

        Ok(())
    }
    async fn handshake<T: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut T,
        _compression: bool,
        // bytes_left_in_package: &mut i32,
    ) -> Result<ConnectionState, String> {
        let handshake_data = protocols::handshake::Data::read_stream(stream).await?;
        // dbg!(&handshake_data);
        Ok(handshake_data.next_state)
        // let protocol_version = VarInt::read_stream(stream).await?;
        // println!("protocol version: {}", protocol_version.as_rs());
        // let address: McString<255> = McString::read_stream(stream)
        //     .await
        //     .map_err(|_| "Could not read string".to_string())?;
        // println!("address: '{}'", address.as_rs());
        // stream.discard(2).await.unwrap(); //server port. Unused
        // let next_state_id = VarInt::read_stream(stream).await?;
        // println!("next state: {}", next_state_id.as_rs());
        // let next_state = FromPrimitive::from_i32(next_state_id.to_rs());
        // match next_state {
        //     Some(next_state) => Ok(next_state),
        //     None => Err(format!(
        //         "Got an unknown next state: {}",
        //         next_state_id.as_rs()
        //     )),
        // }
    }
    async fn handle_package<T: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut RWStreamWithLimit<'_, T>,
        connection_state: ConnectionState,
        compression: bool,
    ) -> Result<ConnectionState, String> {
        let packet_id = VarInt::read_stream(stream).await?;

        println!(
            "Handling new Package with id: {:0>2x} =======================",
            packet_id.as_rs()
        );
        if connection_state == ConnectionState::NotConnected && packet_id.to_rs() == 0x00 {
            return Self::handshake(stream, compression).await;
        }
        match FromPrimitive::from_i32(packet_id.to_rs()) {
            Some(protocol) => {
                let res = types::package::Package::handle(protocol, stream).await;
                // let res = protocols::handle(protocol, stream).await;
                match res {
                    Ok(_) => {
                        println!("Success!");
                    }
                    Err(terminate_connection) => {
                        if terminate_connection {
                            return Err("Something terrible has happened!".to_string());
                        } else {
                            stream.discard_unread().await.map_err(|x| x.to_string())?;
                        }
                        println!("Failure :(");
                    }
                }
            }
            None => {
                stream.discard_unread().await.map_err(|x| x.to_string())?;
                // *bytes_left_in_package -= discard_read(stream, *bytes_left_in_package as u8)
                //     .map_err(|x| x.to_string())? as i32;
                println!("I don't know this protocol yet, so Im gonna ignore it...");
            }
        }
        Ok(connection_state)
    }
}

async fn handle_legacy_ping(stream: &mut TcpStream) -> Result<(), String> {
    println!("handling legacy ping");
    let id = stream.read_u8().await.map_err(|e| e.to_string())?;
    let payload = stream.read_u8().await.map_err(|e| e.to_string())?;
    let plugin_message_ident = stream.read_u8().await.map_err(|e| e.to_string())?;

    stream
        .write_all(&[
            0xfe, // 1st packet id: 0xfe for server list ping
            0x01, // payload: always 1
            0xfa, // 2nd packet id: 0xfa for plugin message
            0x00, 0x0b, // length of following string: always 11 as short,
            0x00, 0x4d, 0x00, 0x43, 0x00, 0x7c, 0x00, 0x50, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67,
            0x00, 0x48, 0x00, 0x6f, 0x00, 0x73, 0x00, 0x74,
            // ^^ MC|PingHost as UTF16-BE

            // length of the rest of the data
            13, // ^^
            // protocol version: 127 for the invalid version, to signal, client is too old
            0, 49, 0, 50, 0, 55, // ^^
            0x00, 0x00, // length of hostname: 0 as short
            0x00, 0x00, 0x00, 0x00, // port: 0 as int
        ])
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn handle_connection(stream: TcpStream) {
    let mut connection = Connection {
        connection_state: ConnectionState::NotConnected,
        tcp_stream: stream,
        compression_active: false,
    };
    let result = connection.handle().await;
    if let Err(e) = result {
        dbg!(e);
    }
}
