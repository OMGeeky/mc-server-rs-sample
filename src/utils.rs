use std::io::ErrorKind;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};

pub struct RWStreamWithLimit<'a, T: AsyncRead + AsyncWrite> {
    stream: &'a mut T,
    read_bytes_left: usize,
}

impl<'a, T: AsyncRead + AsyncWrite + Unpin> RWStreamWithLimit<'a, T> {
    pub(crate) fn new(stream: &'a mut T, read_limit: usize) -> Self {
        Self {
            stream,
            read_bytes_left: read_limit,
        }
    }
    pub(crate) async fn discard_unread(&mut self) -> std::io::Result<usize> {
        let mut total_read = 0;
        while self.read_bytes_left > 0 {
            println!("Discarding {} bytes...", self.read_bytes_left);
            let read = self.stream.read(&mut vec![0; self.read_bytes_left]).await?;
            total_read += read;
            self.read_bytes_left -= read;
            println!(
                "Discarded {read}/{} remaining bytes ({total_read}/{} total)",
                self.read_bytes_left + read,
                self.read_bytes_left + total_read
            );
            if read == 0 {
                const ERROR: &str = "Could not read a single byte";
                println!("{}", ERROR);
                return Err(std::io::Error::new(ErrorKind::Other, ERROR));
            }
            if self.read_bytes_left > 0 {
                //IDK if this makes sense to just throw an error if we don't read all in one go?
                const ERROR: &str = "Couldnt read all bytes in one go";
                println!("{}", ERROR);
                return Err(std::io::Error::new(ErrorKind::Other, ERROR));
            }
            println!("Done Discarding {total_read} bytes");
        }
        Ok(total_read)
    }
    pub fn get_read_left(&self) -> usize {
        self.read_bytes_left
    }
}
impl<'a, T: AsyncRead + AsyncWrite + Unpin> AsyncRead for RWStreamWithLimit<'a, T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.read_bytes_left == 0 {
            return Poll::Ready(Err(std::io::Error::new(
                ErrorKind::Other,
                "There is nothing more to read in this package",
            )));
        }

        let self_mut = self.get_mut();
        let stream = &mut self_mut.stream;

        if self_mut.read_bytes_left < buf.remaining() {
            println!("wants to read more than in the readable part of the stream. Only read readable part, to not screw up the next few parts");
        }
        let bytes_to_read = std::cmp::min(self_mut.read_bytes_left, buf.remaining());
        let mut inner_buf = buf.take(bytes_to_read);

        let read = Pin::new(stream).poll_read(cx, &mut inner_buf);
        if let Poll::Ready(Ok(())) = read {
            let bytes_read = inner_buf.filled().len();
            self_mut.read_bytes_left -= bytes_read;
            buf.advance(bytes_read); // Important: Advance the buffer
            Poll::Ready(Ok(()))
        } else {
            read
        }
    }
}
impl<'a, T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for RWStreamWithLimit<'a, T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let self_mut = self.get_mut();
        self_mut.read_bytes_left = 0;

        let stream = &mut self_mut.stream;
        Pin::new(stream).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let stream = &mut self.get_mut().stream;
        Pin::new(stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let stream = &mut self.get_mut().stream;
        Pin::new(stream).poll_shutdown(cx)
    }
}
