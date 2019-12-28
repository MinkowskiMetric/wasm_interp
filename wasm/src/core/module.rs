use crate::core;

#[derive(Debug)]
pub struct Module {
    types: Vec<core::FuncType>,
    typeidx: Vec<u32>,
    funcs: Vec<core::Func>,
    tables: Vec<core::TableType>,
    mems: Vec<core::MemType>,
    globals: Vec<core::Global>,
    elem: Vec<core::Element>,
    data: Vec<core::Data>,
    start: Option<u32>,
    imports: Vec<core::Import>,
    exports: Vec<core::Export>,
}

impl Module {
    pub fn new(types: Vec<core::FuncType>, typeidx: Vec<u32>, funcs: Vec<core::Func>, tables: Vec<core::TableType>, mems: Vec<core::MemType>, globals: Vec<core::Global>, elem: Vec<core::Element>, data: Vec<core::Data>, start: Option<u32>, imports: Vec<core::Import>, exports: Vec<core::Export>) -> Self {
        Self { types, typeidx, funcs, tables, mems, globals, elem, data, start, imports, exports }
    }
}