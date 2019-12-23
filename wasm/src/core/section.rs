use crate::core;

#[derive(Debug)]
pub enum Section {
    CustomSection { name: String, body: Vec<u8> },
    TypeSection { section_body: Vec<core::FuncType> },
    ImportSection { section_body: Vec<core::Import> },
    FunctionSection { section_body: Vec<u32> },
    TableSection { section_body: Vec<core::TableType> },
    MemorySection { section_body: Vec<core::MemType> },
    GlobalSection { section_body: Vec<core::Global> },
    ExportSection { section_body: Vec<core::Export> },
    StartSection { start_idx: u32 },
    ElementSection { elements: Vec<core::Element> },
    CodeSection { code: Vec<core::Func> },
    DataSection { data: Vec<core::Data> },
    UnknownSection { section_type: u8, section_body: Vec<u8> },
}

