use crate::types::long::Long;
use crate::types::string::McString;
use crate::types::var_int::VarInt;
use crate::types::McRead;
use crate::utils::RWStreamWithLimit;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, BufWriter};

pub struct Protocol {}

#[derive(Debug, Clone)]
pub struct Data {
    details: Vec<(McString<128>, McString<4096>)>,
}
impl crate::types::package::ProtocolDataMarker for Data {}

impl McRead for Data {
    async fn read_stream<T: AsyncRead + Unpin>(stream: &mut T) -> Result<Self, String>
    where
        Self: Sized,
    {
        // let size = stream.
        // let mut v = vec![0; size];
        // let mut writer = BufWriter::new(&mut v);

        // let x = stream.read_buf(&mut v)
        println!();
        loop {
            let byte = stream.read_u8().await.map_err(|e| {
                dbg!(e);
                "idk"
            })?;
            print!(" '{:0>2x}' ", byte);
            dbg!(byte);
        }
        // println!("x");
        // println!("{:?}", v);
        // dbg!(&v);
        // let count = VarInt::read_stream(stream).await?;
        let details = vec![];
        // let string = format!("Still need to get details from stream ({})", *count);
        // dbg!(string);
        // for i in 0..*count {
        //     let title = McString::<128>::read_stream(stream).await?;
        //     let description = McString::<128>::read_stream(stream).await?;
        // }

        Ok(Self { details })
    }
}
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
            // McString::<128>::read_stream(stream).await.map_err(|x| {
            //     dbg!(x);
            //     true
            // })?;
            // McString::<4096>::read_stream(stream).await.map_err(|x| {
            //     dbg!(x);
            //     true
            // })?;
        }
        Err(true)
    }
}
