use crate::type_section_data::TypeSectionData;
use crate::import_section_data::ImportSectionData;
use crate::reader_util::ReaderUtil;

// There is some reorganizing we need to do here. Probably moving everything into one file
use crate::import_section_data::TableType;
use crate::import_section_data::GlobalType;
use crate::import_section_data::MemType;
use crate::module::SectionPayloadReader;
use crate::type_section_data::ValueType;

use crate::expr::Expr;

use std::convert::TryFrom;
use std::io;
use std::io::Read;

#[derive(Debug)]
pub struct Global {
    gt: GlobalType,
    e: Expr,
}

impl Global {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<Global> {
        let gt = GlobalType::read(reader)?;
        let e = Expr::read(reader)?;

        Ok(Global { gt, e })
    }
}

#[derive(Debug)]
pub enum ExportDesc {
    Func(u32),
    Table(u32),
    Mem(u32),
    Global(u32),
}

impl ExportDesc {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<ExportDesc> {
        match reader.read_u8()? {
            0x00 => Ok(ExportDesc::Func(reader.read_leb_u32()?)),
            0x01 => Ok(ExportDesc::Table(reader.read_leb_u32()?)),
            0x02 => Ok(ExportDesc::Mem(reader.read_leb_u32()?)),
            0x03 => Ok(ExportDesc::Global(reader.read_leb_u32()?)),

            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid export desc type")),
        }
    }
}

#[derive(Debug)]
pub struct Export {
    nm: String,
    d: ExportDesc,
}

impl Export {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<Export> {
        let nm = reader.read_name()?;
        let d = ExportDesc::read(reader)?;

        Ok(Export { nm, d })
    }
}

#[derive(Debug)]
pub struct Element {
    x: u32,
    e: Expr,
    y: Vec<u32>,
}

impl Element {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<Element> {
        let x = reader.read_leb_u32()?;
        let e = Expr::read(reader)?;
        let y = reader.read_vec(T::read_leb_u32)?;

        Ok(Element { x, e, y })
    }
}

#[derive(Debug)]
pub struct Locals {
    n: u32,
    t: ValueType,
}
impl Locals {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<Locals> {
        let n = reader.read_leb_u32()?;
        let t = ValueType::read(reader)?;

        Ok(Locals { n, t })
    }
}

#[derive(Debug)]
pub struct Func {
    locals: Vec<Locals>,
    e: Expr,
}

impl Func {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<Func> {
        let size = reader.read_leb_u32()?;

        // Use a subset reader to only read the code part
        let mut payload_reader = SectionPayloadReader::new(reader, size);

        let locals = payload_reader.read_vec(Locals::read)?;
        let e = Expr::read(&mut payload_reader)?;
        
        assert!(payload_reader.is_at_end());

        Ok(Func { locals, e })
    }
}

#[derive(Debug)]
pub struct Data {
    x: u32,
    e: Expr,
    b: Vec<u8>,
}

impl Data {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<Data> {
        let x = reader.read_leb_u32()?;
        let e = Expr::read(reader)?;
        let b = reader.read_vec(T::read_u8)?;

        Ok(Data { x, e, b })
    }
}

#[derive(Debug)]
pub enum Section {
    CustomSection { section_body: Vec<u8> },
    TypeSection { section_body: TypeSectionData },
    ImportSection { section_body: ImportSectionData },
    FunctionSection { section_body: Vec<u32> },
    TableSection { section_body: Vec<TableType> },
    MemorySection { section_body: Vec<MemType> },
    GlobalSection { section_body: Vec<Global> },
    ExportSection { section_body: Vec<Export> },
    StartSection { start_idx: u32 },
    ElementSection { elements: Vec<Element> },
    CodeSection { code: Vec<Func> },
    DataSection { data: Vec<Data> },
    UnknownSection { section_type: u8, section_body: Vec<u8> },
}

fn read_section_body<T: io::Read>(section_reader: &mut T, section_length: u32) -> io::Result<Vec<u8>> {
    let mut data: Vec<u8> = vec![0; usize::try_from(section_length).unwrap()];

    section_reader.read_exact(&mut data)?;

    Ok(data)
}

pub fn process_section<T: io::Read>(section_type: u8, section_length: u32, section_reader: &mut T) -> io::Result<Section> {
    match section_type {
        0 => Ok(Section::CustomSection { section_body: read_section_body(section_reader, section_length)?, }),

        1 => Ok(Section::TypeSection { section_body: TypeSectionData::read(section_reader)?, }),
        2 => Ok(Section::ImportSection { section_body: ImportSectionData::read(section_reader)?, }),
        3 => Ok(Section::FunctionSection { section_body: section_reader.read_vec(T::read_leb_u32)? }),
        4 => Ok(Section::TableSection { section_body: section_reader.read_vec(TableType::read)? }),
        5 => Ok(Section::MemorySection { section_body: section_reader.read_vec(MemType::read)? }),
        6 => Ok(Section::GlobalSection { section_body: section_reader.read_vec(Global::read)? }),
        7 => Ok(Section::ExportSection { section_body: section_reader.read_vec(Export::read)? }),
        8 => Ok(Section::StartSection { start_idx: section_reader.read_leb_u32()? }),
        9 => Ok(Section::ElementSection { elements: section_reader.read_vec(Element::read)? }),
        10 => Ok(Section::CodeSection { code: section_reader.read_vec(Func::read)? }),
        11 => Ok(Section::DataSection { data: section_reader.read_vec(Data::read)? }),

        _ => Ok(Section::UnknownSection { section_type, section_body: read_section_body(section_reader, section_length)?, }),
    }
}