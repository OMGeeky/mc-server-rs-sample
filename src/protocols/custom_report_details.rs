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
        let count = VarInt::read_stream(stream).await.map_err(|x| {
            dbg!(x);
            true
        })?;
        dbg!(&count);
        let string_size = VarInt::read_stream(stream).await.map_err(|x| {
            dbg!(x);
            true
        })?;
        dbg!(&string_size);
        stream.discard_unread().await.map_err(|x| {
            dbg!(x);
            true
        })?;
        // for i in 0..*count {
        //     let title = McString::read_stream(stream).await.map_err(|x| {
        //         dbg!(x);
        //     })?;
        //     let description = McString::read_stream(stream).await.map_err(|x| {
        //         dbg!(x);
        //     })?;
        //     println!(
        //         "Read title & description fo some custom report ({i}): {}\n{}",
        //         title.as_rs(),
        //         description.as_rs()
        //     );
        // }
        Ok(())
    }
}
