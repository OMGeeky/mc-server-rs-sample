use crate::types::long::Long;
use crate::types::package::{OutgoingPackage, OutgoingPackageContent, Package, ProtocolData};
use crate::types::string::McString;
use crate::types::var_int::VarInt;
use crate::types::var_long::VarLong;
use crate::types::{McRead, McWrite};
use crate::utils::RWStreamWithLimit;
use serde_json::json;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufWriter};

pub struct Protocol {}

#[derive(Debug, Clone)]
pub struct Data {}

impl McRead for Data {
    async fn read_stream<T: AsyncRead + Unpin>(stream: &mut T) -> Result<Self, String>
    where
        Self: Sized,
    {
        Ok(Self {})
    }
}
impl crate::types::package::ProtocolDataMarker for Data {}

#[derive(Debug, Clone)]
pub struct ResponseData {
    json_response: McString<32767>,
}
impl Default for ResponseData {
    fn default() -> Self {
        Self {
            json_response: McString::from_string(Protocol::get_sample_result()),
        }
    }
}
impl McWrite for ResponseData {
    type Error = String;

    async fn write_stream<T: AsyncWrite + Unpin>(
        &self,
        stream: &mut T,
    ) -> Result<usize, Self::Error> {
        self.json_response
            .write_stream(stream)
            .await
            .map_err(|e| e.to_string())
    }
}
impl Protocol {
    pub async fn handle<T: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut RWStreamWithLimit<'_, T>,
    ) -> Result<(), bool> {
        println!("Status");
        stream.discard_unread().await.map_err(|x| {
            dbg!(x);
            false
        })?;
        let string = Self::get_sample_result();
        let response = ResponseData {
            json_response: McString::from_string(string),
        };
        let x = Package::Outgoing(OutgoingPackage {
            protocol: crate::protocols::ProtocolResponseId::Status,
            content: OutgoingPackageContent::StatusResponse(response),
        });
        //
        // let mut total_size = 0;
        // let mut v = Vec::new();
        // let mut writer = BufWriter::new(&mut v);
        //
        // //Package ID
        // total_size += VarInt(0x00).write_stream(&mut writer).await.map_err(|x| {
        //     dbg!(x);
        //     false
        // })?;
        //
        // //Status JSON
        // total_size += McString::<32767>::from_string(string)
        //     .write_stream(&mut writer)
        //     .await
        //     .map_err(|x| {
        //         dbg!(x);
        //         false
        //     })?;
        // writer.flush().await.unwrap();
        //
        // println!("total size: {}: {:?}", total_size, &v);
        // //Size in front
        // VarInt(total_size as i32)
        //     .write_stream(stream)
        //     .await
        //     .map_err(|x| {
        //         dbg!(x);
        //         false
        //     })?;
        // //actually write the content to the stream, not just a local buffer
        // stream.write_all(&v).await.map_err(|x| {
        //     dbg!(x);
        //     false
        // })?;

        Ok(())
    }

    fn get_sample_result() -> String {
        json!({
            "version": {
                "name": "1.21.2",
                "protocol": 768
            },
            "players": {
                "max": 1000000000,
                "online": 5,
                "sample": [
                    {
                        "name": "thinkofdeath",
                        "id": "4566e69f-c907-48ee-8d71-d7ba5aa00d20",
                    },
                ],
            },
            "description": {
                "text": "Hello, world!"
            },
            // "favicon": "data:image/png;base64,<data>",
            "enforcesSecureChat": false,
        })
        .to_string()
    }
}
