use std::convert::TryFrom;
use std::io;

pub trait ReaderUtil {
    fn read_u8(&mut self) -> std::result::Result<u8, std::io::Error>;
    fn read_leb_u32(&mut self) -> std::result::Result<u32, std::io::Error>;
    fn read_vec<R, T: Fn(&mut Self) -> std::io::Result<R>>(&mut self, read_fn: T) -> std::io::Result<Vec<R>>;
    fn read_name(&mut self) -> std::io::Result<String>;
}

impl<T> ReaderUtil for T where T: io::Read {
    fn read_u8(&mut self) -> std::result::Result<u8, std::io::Error> {
        let mut buf: [u8; 1] = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_leb_u32(&mut self) -> std::result::Result<u32, std::io::Error> {
        let mut result: u32 = 0;
        let mut shift = 0;

        loop {
            let byte = self.read_u8()?;
            result |= u32::from(byte & 0x7f) << shift;
            if (byte & 0x80) == 0 {
                return Ok(result);
            }
            shift += 7;
        }
    }

    fn read_vec<R, T2: Fn(&mut Self) -> std::io::Result<R>>(&mut self, read_fn: T2) -> std::io::Result<Vec<R>> {
        let vector_length = self.read_leb_u32()?;
        let mut ret = Vec::with_capacity(usize::try_from(vector_length).unwrap());

        for _ in 0..vector_length {
            ret.push(read_fn(self)?);
        }

        Ok(ret)
    }

    fn read_name(&mut self) -> std::io::Result<String> {
        let bytes = self.read_vec(Self::read_u8)?;

        match String::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF8 in name")),
        }
    }
}