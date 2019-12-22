use std::io;

use crate::reader_util::ReaderUtil;

#[derive(Debug)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

impl ValueType {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<ValueType> {
        Self::from_byte(reader.read_u8()?)
    }

    pub fn from_byte(byte: u8) -> io::Result<ValueType> {
        match byte {
            0x7F => Ok(ValueType::I32),
            0x7E => Ok(ValueType::I64),
            0x7D => Ok(ValueType::F32),
            0x7C => Ok(ValueType::F64),

            b => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid value type byte 0x{:02x}", b))),
        }
    }
}

#[derive(Debug)]
pub struct FuncType {
    arg_types: Vec<ValueType>,
    ret_types: Vec<ValueType>,
}

impl FuncType {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let header = reader.read_u8()?;
        if header != 0x60 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid func type header"));
        }

        let arg_types = reader.read_vec(ValueType::read)?;
        let ret_types = reader.read_vec(ValueType::read)?;

        Ok(FuncType { arg_types, ret_types })
    }
}

#[derive(Debug)]
pub struct TypeSectionData {
    func_types: Vec<FuncType>,
}

impl TypeSectionData {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let func_types = reader.read_vec(FuncType::read)?;

        Ok(TypeSectionData { func_types })
    }
}