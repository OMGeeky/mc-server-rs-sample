use crate::utils::RWStreamWithLimit;
use num_derive::FromPrimitive;
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(FromPrimitive)]
pub enum Protocol {
    Status = 0x00,
    Ping = 0x01,
    CustomReportDetails = 0x7a,
}
pub async fn handle<T: AsyncRead + AsyncWrite + Unpin>(
    protocol: Protocol,
    stream: &mut RWStreamWithLimit<'_, T>,
    // bytes_left_in_package: &mut i32,
) -> Result<(), bool> {
    match protocol {
        Protocol::Status => status::Protocol::handle(stream).await?,
        Protocol::Ping => ping::Protocol::handle(stream).await?,
        Protocol::CustomReportDetails => custom_report_details::Protocol::handle(stream).await?,
    };
    Ok(())
}
mod custom_report_details;
mod ping;
mod status;
