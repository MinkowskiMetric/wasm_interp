use num_enum::TryFromPrimitive;
use std::convert::TryInto;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum ValueType {
    F64 = 0x7C,
    F32 = 0x7D,
    I64 = 0x7E,
    I32 = 0x7F,
}

impl ValueType {
    pub fn from_byte(byte: u8) -> Result<Self> {
        // actual values are offset by 0x7C [chrbrn]
        match byte.try_into() {
            Ok(v) => Ok(v),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid value type byte 0x{:02x}", byte),
            )),
        }
    }
}

#[derive(Debug, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum MutableType {
    Const,
    Var,
}

impl MutableType {
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte.try_into() {
            Ok(b) => Ok(b),
            _ => Err(Error::new(ErrorKind::InvalidData, "Unknown mutable type")),
        }
    }
}

#[derive(Debug, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum ElemType {
    FuncRef = 0x70,
}

impl ElemType {
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte.try_into() {
            Ok(s) => Ok(s),
            _ => Err(Error::new(ErrorKind::InvalidData, "Unknown funcref type")),
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
        FuncType {
            arg_types,
            ret_types,
        }
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
        Self {
            mod_name,
            name,
            import_desc,
        }
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
