use std::io;
use std::io::prelude::*;

pub struct ScopedReader<'a, I: io::Read> {
    src: &'a mut I,
    offset: usize,
    size: usize,
}

impl<'a, I> ScopedReader<'a, I>
where
    I: Read,
{
    pub fn new(src: &'a mut I, size: usize) -> Self {
        Self {
            src,
            offset: 0,
            size,
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.offset == self.size
    }
}

impl<'a, I> Read for ScopedReader<'a, I>
where
    I: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // We need to limit the size of the read
        let bytes_to_read = if buf.len() > (self.size - self.offset) {
            (self.size - self.offset)
        } else {
            buf.len()
        };

        if bytes_to_read > 0 {
            let bytes_read = self.src.read(&mut buf[0..bytes_to_read])?;
            self.offset += bytes_read;

            Ok(bytes_read)
        } else {
            Ok(0)
        }
    }
}
