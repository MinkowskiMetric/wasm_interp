use std::io;

#[derive(Debug)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

impl ValueType {
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
pub enum MutableType {
    Const,
    Var,
}

impl MutableType {
    pub fn from_byte(byte: u8) -> io::Result<MutableType> {
        match byte {
            0x00 => Ok(MutableType::Const),
            0x01 => Ok(MutableType::Var),

            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown mutable type")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ElemType { 
    FuncRef
}

impl ElemType {
    pub fn from_byte(byte: u8) -> io::Result<Self> {
        match byte {
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

#[derive(Debug)]
pub struct TableType {
    et: ElemType,
    lim: Limits,
}

impl TableType {
    pub fn new(et: ElemType, lim: Limits) -> Self {
        Self { et, lim }
    }
}

#[derive(Debug)]
pub struct MemType {
    limits: Limits,
}

impl MemType {
    pub fn new(limits: Limits) -> Self {
        Self { limits }
    }
}

#[derive(Debug)]
pub struct GlobalType {
    t: ValueType,
    m: MutableType,
}

impl GlobalType {
    pub fn new(t: ValueType, m: MutableType) -> Self {
        Self { t, m }
    }
}

#[derive(Debug)]
pub struct FuncType {
    arg_types: Vec<ValueType>,
    ret_types: Vec<ValueType>,
}

impl FuncType {
    pub fn new(arg_types: Vec<ValueType>, ret_types: Vec<ValueType>) -> FuncType {
        FuncType { arg_types, ret_types }
    }
}

#[derive(Debug)]
pub enum ImportDesc {
    TypeIdx(u32),
    TableType(TableType),
    MemType(MemType),
    GlobalType(GlobalType),
}

#[derive(Debug)]
pub struct Import {
    mod_name: String,
    name: String,
    import_desc: ImportDesc,
}

impl Import {
    pub fn new(mod_name: String, name: String, import_desc: ImportDesc) -> Self {
        Self { mod_name, name, import_desc }
    }
}

#[derive(Debug)]
pub struct Expr {
    // So, a basic expr is just the bytes that make up the expression
    instr: Vec<u8>,
}

impl Expr {
    pub fn new(instr: Vec<u8>) -> Self {
        Self { instr }
    }
}

#[derive(Debug)]
pub struct Global {
    gt: GlobalType,
    e: Expr,
}

impl Global {
    pub fn new(gt: GlobalType, e: Expr) -> Self {
        Self { gt, e }
    }
}

#[derive(Debug)]
pub enum ExportDesc {
    Func(u32),
    Table(u32),
    Mem(u32),
    Global(u32),
}

#[derive(Debug)]
pub struct Export {
    nm: String,
    d: ExportDesc,
}

impl Export {
    pub fn new(nm: String, d: ExportDesc) -> Self {
        Self { nm, d }
    }
}

#[derive(Debug)]
pub struct Element {
    x: u32,
    e: Expr,
    y: Vec<u32>,
}

impl Element {
    pub fn new(x: u32, e: Expr, y: Vec<u32>) -> Self {
        Self { x, e, y }
    }
}

#[derive(Debug)]
pub struct Locals {
    n: u32,
    t: ValueType,
}

impl Locals {
    pub fn new(n: u32, t: ValueType) -> Self {
        Self { n, t }
    }
}

#[derive(Debug)]
pub struct Func {
    locals: Vec<Locals>,
    e: Expr,
}

impl Func {
    pub fn new(locals: Vec<Locals>, e: Expr) -> Self {
        Self { locals, e }
    }
}

#[derive(Debug)]
pub struct Data {
    x: u32,
    e: Expr,
    b: Vec<u8>,
}

impl Data {
    pub fn new(x: u32, e: Expr, b: Vec<u8>) -> Self {
        Self { x, e, b }
    }
}