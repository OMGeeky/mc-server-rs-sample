#![allow(unused)]
pub mod protocols;
pub mod types;
pub mod utils;

use crate::types::package::IncomingPackage;
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
pub(crate) enum ConnectionState {
    NotConnected = 0,
    Status = 1,
    Login = 2,
    Transfer = 3,
    Configuration = 4,
    Play = 5,
    ///Internal use
    Closed = -1,
}
struct Connection {
    connection_state: ConnectionState,
    tcp_stream: TcpStream,
    compression_active: bool,
}
impl Connection {
    async fn shutdown(&mut self) {
        self.connection_state = ConnectionState::Closed;
        self.tcp_stream.shutdown().await;
    }
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
                self.shutdown().await;
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
                    self.shutdown().await;
                    println!("Got an error during package handling: {e}");
                }
            }
        }

        Ok(())
    }
    async fn handle_package<T: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut RWStreamWithLimit<'_, T>,
        connection_state: ConnectionState,
        compression: bool,
    ) -> Result<ConnectionState, String> {
        let res = IncomingPackage::handle_incoming(stream, connection_state).await?;
        if let Some(new_connection_state) = res {
            return Ok(new_connection_state);
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
