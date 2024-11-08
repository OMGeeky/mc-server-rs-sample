use crate::types::var_int::VarInt;
use crate::types::{McRead, McRustRepr, McWrite};
use std::io::{Read, Write};

pub struct McString(pub String);
impl McRead for McString {
    type Error = ();

    fn read_stream<T: Read>(b: &mut T) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let size = VarInt::read_stream(b).map_err(|x| {
            dbg!(x);
        })?;
        let size = *size as usize;

        let mut bytes = vec![0u8; size];
        let actual_size = b.read(&mut bytes).map_err(|x| {
            dbg!(x);
        })?;
        assert_eq!(size, actual_size);
        let value = String::from_utf8(bytes).map_err(|x| {
            dbg!(x);
        })?;
        Ok(Self(value))
    }
}
impl McWrite for McString {
    type Error = std::io::Error;

    fn write_stream<T: Write>(&self, stream: &mut T) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        let buf = self.0.as_bytes();
        let length = buf.len(); //This does not actually count right (see https://wiki.vg/Protocol#Type:String)
        VarInt(length as i32).write_stream(stream)?;

        stream.write_all(buf)?;
        Ok(length)
    }
}
impl McRustRepr for McString {
    type RustRepresentation = String;

    fn into_rs(self) -> Self::RustRepresentation {
        self.0
    }

    fn to_rs(&self) -> Self::RustRepresentation {
        self.0.to_owned()
    }

    fn as_rs(&self) -> &Self::RustRepresentation {
        &self.0
    }
}
