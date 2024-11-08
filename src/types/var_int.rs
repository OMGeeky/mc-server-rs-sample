use crate::types::{McRead, McRustRepr, McWrite};
use std::io::{Read, Write};
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct VarInt(pub i32);
impl Deref for VarInt {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl McWrite for VarInt {
    type Error = std::io::Error;

    fn write_stream<T: Write>(&self, stream: &mut T) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        let mut value = self.0 as u32;
        loop {
            if (value & Self::SEGMENT_BITS as u32) == 0 {
                let _ = stream.write(&[value.to_le_bytes()[0]])?;
                return Ok(1);
            }

            let x = value & Self::SEGMENT_BITS as u32 | Self::CONTINUE_BIT as u32;
            let x = x.to_le_bytes()[0];
            let _ = stream.write(&[x])?;
            value >>= 7;
        }
    }
}
impl McRead for VarInt {
    type Error = String;
    fn read_stream<T: Read>(b: &mut T) -> Result<Self, Self::Error> {
        let mut value = 0i32;
        let mut position = 0;
        // println!("CONTINUE bit: {:0>32b}", Self::CONTINUE_BIT);
        // println!("SEGMENT  bit: {:0>32b}", Self::SEGMENT_BITS);

        loop {
            let mut current_byte = 0u8;
            b.read_exact(std::slice::from_mut(&mut current_byte))
                .map_err(|x| x.to_string())?;
            // println!(
            //     "b: {:0>32b}\nm: {:0>32b}\nr: {:0>32b}\n>: {:0>32b} ({position})\nv: {:0>32b}\nr2:{:0>32b}",
            //     current_byte,
            //     Self::SEGMENT_BITS,
            //     current_byte & Self::SEGMENT_BITS,
            //     ((current_byte & Self::SEGMENT_BITS) as i32) << position,
            //     value,
            //     value | (((current_byte & Self::SEGMENT_BITS) as i32) << position)
            // );
            value |= ((current_byte & Self::SEGMENT_BITS) as i32) << position;
            // println!("{x}:\n{current_byte:>32b}:\n{value:>32b}:\n{value}");

            if (current_byte & Self::CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                return Err("VarInt is too big".to_string());
            }
        }

        Ok(Self(value))
    }
}
impl VarInt {
    const SEGMENT_BITS: u8 = 0x7F;
    const CONTINUE_BIT: u8 = 0x80;
}
impl McRustRepr for VarInt {
    type RustRepresentation = i32;

    fn into_rs(self) -> Self::RustRepresentation {
        self.0
    }

    fn to_rs(&self) -> Self::RustRepresentation {
        self.0
    }

    fn as_rs(&self) -> &Self::RustRepresentation {
        &self.0
    }
}
