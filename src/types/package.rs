use crate::protocols::{self, NotImplementedData, ProtocolResponseId};
use crate::types::string::McString;
use crate::types::var_int::VarInt;
use crate::types::{McRead, McRustRepr, McWrite};
use crate::utils::RWStreamWithLimit;
use crate::{types, ConnectionState};
use mc_rust_server_macros::McProtocol;
use num_traits::ToPrimitive;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufWriter};

#[derive(Debug, Clone)]
pub enum Package {
    Incoming(IncomingPackage),
    Outgoing(OutgoingPackage),
}
#[derive(Debug, Clone)]
pub struct IncomingPackage {
    pub(crate) content: IncomingPackageContent,
}
#[derive(Debug, Clone)]
pub struct OutgoingPackage {
    pub(crate) protocol: ProtocolResponseId,
    pub(crate) content: OutgoingPackageContent,
}
#[derive(Debug, Clone, McProtocol)]
pub enum IncomingPackageContent {
    #[protocol_read(ConnectionState::NotConnected, 0x00)]
    Handshake(protocols::handshake::Data),
    #[protocol_read(ConnectionState::Status, 0x00)]
    Status(protocols::status::Data),
    #[protocol_read(ConnectionState::Status, 0x01)]
    Ping(protocols::ping::Data),
    #[protocol_read(ConnectionState::Play, 0x7a)]
    CustomReportDetails(protocols::custom_report_details::Data),
    #[protocol_read(ConnectionState::Login, 0x00)]
    LoginStart(NotImplementedData),
    #[protocol_read(ConnectionState::Login, 0x01)]
    EncryptionResponse(NotImplementedData),
    #[protocol_read(ConnectionState::Login, 0x02)]
    LoginPluginResponse(NotImplementedData),
    #[protocol_read(ConnectionState::Login, 0x03)]
    CookieResponse(NotImplementedData),
}
#[derive(Debug, Clone)]
pub enum OutgoingPackageContent {
    StatusResponse(protocols::status::ResponseData),
    PingResponse(protocols::ping::ResponseData),
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
    pub(crate) async fn handle_incoming<T: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut RWStreamWithLimit<'_, T>,
        connection_state: ConnectionState,
    ) -> Result<Option<ConnectionState>, String> {
        let packet_id = VarInt::read_stream(stream)
            .await
            .map_err(|e| e.to_string())?;

        println!(
            "Handling new Package with id: {:0>2x} =======================",
            packet_id.as_rs()
        );

        let incoming_content =
            IncomingPackageContent::read_protocol_data(packet_id, connection_state, stream).await;

        let incoming = IncomingPackage {
            content: match incoming_content {
                Ok(incoming) => incoming,
                Err(e) => {
                    stream.discard_unread().await.map_err(|x| x.to_string())?;
                    return Err(e);
                }
            },
        };
        let res = incoming.answer(stream).await;
        match res {
            Ok(connection_state_change) => {
                println!("Success!");
                return Ok(connection_state_change);
            }
            Err(terminate_connection) => {
                if terminate_connection {
                    return Err("Something terrible has happened!".to_string());
                } else {
                    stream.discard_unread().await.map_err(|x| x.to_string())?;
                }
                println!("Failure :(");
            }
        }

        Ok(None)
    }
    async fn answer<T: AsyncRead + AsyncWrite + Unpin>(
        &self,
        stream: &mut RWStreamWithLimit<'_, T>,
    ) -> Result<Option<ConnectionState>, bool> {
        let (answer, changed_connection_state) = match &self.content {
            (IncomingPackageContent::Handshake(handshake_data)) => {
                (None, Some(handshake_data.next_state))
            }
            IncomingPackageContent::Status(_) => (
                Some(OutgoingPackage {
                    protocol: ProtocolResponseId::Status,
                    content: OutgoingPackageContent::StatusResponse(
                        protocols::status::ResponseData::default(),
                    ),
                }),
                None,
            ),
            (IncomingPackageContent::Ping(ping_data)) => (
                Some(OutgoingPackage {
                    protocol: ProtocolResponseId::Ping,
                    content: OutgoingPackageContent::PingResponse(protocols::ping::ResponseData {
                        timespan: ping_data.timespan,
                    }),
                }),
                None,
            ),
            _ => (None, None), //TODO: implement the other IncomingPackageContent variants
        };
        if let Some(outgoing_package) = answer {
            outgoing_package.write_stream(stream).await.map_err(|e| {
                dbg!(e);
                false
            })?;
        }
        Ok(changed_connection_state)
    }
}

pub(crate) trait ProtocolDataMarker {}
pub(crate) trait ProtocolData {
    async fn read_data<T: AsyncRead + Unpin>(stream: &mut T) -> Result<Self, String>
    where
        Self: Sized;
}
impl<T> ProtocolData for T
where
    T: ProtocolDataMarker + McRead,
{
    async fn read_data<Stream: AsyncRead + Unpin>(stream: &mut Stream) -> Result<Self, String>
    where
        Self: Sized,
    {
        Self::read_stream(stream).await
    }
}
async fn read_protocol_data<S, T: AsyncRead + Unpin>(stream: &mut T) -> Result<S, String>
where
    S: Sized + ProtocolData,
{
    S::read_data::<T>(stream).await
}
