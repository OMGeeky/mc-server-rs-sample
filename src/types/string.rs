use crate::types::var_int::VarInt;
use crate::types::{McRead, McRustRepr, McWrite};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct McString<const MAX_SIZE: usize> {
    pub value: String,
}
impl<const MAX_SIZE: usize> McString<MAX_SIZE> {
    pub fn measure_size(s: &str) -> usize {
        // 3. UTF-8 encoded byte length
        let utf8_len = s.bytes().len();

        // 5. Calculate total length (including VarInt prefix)
        let varint_size = VarInt(utf8_len as i32).get_size();
        if varint_size > 3 {
            //TODO: This is not allowed
        }
        // println!("strlen: {}+({}?{})", varint_size, utf8_len, utf16_len);
        varint_size + utf8_len
    }
    pub fn from_string(s: String) -> Self {
        Self { value: s }
    }
}
impl<const MAX_SIZE: usize> McRead for McString<MAX_SIZE> {
    type Error = ();

    async fn read_stream<T: AsyncRead + Unpin>(b: &mut T) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let max_size = VarInt::read_stream(b).await.map_err(|x| {
            dbg!(x);
        })?;
        let size = *max_size as usize;
        println!("Reading string of length: {}", size);

        // Check if the size exceeds the maximum allowed length (n)
        if size > (MAX_SIZE * 3) + 3 {
            return Err(()); // Or a more specific error type
        }

        let mut bytes = vec![0u8; size];
        let actual_size = b.read(&mut bytes).await.map_err(|x| {
            dbg!(x);
        })?;
        let value = String::from_utf8(bytes).map_err(|x| {
            dbg!(x);
        })?;
        assert_eq!(
            size, actual_size,
            "Did not read all that was to read {}",
            value
        );
        Ok(Self { value })
    }
}
impl<const MAX_SIZE: usize> McWrite for McString<MAX_SIZE> {
    type Error = std::io::Error;

    async fn write_stream<T: AsyncWrite + Unpin>(
        &self,
        stream: &mut T,
    ) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        let buf = self.value.as_bytes();
        let length = Self::measure_size(&self.value);
        println!("string length: {}", length);
        let length_var_int = VarInt(length as i32);
        let written = length_var_int.write_stream(stream).await?;
        println!("Writing String to stream: '{}'", self.value);
        stream.write_all(buf).await?;
        Ok(buf.len() + written)
    }
}
impl<const MAX_SIZE: usize> McRustRepr for McString<MAX_SIZE> {
    type RustRepresentation = String;

    fn into_rs(self) -> Self::RustRepresentation {
        self.value
    }

    fn to_rs(&self) -> Self::RustRepresentation {
        self.value.to_owned()
    }

    fn as_rs(&self) -> &Self::RustRepresentation {
        &self.value
    }
}
