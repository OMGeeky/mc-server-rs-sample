use crate::types::{McRead, McRustRepr, McWrite};

use num_traits::ToPrimitive;
use std::ops::Deref;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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

    async fn write_stream<T: AsyncWrite + Unpin>(
        &self,
        stream: &mut T,
    ) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        write_varint(stream, self.0).await
    }
}
impl McRead for VarInt {
    async fn read_stream<T: AsyncRead + Unpin>(b: &mut T) -> Result<Self, String> {
        let value = read_stream(b, 32).await? as i32;
        Ok(Self(value))
    }
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

pub(crate) async fn read_stream<T: AsyncRead + Unpin>(
    b: &mut T,
    max_size: usize,
) -> Result<u64, String> {
    let mut value = 0u64;
    let mut position = 0;

    loop {
        let current_byte = b.read_u8().await.map_err(|x| x.to_string())?;
        value |= ((current_byte & SEGMENT_BITS_U8) as u64) << position;
        if (current_byte & CONTINUE_BIT_U8) == 0 {
            break;
        }

        position += 7;

        if position >= max_size {
            return Err("Number is too big".to_string());
        }
    }

    Ok(value)
}

async fn write_varint<W: AsyncWrite + Unpin>(
    writer: &mut W,
    mut value: i32,
) -> std::io::Result<usize> {
    let mut written = 0;
    loop {
        written += 1;
        if (value & !SEGMENT_BITS) == 0 {
            writer.write_u8(value as u8).await?;
            break;
        }

        writer
            .write_u8(((value & SEGMENT_BITS) | CONTINUE_BIT) as u8)
            .await?;

        value = (value as u32 >> 7) as i32; // Simulate Java's >>>
    }
    Ok(written)
}

pub(crate) fn get_number_bytes_size(mut value: u64) -> usize {
    let mut size = 1;

    while value >= CONTINUE_BIT_U8 as u64 {
        value >>= 7;
        size += 1;
    }

    size
}

impl VarInt {
    pub(crate) fn get_size(&self) -> usize {
        let value = self.0 as u64; // Convert to u64 for bitwise operations
        get_number_bytes_size(value)
    }
}

const SEGMENT_BITS: i32 = 0x7F;
const CONTINUE_BIT: i32 = 0x80;
const SEGMENT_BITS_U8: u8 = SEGMENT_BITS as u8;
const CONTINUE_BIT_U8: u8 = CONTINUE_BIT as u8;

#[cfg(test)]
mod tests {
    use crate::types::var_int::VarInt;
    use crate::types::{McRead, McWrite};
    use tokio::io::{AsyncWriteExt, BufReader, BufWriter};

    #[tokio::test]
    async fn test_varint_read_stream() {
        let test_cases = [
            (vec![0x00], 0),
            (vec![0x01], 1),
            (vec![0x02], 2),
            (vec![0x7f], 127),
            (vec![0x80, 0x01], 128),
            (vec![0xff, 0x01], 255),
            (vec![0xdd, 0xc7, 0x01], 25565),
            (vec![0xff, 0xff, 0x7f], 2097151),
            (vec![0xff, 0xff, 0xff, 0xff, 0x07], 2147483647),
            (vec![0xff, 0xff, 0xff, 0xff, 0x0f], -1),
            (vec![0x80, 0x80, 0x80, 0x80, 0x08], -2147483648),
        ];

        for (input_bytes, expected_value) in test_cases {
            let mut reader = BufReader::new(input_bytes.as_slice());
            let varint = VarInt::read_stream(&mut reader).await.unwrap();
            assert_eq!(
                varint.0, expected_value,
                "Decoding mismatch for bytes: {:?}",
                input_bytes
            );
            let mut reversed_bytes = Vec::new();

            let mut writer = BufWriter::new(&mut reversed_bytes);
            varint.write_stream(&mut writer).await.unwrap();
            writer.flush().await.unwrap();

            assert_eq!(
                input_bytes, reversed_bytes,
                "Back and forth did not work for {}",
                *varint
            )
        }
    }
}
