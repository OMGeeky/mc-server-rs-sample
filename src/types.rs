use tokio::io::{AsyncRead, AsyncWrite};

pub(crate) trait McRead {
    type Error;
    async fn read_stream<T: AsyncRead + Unpin>(stream: &mut T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
pub(crate) trait McWrite {
    type Error;
    async fn write_stream<T: AsyncWrite + Unpin>(
        &self,
        stream: &mut T,
    ) -> Result<usize, Self::Error>
    where
        Self: Sized;
}
pub trait McRustRepr {
    type RustRepresentation;
    fn into_rs(self) -> Self::RustRepresentation;
    fn to_rs(&self) -> Self::RustRepresentation;
    fn as_rs(&self) -> &Self::RustRepresentation;
}
pub mod string;
pub mod var_int;
