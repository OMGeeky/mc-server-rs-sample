use crate::types::McRead;
use crate::utils::{MyAsyncReadExt, RWStreamWithLimit};
use crate::ConnectionState;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone)]
pub enum ProtocolResponseId {
    Status = 0x00,
    Ping = 0x01,
}
#[derive(Debug, Clone)]
pub struct NotImplementedData {}
impl McRead for NotImplementedData {
    async fn read_stream<T: AsyncRead + Unpin>(stream: &mut T) -> Result<Self, String>
    where
        Self: Sized,
    {
        Err("Did not implement this protocol yet".to_string())
    }
}

impl crate::types::package::ProtocolDataMarker for NotImplementedData {}
pub(crate) mod custom_report_details;
pub(crate) mod handshake;
pub(crate) mod ping;
pub(crate) mod status;
