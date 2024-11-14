use crate::types::var_int::VarInt;
use crate::types::var_long::VarLong;
use crate::types::{McRead, McWrite};
use crate::utils::RWStreamWithLimit;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct Protocol {}

impl Protocol {
    pub async fn handle<T: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut RWStreamWithLimit<'_, T>,
    ) -> Result<(), bool> {
        println!("Ping");
        let v = stream.read_i64().await.map_err(|e| {
            dbg!(e);
            false
        })?;
        VarInt(0x01).write_stream(stream).await.map_err(|x| {
            dbg!(x);
            false
        })?;
        stream.write_i64(v).await.map_err(|e| {
            dbg!(e);
            false
        })?;

        Ok(())
    }
}
