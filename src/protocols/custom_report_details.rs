use crate::types::string::McString;
use crate::types::var_int::VarInt;
use crate::types::McRead;
use crate::utils::RWStreamWithLimit;
use tokio::io::{AsyncRead, AsyncWrite};

pub struct Protocol {}

impl Protocol {
    pub async fn handle<T: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut RWStreamWithLimit<'_, T>,
        // bytes_left_in_package: &mut i32,
    ) -> Result<(), bool> {
        println!("Some custom report detail stuff...");
        let count = VarInt::read_stream(stream).await.map_err(|x| {
            dbg!(x);
            true
        })?;
        dbg!(&count);
        for i in 0..*count {
            McString::<128>::read_stream(stream).await.map_err(|x| {
                dbg!(x);
                true
            })?;
            McString::<4096>::read_stream(stream).await.map_err(|x| {
                dbg!(x);
                true
            })?;
        }
        Err(true)
    }
}
