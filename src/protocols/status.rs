use crate::types::string::McString;
use crate::types::var_int::VarInt;
use crate::types::McWrite;
use crate::utils::RWStreamWithLimit;
use tokio::io::{AsyncRead, AsyncWrite};

pub struct Protocol {}

impl Protocol {
    pub async fn handle<T: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut RWStreamWithLimit<'_, T>,
        // bytes_left_in_package: &mut i32,
    ) -> Result<(), bool> {
        println!("Status");
        VarInt(0x01).write_stream(stream).await.map_err(|x| {
            dbg!(x);
            false
        })?;

        McString::<32767>::from_string(Self::get_sample_result())
            .write_stream(stream)
            .await
            .map_err(|x| {
                dbg!(x);
                false
            })?;
        // stream.discard_unread().await.map_err(|x| {
        //     dbg!(x);
        //     false
        // })?;
        // *bytes_left_in_package = 0;
        Ok(())
    }
    fn get_sample_result() -> String {
        "{
            \"version\": {
                \"name\": \"1.21.2\",
                \"protocol\": 768
            },
            \"players\": {
                \"max\": 100,
                \"online\": 5,
                \"sample\": [
                    {
                        \"name\": \"thinkofdeath\",
                        \"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"
                    }
                ]
            },
            \"description\": {
                \"text\": \"Hello, world!\"
            },
            \"favicon\": \"data:image/png;base64,<data>\",
            \"enforcesSecureChat\": false
        }"
        .to_string()
    }
}
