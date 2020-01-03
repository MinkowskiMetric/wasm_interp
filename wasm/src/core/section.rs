use num_enum::TryFromPrimitive;
use std::io::{Error, ErrorKind, Read, Result};

use crate::reader::{ReaderUtil, TypeReader};

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

impl TypeReader for SectionType {
    fn read<T: Read>(reader: &mut T) -> Result<Self> {
        match Self::try_from_primitive(reader.read_u8()?) {
            Ok(s) => Ok(s),
            _ => Err(Error::new(ErrorKind::InvalidData, "Unknown section type")),
        }
    }
}
