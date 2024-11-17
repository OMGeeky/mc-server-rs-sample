use crate::types::long::Long;
use crate::types::string::McString;
use crate::types::var_int::VarInt;
use crate::types::McRead;
use crate::utils::RWStreamWithLimit;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, BufWriter};

#[derive(Debug, Clone)]
pub struct Data {
    details: Vec<(McString<128>, McString<4096>)>,
}
impl crate::types::package::ProtocolDataMarker for Data {}

impl McRead for Data {
    async fn read_stream<T: AsyncRead + Unpin>(stream: &mut T) -> Result<Self, String>
    where
        Self: Sized,
    {
        let count = VarInt::read_stream(stream).await?;
        let mut details = vec![];
        for i in 0..*count {
            let title = McString::read_stream(stream).await?;
            let description = McString::read_stream(stream).await?;
            details.push((title, description));
        }

        Ok(Self { details })
    }
}
