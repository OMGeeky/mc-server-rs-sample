use crate::utils::RWStreamWithLimit;
use crate::ConnectionState;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone)]
pub enum NotConnectedProtocolIds {
    Handshake = 0x00,
}
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone)]
pub enum StatusProtocolIds {
    Status = 0x00,
    Ping = 0x01,
}
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone)]
pub enum LoginProtocolIds {
    LoginStart = 0x00,
    EncryptionResponse = 0x01,
    LoginPluginResponse = 0x02,
    CookieResponse = 0x04,
}
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone)]
pub enum TransferProtocolIds {}
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone)]
pub enum ConfigurationProtocolIds {}
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone)]
pub enum PlayProtocolIds {}
#[derive(Debug, Copy, Clone)]
pub enum ProtocolId {
    NotConnected(NotConnectedProtocolIds),
    Status(StatusProtocolIds),
    Login(LoginProtocolIds),
    Transfer(TransferProtocolIds),
    Configuration(ConfigurationProtocolIds),
    Play(PlayProtocolIds),
    // CustomReportDetails = 0x7a,
}
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone)]
pub enum ProtocolResponseId {
    Status = 0x00,
    Ping = 0x01,
}
impl ProtocolId {
    pub(crate) fn from_id_and_state(id: i32, state: ConnectionState) -> Option<Self> {
        Some(match state {
            ConnectionState::NotConnected => Self::NotConnected(FromPrimitive::from_i32(id)?),
            ConnectionState::Status => Self::Status(FromPrimitive::from_i32(id)?),
            ConnectionState::Login => Self::Login(FromPrimitive::from_i32(id)?),
            ConnectionState::Transfer => Self::Transfer(FromPrimitive::from_i32(id)?),
            ConnectionState::Configuration => Self::Configuration(FromPrimitive::from_i32(id)?),
            ConnectionState::Play => Self::Play(FromPrimitive::from_i32(id)?),
            ConnectionState::Closed => return None,
        })
    }
}
pub(crate) mod custom_report_details;
pub(crate) mod handshake;
pub(crate) mod ping;
pub(crate) mod status;
