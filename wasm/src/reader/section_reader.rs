use std::io::{Read, Result};

use crate::core;
use crate::reader::{ReaderUtil, TypeReader};

impl TypeReader for core::SectionType {
    fn read<T: Read>(reader: &mut T) -> Result<Self> {
        Self::from_byte(reader.read_u8()?)
    }
}
