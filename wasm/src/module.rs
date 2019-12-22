use crate::reader_util::ReaderUtil;
use crate::section;

use std::convert::TryFrom;
use std::fs::File;
use std::io;

pub struct SectionPayloadReader<'a, I: io::Read> {
    src: &'a mut I,
    offset: usize,
    size: usize,
}

impl<'a, I> SectionPayloadReader<'a, I> where I: io::Read {
    pub fn new(src: &'a mut I, length: u32) -> Self {
        SectionPayloadReader { src, offset: 0, size: usize::try_from(length).unwrap() }
    }

    pub fn is_at_end(&self) -> bool {
        self.offset == self.size
    }
}

impl<'a, I> io::Read for SectionPayloadReader<'a, I> where I: io::Read {
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

struct SectionIterator<I> where I: io::Read {
    src: Option<I>,
    sections_read: u32,
    sections_limit: u32,
}

impl<I> Iterator for SectionIterator<I> where I: io::Read {
    type Item = section::Section;

    fn next(&mut self) -> Option<section::Section> {
        if self.sections_limit == 0 || self.sections_read < self.sections_limit {
            if let Some(file) = &mut self.src {
                if let Ok(section_type) = file.read_u8() {
                    if let Ok(section_length) = file.read_leb_u32() {
                        let mut section_reader = SectionPayloadReader::new(file, section_length);

                        match section::process_section(section_type, section_length, &mut section_reader) {
                            Ok(section) => {
                                assert!(section_reader.is_at_end(), "Failed to read whole section");
                                self.sections_read += 1;
                                return Some(section);
                            },
                            Err(e) => {
                                println!("Failed to read section {:?}", e);
                            }
                        }
                    }
                }
            }
        }

        // Failed to read for some reason
        self.src = None;
        None
    }
}

pub fn read<T: io::Read>(mut src: T) -> io::Result<impl Iterator<Item = section::Section>> {
    const HEADER_LENGTH: usize = 8;
    const EXPECTED_HEADER: [u8; 8] = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

    let mut header: [u8; HEADER_LENGTH] = [0; HEADER_LENGTH];

    // Read in the header
    src.read_exact(&mut header)?;

    if header != EXPECTED_HEADER {
        Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid module header"))
    } else {
        Ok(SectionIterator { src: Some(src), sections_read: 0, sections_limit: 0 })
    }
}

pub fn read_from_path(path: &str) -> io::Result<impl Iterator<Item = section::Section>> {
    let file = File::open(path)?;

    read(file)
}