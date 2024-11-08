use std::io::{ErrorKind, Read, Write};

pub struct RWStreamWithLimit<'a, T: Read + Write> {
    stream: &'a mut T,
    read_bytes_left: usize,
}
impl<'a, T: Read + Write> RWStreamWithLimit<'a, T> {
    pub(crate) fn new(stream: &'a mut T, read_limit: usize) -> Self {
        Self {
            stream,
            read_bytes_left: read_limit,
        }
    }
    pub(crate) fn discard_unread(&mut self) -> std::io::Result<usize> {
        let mut total_read = 0;
        while self.read_bytes_left > 0 {
            let read = self.stream.read(&mut vec![0; self.read_bytes_left])?;
            total_read += read;
            self.read_bytes_left -= read;
        }
        Ok(total_read)
    }
    pub fn get_read_left(&self) -> usize {
        self.read_bytes_left
    }
}
impl<'a, T: Read + Write> Read for RWStreamWithLimit<'a, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read;
        if self.read_bytes_left > 0 {
            if self.read_bytes_left >= buf.len() {
                bytes_read = self.stream.read(buf)?;
            } else {
                println!("wants to read more than in the readable part of the stream");
                bytes_read = self.stream.read(&mut buf[0..self.read_bytes_left])?;
                //TODO: decide if we wanna throw an error here or nah
            }
            self.read_bytes_left -= bytes_read; //TODO: maybe check if we read to much?
        } else {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "There is nothing more to read in this package",
            ));
            //TODO: maybe throw an error since there is no way anything gets read anymore?
        }
        Ok(bytes_read)
    }
}
impl<'a, T: Read + Write> Write for RWStreamWithLimit<'a, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.read_bytes_left = 0;
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}
