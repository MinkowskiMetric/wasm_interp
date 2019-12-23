use std::io;
use std::io::prelude::*;

use std::convert::TryFrom;

use crate::reader::{ReaderUtil, TypeReader, ScopedReader};
use crate::core;

impl TypeReader for Option<core::Section> {
    fn read<T: Read>(reader: &mut T) -> io::Result<Self> {
        // We allow the read of the type to fail with end of file because that is how we detect the end of
        // the file
        match reader.read_u8() {
            Ok(section_type) => {
                let section_length = reader.read_leb_u32()?;

                // Wrap the reader in a reader that prevents us from reading out of the section
                let mut section_reader = ScopedReader::new(reader, usize::try_from(section_length).unwrap());

                let ret = match section_type {
                    0 => {
                        let name = section_reader.read_name()?;
                        let body = section_reader.read_bytes_to_end()?;

                        core::Section::CustomSection { name, body }
                    },

                    1 => core::Section::TypeSection { section_body: section_reader.read_vec(core::FuncType::read)?, },
                    2 => core::Section::ImportSection { section_body: section_reader.read_vec(core::Import::read)? },
                    3 => core::Section::FunctionSection { section_body: section_reader.read_vec(ScopedReader::read_leb_u32)? },
                    4 => core::Section::TableSection { section_body: section_reader.read_vec(core::TableType::read)? },
                    5 => core::Section::MemorySection { section_body: section_reader.read_vec(core::MemType::read)? },
                    6 => core::Section::GlobalSection { section_body: section_reader.read_vec(core::Global::read)? },
                    7 => core::Section::ExportSection { section_body: section_reader.read_vec(core::Export::read)? },
                    8 => core::Section::StartSection { start_idx: section_reader.read_leb_u32()? },
                    9 => core::Section::ElementSection { elements: section_reader.read_vec(core::Element::read)? },
                    10 => core::Section::CodeSection { code: section_reader.read_vec(core::Func::read)? },
                    11 => core::Section::DataSection { data: section_reader.read_vec(core::Data::read)? },

                    _ => core::Section::UnknownSection { section_type, section_body: section_reader.read_bytes_to_end()? },
                };

                if section_reader.is_at_end() {
                    Ok(Some(ret))
                } else {
                    assert!(false, "Failed to read whole section");
                    Err(io::Error::new(io::ErrorKind::InvalidData, "Failed to read whole section"))
                }
            },

            Err(e) => match e.kind() {
                io::ErrorKind::UnexpectedEof => Ok(None),
                _ => Err(e),
            },
        }
        /*if self.sections_limit == 0 || self.sections_read < self.sections_limit {
            if let Some(file) = &mut self.src {
                if let Ok(section_type) = file.read_u8() {
                    if let Ok(section_length) = file.read_leb_u32() {
                        let mut section_reader = ScopedReader::new(file, usize::try_from(section_length).unwrap());

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
        None*/
    }
}