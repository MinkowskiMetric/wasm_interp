use std::io;
use std::io::prelude::*;

use crate::reader::{ReaderUtil, TypeReader};
use crate::core;

impl TypeReader for core::SectionType {
    fn read<T: Read>(reader: &mut T) -> io::Result<Self> {
        Self::from_byte(reader.read_u8()?)
    }
}
