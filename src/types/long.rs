use crate::types::var_int::{read_stream, VarInt};
use crate::types::{McRead, McRustRepr, McWrite};
use std::ops::Deref;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Copy, Clone)]
pub struct Long(pub i64);
impl Deref for Long {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl McWrite for Long {
    type Error = std::io::Error;

    async fn write_stream<T: AsyncWrite + Unpin>(
        &self,
        stream: &mut T,
    ) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        stream.write_i64(self.0).await?;
        Ok(8)
    }
}
impl McRead for Long {
    async fn read_stream<T: AsyncRead + Unpin>(b: &mut T) -> Result<Self, String> {
        Ok(Self(b.read_i64().await.map_err(|e| e.to_string())?))
    }
}
impl McRustRepr for Long {
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
