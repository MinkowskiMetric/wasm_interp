use num_enum::TryFromPrimitive;
use std::convert::TryInto;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum SectionType {
    CustomSection,
    TypeSection,
    ImportSection,
    FunctionSection,
    TableSection,
    MemorySection,
    GlobalSection,
    ExportSection,
    StartSection,
    ElementSection,
    CodeSection,
    DataSection,
}

impl SectionType {
    pub fn from_byte(byte: u8) -> Result<Self> {
        let s: std::result::Result<SectionType, _> = byte.try_into();
        match s {
            Ok(s) => Ok(s),
            _ => Err(Error::new(ErrorKind::InvalidData, "Unknown section type")),
        }
    }
}
