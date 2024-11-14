use crate::utils::RWStreamWithLimit;
use num_derive::{FromPrimitive, ToPrimitive};
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone)]
pub enum ProtocolId {
    Status = 0x00,
    Ping = 0x01,
    CustomReportDetails = 0x7a,
}
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone)]
pub enum ProtocolResponseId {
    Status = 0x00,
    Ping = 0x01,
}
pub async fn handle<T: AsyncRead + AsyncWrite + Unpin>(
    protocol: ProtocolId,
    stream: &mut RWStreamWithLimit<'_, T>,
    // bytes_left_in_package: &mut i32,
) -> Result<(), bool> {
    match protocol {
        ProtocolId::Status => status::Protocol::handle(stream).await?,
        ProtocolId::Ping => ping::Protocol::handle(stream).await?,
        ProtocolId::CustomReportDetails => custom_report_details::Protocol::handle(stream).await?,
    };
    Ok(())
}
pub(crate) mod custom_report_details;
pub(crate) mod handshake;
pub(crate) mod ping;
pub(crate) mod status;
