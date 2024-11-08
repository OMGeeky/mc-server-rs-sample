use crate::protocols::status::StatusProtocol;
use crate::utils::RWStreamWithLimit;
use num_derive::FromPrimitive;
use std::io::{Read, Write};
// use num_traits::FromPrimitive;

#[derive(FromPrimitive)]
pub enum Protocols {
    Status = 0x00,
    Ping = 0x01,
}
pub fn handle<T: Read + Write>(
    protocol: Protocols,
    stream: &mut RWStreamWithLimit<T>,
    // bytes_left_in_package: &mut i32,
) -> Result<(), ()> {
    match protocol {
        Protocols::Status => StatusProtocol::handle(stream)?,
        Protocols::Ping => {}
    };
    Ok(())
}
mod status;
