use std::io;
use std::io::prelude::*;

use std::convert::TryFrom;

use crate::core;
use crate::reader::{ReaderUtil, ScopedReader, TypeReader};

fn append_to_vector<R>(target: &mut Vec<R>, mut extra: Vec<R>) {
    target.append(&mut extra);
}

#[derive(Debug)]
struct ModuleBuilder {
    types: Vec<core::FuncType>,
    typeidx: Vec<usize>,
    funcs: Vec<core::Func>,
    tables: Vec<core::TableType>,
    mems: Vec<core::MemType>,
    globals: Vec<core::GlobalDef>,
    elem: Vec<core::Element>,
    data: Vec<core::Data>,
    start: Option<u32>,
    imports: Vec<core::Import>,
    exports: Vec<core::Export>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        ModuleBuilder {
            types: Vec::new(),
            typeidx: Vec::new(),
            funcs: Vec::new(),
            tables: Vec::new(),
            mems: Vec::new(),
            globals: Vec::new(),
            elem: Vec::new(),
            data: Vec::new(),
            start: None,
            imports: Vec::new(),
            exports: Vec::new(),
        }
    }

    pub fn process_section<T: Read>(
        &mut self,
        section_type: core::SectionType,
        reader: &mut T,
    ) -> io::Result<()> {
        match section_type {
            core::SectionType::TypeSection => Ok(append_to_vector(
                &mut self.types,
                reader.read_vec(core::FuncType::read)?,
            )),
            core::SectionType::ImportSection => Ok(append_to_vector(
                &mut self.imports,
                reader.read_vec(core::Import::read)?,
            )),
            core::SectionType::FunctionSection => Ok(append_to_vector(
                &mut self.typeidx,
                reader.read_vec(T::read_leb_usize)?,
            )),
            core::SectionType::TableSection => Ok(append_to_vector(
                &mut self.tables,
                reader.read_vec(core::TableType::read)?,
            )),
            core::SectionType::MemorySection => Ok(append_to_vector(
                &mut self.mems,
                reader.read_vec(core::MemType::read)?,
            )),
            core::SectionType::GlobalSection => Ok(append_to_vector(
                &mut self.globals,
                reader.read_vec(core::GlobalDef::read)?,
            )),
            core::SectionType::ExportSection => Ok(append_to_vector(
                &mut self.exports,
                reader.read_vec(core::Export::read)?,
            )),
            core::SectionType::StartSection => self.update_start(reader.read_leb_u32()?),
            core::SectionType::ElementSection => Ok(append_to_vector(
                &mut self.elem,
                reader.read_vec(core::Element::read)?,
            )),
            core::SectionType::CodeSection => Ok(append_to_vector(
                &mut self.funcs,
                reader.read_vec(core::Func::read)?,
            )),
            core::SectionType::DataSection => Ok(append_to_vector(
                &mut self.data,
                reader.read_vec(core::Data::read)?,
            )),

            _ => panic!("Cannot read unknown or custom sections"),
        }
    }

    pub fn get_next_section_type(
        current_section_type: core::SectionType,
    ) -> Option<core::SectionType> {
        match current_section_type {
            core::SectionType::TypeSection => Some(core::SectionType::ImportSection),
            core::SectionType::ImportSection => Some(core::SectionType::FunctionSection),
            core::SectionType::FunctionSection => Some(core::SectionType::TableSection),
            core::SectionType::TableSection => Some(core::SectionType::MemorySection),
            core::SectionType::MemorySection => Some(core::SectionType::GlobalSection),
            core::SectionType::GlobalSection => Some(core::SectionType::ExportSection),
            core::SectionType::ExportSection => Some(core::SectionType::StartSection),
            core::SectionType::StartSection => Some(core::SectionType::ElementSection),
            core::SectionType::ElementSection => Some(core::SectionType::CodeSection),
            core::SectionType::CodeSection => Some(core::SectionType::DataSection),
            core::SectionType::DataSection => None,

            _ => panic!("Cannot read unknown or custom sections"),
        }
    }

    pub fn make_module(self) -> io::Result<core::RawModule> {
        if self.typeidx.len() == 0 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "No functions found",
            ))
        } else if self.typeidx.len() != self.funcs.len() {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "TypeIdx and code tables do not match sizes",
            ))
        } else {
            // TODOTODOTODO - this will get more complicated - there is more processing to be done here
            // to tie up the functions table
            Ok(core::RawModule::new(
                self.types,
                self.typeidx,
                self.funcs,
                self.tables,
                self.mems,
                self.globals,
                self.elem,
                self.data,
                self.start,
                self.imports,
                self.exports,
            ))
        }
    }

    fn update_start(&mut self, new_start: u32) -> io::Result<()> {
        if let Some(_) = self.start {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Multiple start sections found",
            ))
        } else {
            self.start = Some(new_start);
            Ok(())
        }
    }
}

fn read_next_section_header<T: Read>(reader: &mut T) -> io::Result<Option<core::SectionType>> {
    match core::SectionType::read(reader) {
        Err(e) => {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                // Failing to read a section at the end of the file is expected because that is how we detect the end
                // of the file
                Ok(None)
            } else {
                Err(e)
            }
        }
        Ok(s) => Ok(Some(s)),
    }
}

impl TypeReader for core::RawModule {
    fn read<T: Read>(reader: &mut T) -> io::Result<Self> {
        const HEADER_LENGTH: usize = 8;
        const EXPECTED_HEADER: [u8; 8] = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

        let mut header: [u8; HEADER_LENGTH] = [0; HEADER_LENGTH];

        // Read in the header
        reader.read_exact(&mut header)?;

        if header != EXPECTED_HEADER {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid module header",
            ))
        } else {
            let mut current_section_type: Option<core::SectionType> =
                Some(core::SectionType::TypeSection);
            let mut module_builder = ModuleBuilder::new();

            loop {
                if let Some(section_type) = read_next_section_header(reader)? {
                    // Read the section length
                    let section_length = usize::try_from(reader.read_leb_u32()?).unwrap();
                    // And make a scoped reader for the section
                    let mut section_reader = ScopedReader::new(reader, section_length);

                    // Always skip custom sections wherever they appear
                    if section_type == core::SectionType::CustomSection {
                        // Read the section name
                        let section_name = section_reader.read_name()?;
                        let _section_body = section_reader.read_bytes_to_end()?;

                        println!("Skipping custom section \"{}\"", section_name);
                    } else {
                        while let Some(expected_section_type) = current_section_type {
                            if expected_section_type == section_type {
                                // This is the correct section type so we process it and move on
                                module_builder
                                    .process_section(section_type, &mut section_reader)?;

                                // And the next section type is the same as this one
                                current_section_type = Some(expected_section_type);
                                break;
                            } else {
                                // The section type doesn't match, so we move on to see if it
                                // is the next valid section
                                current_section_type =
                                    ModuleBuilder::get_next_section_type(expected_section_type);
                            }
                        }

                        if current_section_type == None {
                            assert!(false, "Sections are in unexpected order");
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid section order",
                            ));
                        }
                    }

                    if !section_reader.is_at_end() {
                        assert!(false, "Failed to read whole section");
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Failed to read whole section",
                        ));
                    }
                } else {
                    // End of file, so we can break out of the loop
                    break;
                }
            }

            module_builder.make_module()
        }
    }
}
