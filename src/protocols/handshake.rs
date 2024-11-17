use crate::types::long::Long;
use crate::types::string::McString;
use crate::types::var_int::{read_stream, VarInt};
use crate::types::var_long::VarLong;
use crate::types::{McRead, McRustRepr, McWrite};
use crate::utils::RWStreamWithLimit;
use crate::ConnectionState;
use num_traits::FromPrimitive;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct Data {
    protocol_version: VarInt,
    server_address: McString<255>,
    server_port: u16,
    pub(crate) next_state: ConnectionState,
}
impl crate::types::package::ProtocolDataMarker for Data {}

impl McRead for Data {
    async fn read_stream<T: AsyncRead + Unpin>(b: &mut T) -> Result<Self, String> {
        println!("Reading Handshake");
        let protocol_version = VarInt::read_stream(b).await?;
        let server_address = McString::read_stream(b).await?;
        let server_port = b
            .read_u16()
            .await
            .map_err(|e| format!("Error reading port:{e}"))?;

        let next_state_id = VarInt::read_stream(b).await?;
        println!("next state: {}", next_state_id.as_rs());
        let next_state = FromPrimitive::from_i32(next_state_id.to_rs());
        let next_state = match next_state {
            Some(next_state) => next_state,
            None => {
                return Err(format!(
                    "Got an unknown next state: {}",
                    next_state_id.as_rs()
                ))
            }
        };
        Ok(Self {
            server_port,
            server_address,
            next_state,
            protocol_version,
        })
    }
}
