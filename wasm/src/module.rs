use crate::reader::TypeReader;
use crate::core;

use std::fs::File;
use std::io::{self, BufReader};

struct SectionIterator<I> where I: io::Read {
    src: I,
}

impl<I> Iterator for SectionIterator<I> where I: io::Read {
    type Item = io::Result<core::Section>;

    fn next(&mut self) -> Option<Self::Item> {
        match Option::<core::Section>::read(&mut self.src) {
            Ok(Some(s)) => Some(Ok(s)),
            Ok(None) => None,

            Err(e) => Some(Err(e)),
        }
    }
}

pub fn read<T: io::Read>(mut src: T) -> io::Result<impl Iterator<Item = io::Result<core::Section>>> {
    const HEADER_LENGTH: usize = 8;
    const EXPECTED_HEADER: [u8; 8] = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

    let mut header: [u8; HEADER_LENGTH] = [0; HEADER_LENGTH];

    // Read in the header
    src.read_exact(&mut header)?;

    if header != EXPECTED_HEADER {
        Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid module header"))
    } else {
        Ok(SectionIterator { src })
    }
}

pub fn read_from_path(path: &str) -> io::Result<impl Iterator<Item = io::Result<core::Section>>> {
    let file = File::open(path)?;
    let file = BufReader::new(file);

    read(file)
}