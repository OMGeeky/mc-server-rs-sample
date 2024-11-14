use crate::types::string::McString;
use crate::types::var_int::VarInt;
use crate::types::var_long::VarLong;
use crate::types::McWrite;
use crate::utils::RWStreamWithLimit;
use serde_json::json;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufWriter};

pub struct Protocol {}

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
        let mut total_size = 0;
        let mut v = Vec::new();
        let mut writer = BufWriter::new(&mut v);

        //Package ID
        total_size += VarInt(0x00).write_stream(&mut writer).await.map_err(|x| {
            dbg!(x);
            false
        })?;

        //Status JSON
        total_size += McString::<32767>::from_string(string)
            .write_stream(&mut writer)
            .await
            .map_err(|x| {
                dbg!(x);
                false
            })?;
        writer.flush().await.unwrap();

        println!("total size: {}: {:?}", total_size, &v);
        //Size in front
        VarInt(total_size as i32)
            .write_stream(stream)
            .await
            .map_err(|x| {
                dbg!(x);
                false
            })?;
        //actually write the content to the stream, not just a local buffer
        stream.write_all(&v).await.map_err(|x| {
            dbg!(x);
            false
        })?;

        Ok(())
    }

    fn get_sample_result() -> String {
        json!({
            "version": {
                "name": "1.21.2",
                "protocol": 768
            },
            "players": {
                "max": 100,
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
