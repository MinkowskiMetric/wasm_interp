use std::io;
use std::io::prelude::*;

use std::convert::TryFrom;

use crate::core;
use crate::parser;
use crate::reader::{ReaderUtil, ScopedReader};

pub trait TypeReader
where
    Self: std::marker::Sized,
{
    fn read<T: Read>(reader: &mut T) -> io::Result<Self>;
}

impl TypeReader for core::ValueType {
    fn read<T: Read>(reader: &mut T) -> io::Result<Self> {
        Self::from_byte(reader.read_u8()?)
    }
}

impl TypeReader for core::MutableType {
    fn read<T: Read>(reader: &mut T) -> io::Result<Self> {
        Self::from_byte(reader.read_u8()?)
    }
}

impl TypeReader for core::ElemType {
    fn read<T: Read>(reader: &mut T) -> io::Result<Self> {
        Self::from_byte(reader.read_u8()?)
    }
}

impl TypeReader for core::Limits {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        match reader.read_u8()? {
            0x00 => Ok(core::Limits::Unbounded(reader.read_leb_u32()?)),
            0x01 => {
                let min = reader.read_leb_u32()?;
                let max = reader.read_leb_u32()?;

                Ok(core::Limits::Bounded(min, max))
            }

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown Limits tag",
            )),
        }
    }
}

impl TypeReader for core::TableType {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let et = core::ElemType::read(reader)?;
        let lim = core::Limits::read(reader)?;

        Ok(Self::new(et, lim))
    }
}

impl TypeReader for core::MemType {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        Ok(Self::new(core::Limits::read(reader)?))
    }
}

impl TypeReader for core::GlobalType {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let t = core::ValueType::read(reader)?;
        let m = core::MutableType::read(reader)?;

        Ok(Self::new(t, m))
    }
}

impl TypeReader for core::FuncType {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let header = reader.read_u8()?;
        if header != 0x60 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid func type header",
            ));
        }

        let arg_types = reader.read_vec(core::ValueType::read)?;
        let ret_types = reader.read_vec(core::ValueType::read)?;

        Ok(core::FuncType::new(arg_types, ret_types))
    }
}

impl TypeReader for core::ImportDesc {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        match reader.read_u8()? {
            0x00 => Ok(Self::TypeIdx(reader.read_leb_usize()?)),
            0x01 => Ok(Self::TableType(core::TableType::read(reader)?)),
            0x02 => Ok(Self::MemType(core::MemType::read(reader)?)),
            0x03 => Ok(Self::GlobalType(core::GlobalType::read(reader)?)),

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown ImportDesc tag",
            )),
        }
    }
}

impl TypeReader for core::Import {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let mod_name = reader.read_name()?;
        let name = reader.read_name()?;
        let import_desc = core::ImportDesc::read(reader)?;

        Ok(Self::new(mod_name, name, import_desc))
    }
}

impl TypeReader for core::Expr {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        Ok(Self::new(parser::read_expression_bytes(reader)?))
    }
}

impl TypeReader for core::GlobalDef {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let gt = core::GlobalType::read(reader)?;
        let e = core::Expr::read(reader)?;

        Ok(Self::new(gt, e))
    }
}

impl TypeReader for core::ExportDesc {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        match reader.read_u8()? {
            0x00 => Ok(core::ExportDesc::Func(reader.read_leb_usize()?)),
            0x01 => Ok(core::ExportDesc::Table(reader.read_leb_usize()?)),
            0x02 => Ok(core::ExportDesc::Mem(reader.read_leb_usize()?)),
            0x03 => Ok(core::ExportDesc::Global(reader.read_leb_usize()?)),

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid export desc type",
            )),
        }
    }
}

impl TypeReader for core::Export {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let nm = reader.read_name()?;
        let d = core::ExportDesc::read(reader)?;

        Ok(Self::new(nm, d))
    }
}

impl TypeReader for core::Element {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let x = reader.read_leb_usize()?;
        let e = core::Expr::read(reader)?;
        let y = reader.read_vec(T::read_leb_u32)?;

        Ok(Self::new(x, e, y))
    }
}

impl TypeReader for core::Locals {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let n = reader.read_leb_u32()?;
        let t = core::ValueType::read(reader)?;

        Ok(Self::new(n, t))
    }
}

impl TypeReader for core::Func {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let size = reader.read_leb_u32()?;

        // Use a subset reader to only read the code part
        let mut payload_reader = ScopedReader::new(reader, usize::try_from(size).unwrap());

        let locals = payload_reader.read_vec(core::Locals::read)?;
        let e = core::Expr::read(&mut payload_reader)?;

        assert!(payload_reader.is_at_end());

        Ok(Self::new(locals, e))
    }
}

impl TypeReader for core::Data {
    fn read<T: io::Read>(reader: &mut T) -> io::Result<Self> {
        let x = reader.read_leb_usize()?;
        let e = core::Expr::read(reader)?;
        let b = reader.read_vec(T::read_u8)?;

        Ok(Self::new(x, e, b))
    }
}
