use crate::types::long::Long;
use crate::types::package::ProtocolData;
use crate::types::string::McString;
use crate::types::var_int::VarInt;
use crate::types::var_long::VarLong;
use crate::types::{McRead, McWrite};
use crate::utils::RWStreamWithLimit;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct Data {
    pub timespan: Long,
}
impl McRead for Data {
    async fn read_stream<T: AsyncRead + Unpin>(stream: &mut T) -> Result<Self, String>
    where
        Self: Sized,
    {
        Ok(Self {
            timespan: Long::read_stream(stream).await?,
        })
    }
}

impl crate::types::package::ProtocolDataMarker for Data {}
#[derive(Debug, Clone)]
pub struct ResponseData {
    pub(crate) timespan: Long,
}
impl McWrite for ResponseData {
    type Error = String;

    async fn write_stream<T: AsyncWrite + Unpin>(
        &self,
        stream: &mut T,
    ) -> Result<usize, Self::Error> {
        self.timespan
            .write_stream(stream)
            .await
            .map_err(|e| e.to_string())
    }
}
