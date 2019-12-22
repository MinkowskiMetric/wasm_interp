use std::io;

use crate::reader_util::ReaderUtil;
use crate::type_section_data::ValueType;

#[derive(Debug)]
pub enum MutableType {
    Const,
    Var,
}

impl MutableType {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<MutableType> {
        match reader.read_u8()? {
            0x00 => Ok(MutableType::Const),
            0x01 => Ok(MutableType::Var),

            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown mutable type")),
        }
    }
}

#[derive(Debug)]
pub enum ElemType { 
    FuncRef
}

impl ElemType {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<ElemType> {
        match reader.read_u8()? {
            0x70 => Ok(ElemType::FuncRef),

            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown funcref type")),
        }
    }
}

#[derive(Debug)]
pub enum Limits {
    Unbounded(u32),
    Bounded(u32, u32),
}

impl Limits {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<Limits> {
        match reader.read_u8()? {
            0x00 => Ok(Limits::Unbounded(reader.read_leb_u32()?)),
            0x01 => {
                let min = reader.read_leb_u32()?;
                let max = reader.read_leb_u32()?;
                
                Ok(Limits::Bounded(min, max))
            },

            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown Limits tag")),
        }
    }
}

#[derive(Debug)]
pub struct TableType {
    et: ElemType,
    lim: Limits,
}

impl TableType {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<TableType> {
        let et = ElemType::read(reader)?;
        let lim = Limits::read(reader)?;

        Ok(TableType { et, lim })
    }
}

#[derive(Debug)]
pub struct MemType {
    limits: Limits,
}

impl MemType {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<MemType> {
        Ok(MemType { limits: Limits::read(reader)? })
    }
}

#[derive(Debug)]
pub struct GlobalType {
    t: ValueType,
    m: MutableType,
}

impl GlobalType {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<GlobalType> {
        let t = ValueType::read(reader)?;
        let m = MutableType::read(reader)?;

        Ok(GlobalType { t, m })
    }
}
#[derive(Debug)]
pub enum ImportDesc {
    TypeIdx(u32),
    TableType(TableType),
    MemType(MemType),
    GlobalType(ValueType, MutableType),
}

impl ImportDesc {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<ImportDesc> {
        match reader.read_u8()? {
            0x00 => Ok(ImportDesc::TypeIdx(reader.read_leb_u32()?)),
            0x01 => Ok(ImportDesc::TableType(TableType::read(reader)?)),
            0x02 => Ok(ImportDesc::MemType(MemType::read(reader)?)),
            0x03 => {
                let value_type = ValueType::read(reader)?;
                let mut_type = MutableType::read(reader)?;

                Ok(ImportDesc::GlobalType(value_type, mut_type))
            },

            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown ImportDesc tag")),
        }
    }
}

#[derive(Debug)]
pub struct Import {
    mod_name: String,
    name: String,
    import_desc: ImportDesc,
}

impl Import {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<Import> {
        let mod_name = reader.read_name()?;
        let name = reader.read_name()?;
        let import_desc = ImportDesc::read(reader)?;

        Ok(Import { mod_name, name, import_desc })
    }
}

#[derive(Debug)]
pub struct ImportSectionData {
    imports: Vec<Import>,
}

impl ImportSectionData {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<ImportSectionData> {
        let imports = reader.read_vec(Import::read)?;

        Ok(ImportSectionData { imports })
    }
}