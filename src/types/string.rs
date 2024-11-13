use crate::types::var_int::VarInt;
use crate::types::{McRead, McRustRepr, McWrite};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct McString<const MAX_SIZE: usize> {
    pub value: String,
}
impl<const MAX_SIZE: usize> McString<MAX_SIZE> {
    fn measure_size(s: &str) -> usize {
        s.len()
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

        // Check if the size exceeds the maximum allowed length (n)
        if size > (MAX_SIZE * 3) + 3 {
            return Err(()); // Or a more specific error type
        }

        let mut bytes = vec![0u8; size];
        let actual_size = b.read(&mut bytes).await.map_err(|x| {
            dbg!(x);
        })?;
        assert_eq!(size, actual_size);
        let value = String::from_utf8(bytes).map_err(|x| {
            dbg!(x);
        })?;
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
        VarInt(length as i32).write_stream(stream).await?;

        stream.write_all(buf).await?;
        Ok(length)
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
