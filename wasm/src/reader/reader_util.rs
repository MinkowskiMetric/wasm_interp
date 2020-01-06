use anyhow::{anyhow, Result};
use std::convert::TryFrom;
use std::io;

pub trait ReaderUtil {
    fn read_u8(&mut self) -> Result<u8>;
    fn read_leb_u32(&mut self) -> Result<u32>;
    fn read_leb_usize(&mut self) -> Result<usize>;

    fn read_vec<R, T: Fn(&mut Self) -> Result<R>>(&mut self, read_fn: T) -> Result<Vec<R>>;

    fn read_name(&mut self) -> Result<String>;
    fn read_bytes_to_end(&mut self) -> Result<Vec<u8>>;
}

impl<T> ReaderUtil for T
where
    T: io::Read,
{
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf: [u8; 1] = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_leb_u32(&mut self) -> Result<u32> {
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

    fn read_leb_usize(&mut self) -> Result<usize> {
        Ok(usize::try_from(self.read_leb_u32()?).unwrap())
    }

    fn read_vec<R, T2: Fn(&mut Self) -> Result<R>>(&mut self, read_fn: T2) -> Result<Vec<R>> {
        let vector_length = self.read_leb_u32()?;
        let mut ret = Vec::with_capacity(usize::try_from(vector_length).unwrap());

        for _ in 0..vector_length {
            ret.push(read_fn(self)?);
        }

        Ok(ret)
    }

    fn read_name(&mut self) -> Result<String> {
        let bytes = self.read_vec(Self::read_u8)?;

        match String::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(_) => Err(anyhow!("Invalid UTF8 in name")),
        }
    }

    fn read_bytes_to_end(&mut self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        self.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}
