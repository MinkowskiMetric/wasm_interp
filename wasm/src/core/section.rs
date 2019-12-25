use std::io;

#[derive(Debug,PartialEq)]
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
    UnknownSection(u8),
}

impl SectionType {
    pub fn from_byte_allow_unknown(byte: u8) -> Self {
        match byte {
            0 => SectionType::CustomSection,
            1 => SectionType::TypeSection,
            2 => SectionType::ImportSection,
            3 => SectionType::FunctionSection,
            4 => SectionType::TableSection,
            5 => SectionType::MemorySection,
            6 => SectionType::GlobalSection,
            7 => SectionType::ExportSection,
            8 => SectionType::StartSection,
            9 => SectionType::ElementSection,
            10 => SectionType::CodeSection,
            11 => SectionType::DataSection,

            b => SectionType::UnknownSection(b),
        }
    }

    pub fn from_byte(byte: u8) -> io::Result<Self> {
        match Self::from_byte_allow_unknown(byte) {
            SectionType::UnknownSection(b) => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unknown section type 0x{:02x}", b))),
            r => Ok(r),
        }
    }
}
