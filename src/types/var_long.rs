use crate::types::{McRead, McRustRepr, McWrite};

use std::ops::Deref;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Copy, Clone)]
pub struct VarLong(pub i64);
impl Deref for VarLong {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl VarLong {
    pub(crate) fn get_size(&self) -> usize {
        let value = self.0 as u64; // Convert to u64 for bitwise operations
        crate::types::var_int::get_number_bytes_size(value)
    }
}
impl McWrite for VarLong {
    type Error = std::io::Error;

    async fn write_stream<T: AsyncWrite + Unpin>(
        &self,
        stream: &mut T,
    ) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        let value = self.0 as u64;
        write_varlong(stream, self.0).await
    }
}
impl McRead for VarLong {
    async fn read_stream<T: AsyncRead + Unpin>(b: &mut T) -> Result<Self, String> {
        let value = crate::types::var_int::read_stream(b, 64).await? as i64;
        Ok(Self(value))
    }
}
impl McRustRepr for VarLong {
    type RustRepresentation = i64;

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

async fn write_varlong<W: AsyncWrite + Unpin>(
    writer: &mut W,
    mut value: i64,
) -> std::io::Result<usize> {
    let mut written = 0;
    loop {
        written += 1;
        if (value & !SEGMENT_BITS) == 0 {
            writer.write_u8(value as u8).await.unwrap();
            break;
        }

        writer
            .write_u8(((value & SEGMENT_BITS) | CONTINUE_BIT) as u8)
            .await
            .unwrap();

        value = (value as u64 >> 7) as i64; // Simulate Java's >>>
    }
    Ok(written)
}

const SEGMENT_BITS: i64 = 0x7F;
const CONTINUE_BIT: i64 = 0x80;

#[cfg(test)]
mod tests {
    use crate::types::var_long::VarLong;
    use crate::types::{McRead, McWrite};
    use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
    use tokio_util::bytes::BufMut;

    #[tokio::test]
    async fn test_varlong_read_stream() {
        let test_cases = [
            (vec![0x00], 0),
            (vec![0x01], 1),
            (vec![0x02], 2),
            (vec![0x7f], 127),
            (vec![0x80, 0x01], 128),
            (vec![0xff, 0x01], 255),
            (vec![0xff, 0xff, 0xff, 0xff, 0x07], 2147483647),
            (
                vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
                9223372036854775807,
            ),
            (
                vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
                -1,
            ),
            (
                vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
                -2147483648,
            ),
            (
                vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
                -9223372036854775808,
            ),
        ];

        for (input_bytes, expected_value) in test_cases {
            let mut reader = BufReader::new(input_bytes.as_slice());
            let varint = VarLong::read_stream(&mut reader).await.unwrap();
            assert_eq!(
                *varint, expected_value,
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
