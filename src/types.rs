use std::io::{Read, Write};

pub trait McRead {
    type Error;
    fn read_stream<T: Read>(stream: &mut T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
pub trait McWrite {
    type Error;
    fn write_stream<T: Write>(&self, stream: &mut T) -> Result<usize, Self::Error>
    where
        Self: Sized;
}
pub trait McRustRepr {
    type RustRepresentation;
    fn into_rs(self) -> Self::RustRepresentation;
    fn to_rs(&self) -> Self::RustRepresentation;
    fn as_rs(&self) -> &Self::RustRepresentation;
}
pub mod string;
pub mod var_int;
