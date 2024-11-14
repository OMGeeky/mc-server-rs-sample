use crate::protocols::{self, ProtocolId, ProtocolResponseId};
use crate::types::string::McString;
use crate::types::var_int::VarInt;
use crate::types::{McRead, McWrite};
use crate::utils::RWStreamWithLimit;
use num_traits::ToPrimitive;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufWriter};

#[derive(Debug, Clone)]
pub enum Package {
    Incoming(IncomingPackage),
    Outgoing(OutgoingPackage),
}
#[derive(Debug, Clone)]
pub struct IncomingPackage {
    pub(crate) protocol: ProtocolId,
    pub(crate) content: IncomingPackageContent,
}
#[derive(Debug, Clone)]
pub struct OutgoingPackage {
    pub(crate) protocol: ProtocolResponseId,
    pub(crate) content: OutgoingPackageContent,
}
impl OutgoingPackage {
    pub fn empty() {}
}
#[derive(Debug, Clone)]
pub enum IncomingPackageContent {
    Handshake(crate::protocols::handshake::Data),
    Status(crate::protocols::status::Data),
    Ping(crate::protocols::ping::Data),
    CustomReportDetails(crate::protocols::custom_report_details::Data),
}
#[derive(Debug, Clone)]
pub enum OutgoingPackageContent {
    StatusResponse(crate::protocols::status::ResponseData),
    PingResponse(crate::protocols::ping::ResponseData),
}
impl McWrite for OutgoingPackageContent {
    type Error = String;

    async fn write_stream<T: AsyncWrite + Unpin>(
        &self,
        stream: &mut T,
    ) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        match self {
            OutgoingPackageContent::StatusResponse(x) => x.write_stream(stream).await,
            OutgoingPackageContent::PingResponse(x) => x.write_stream(stream).await,
        }
    }
}
impl McWrite for OutgoingPackage {
    type Error = String;

    async fn write_stream<T: AsyncWrite + Unpin>(
        &self,
        stream: &mut T,
    ) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        let id = self.protocol;
        let mut total_size = 0;

        //write the content to a local buffer first to determine size
        let mut v = Vec::new();
        println!("total size: {}: {:?}", total_size, &v);

        let mut writer = BufWriter::new(&mut v);
        total_size +=
            VarInt(ToPrimitive::to_i32(&id).expect("All the ids should be hard coded to work..."))
                .write_stream(&mut writer)
                .await
                .map_err(|e| e.to_string())?;
        writer.flush().await.map_err(|e| e.to_string())?;
        println!("total size: {}: {:?}", total_size, &v);

        let mut writer = BufWriter::new(&mut v);
        total_size += self.content.write_stream(&mut writer).await?;
        writer.flush().await.map_err(|e| e.to_string())?;
        println!("total size: {}: {:?}", total_size, &v);

        // //Size in front
        let x = VarInt(total_size as i32)
            .write_stream(stream)
            .await
            .map_err(|x| {
                dbg!(&x);
                format!("Error writing the size: {:?}", x).to_string()
            })?;
        // //actually write the content to the stream, not just a local buffer
        stream.write_all(&v).await.map_err(|x| {
            dbg!(&x);
            format!("Error writing the bytes: {:?}", x).to_string()
        })?;
        stream.flush().await.map_err(|e| e.to_string())?;
        Ok(total_size + x)
    }
}
impl IncomingPackage {
    async fn answer<T: AsyncRead + AsyncWrite + Unpin>(
        &self,
        stream: &mut RWStreamWithLimit<'_, T>,
    ) -> Result<(), bool> {
        let answer = match (&self.protocol, &self.content) {
            (ProtocolId::Status, _) => Some(OutgoingPackage {
                protocol: ProtocolResponseId::Status,
                content: OutgoingPackageContent::StatusResponse(
                    protocols::status::ResponseData::default(),
                ),
            }),
            (ProtocolId::Ping, IncomingPackageContent::Ping(ping_data)) => Some(OutgoingPackage {
                protocol: ProtocolResponseId::Ping,
                content: OutgoingPackageContent::PingResponse(protocols::ping::ResponseData {
                    timespan: ping_data.timespan,
                }),
            }),
            (ProtocolId::Ping, _) => unreachable!(),
            (ProtocolId::CustomReportDetails, _) => None,
        };
        if let Some(outgoing_package) = answer {
            outgoing_package.write_stream(stream).await.map_err(|e| {
                dbg!(e);
                false
            })?;
        }
        Ok(())
    }
}
impl Package {
    pub async fn handle<T: AsyncRead + AsyncWrite + Unpin>(
        protocol_id: ProtocolId,
        stream: &mut RWStreamWithLimit<'_, T>,
    ) -> Result<(), bool> {
        let incoming_content = read_data(protocol_id, stream).await.map_err(|e| {
            dbg!(e);
            true
        })?;
        let incoming = IncomingPackage {
            protocol: protocol_id,
            content: incoming_content,
        };
        incoming.answer(stream).await?;
        Ok(())
    }
}

pub async fn read_data<T: AsyncRead + AsyncWrite + Unpin>(
    protocol_id: ProtocolId,
    stream: &mut RWStreamWithLimit<'_, T>,
) -> Result<IncomingPackageContent, String> {
    Ok(match protocol_id {
        ProtocolId::Status => {
            IncomingPackageContent::Status(protocols::status::Data::read_stream(stream).await?)
        }
        ProtocolId::Ping => {
            IncomingPackageContent::Ping(protocols::ping::Data::read_stream(stream).await?)
        }
        ProtocolId::CustomReportDetails => {
            // return Err("Not implemented".to_string());
            let x = IncomingPackageContent::CustomReportDetails(
                protocols::custom_report_details::Data::read_stream(stream).await?,
            );
            stream.discard_unread().await.map_err(|e| {
                dbg!(e);
                "Could not discard unused stuff"
            })?;
            x
        }
    })
}
