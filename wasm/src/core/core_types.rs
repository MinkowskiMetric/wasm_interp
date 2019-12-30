use num_enum::TryFromPrimitive;
use std::convert::TryInto;
use std::io::{Error, ErrorKind, Result};

use crate::parser::InstructionSource;

#[derive(Debug, Clone, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum ValueType {
    F64 = 0x7C,
    F32 = 0x7D,
    I64 = 0x7E,
    I32 = 0x7F,
}

impl ValueType {
    pub fn from_byte(byte: u8) -> Result<Self> {
        // actual values are offset by 0x7C [cb]
        match byte.try_into() {
            Ok(v) => Ok(v),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid value type byte 0x{:02x}", byte),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, TryFromPrimitive)]
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

#[derive(Debug, Clone, PartialEq, TryFromPrimitive)]
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

#[derive(Debug, Clone)]
pub enum Limits {
    Unbounded(u32),
    Bounded(u32, u32),
}

#[derive(Debug, Clone)]
pub struct TableType {
    et: ElemType,
    lim: Limits,
}

impl TableType {
    pub fn new(et: ElemType, lim: Limits) -> Self {
        Self { et, lim }
    }
}

#[derive(Debug, Clone)]
pub struct MemType {
    limits: Limits,
}

impl MemType {
    pub fn new(limits: Limits) -> Self {
        Self { limits }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalType {
    t: ValueType,
    m: MutableType,
}

impl GlobalType {
    pub fn new(t: ValueType, m: MutableType) -> Self {
        Self { t, m }
    }
}

#[derive(Debug, Clone)]
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
    TypeIdx(usize),
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

    pub fn mod_name(&self) -> &str {
        &self.mod_name
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn desc(&self) -> &ImportDesc {
        &self.import_desc
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    // So, a basic expr is just the bytes that make up the expression
    instr: Vec<u8>,
}

impl Expr {
    pub fn new(instr: Vec<u8>) -> Self {
        Self { instr }
    }
}

impl InstructionSource for Expr {
    fn get_instruction_bytes(&self) -> &[u8] {
        &self.instr
    }
}

#[derive(Debug)]
pub struct GlobalDef {
    gt: GlobalType,
    e: Expr,
}

impl GlobalDef {
    pub fn new(gt: GlobalType, e: Expr) -> Self {
        Self { gt, e }
    }
}

#[derive(Debug)]
pub enum ExportDesc {
    Func(usize),
    Table(usize),
    Mem(usize),
    Global(usize),
}

#[derive(Debug)]
pub struct Export {
    pub nm: String,
    pub d: ExportDesc,
}

impl Export {
    pub fn new(nm: String, d: ExportDesc) -> Self {
        Self { nm, d }
    }
}

#[derive(Debug)]
pub struct Element {
    x: usize,
    e: Expr,
    y: Vec<u32>,
}

impl Element {
    pub fn new(x: usize, e: Expr, y: Vec<u32>) -> Self {
        Self { x, e, y }
    }

    pub fn table_idx(&self) -> usize {
        self.x
    }

    pub fn expr(&self) -> &Expr {
        &self.e
    }
}

#[derive(Debug, Clone)]
pub struct Locals {
    n: u32,
    t: ValueType,
}

impl Locals {
    pub fn new(n: u32, t: ValueType) -> Self {
        Self { n, t }
    }
}

#[derive(Debug, Clone)]
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
    x: usize,
    e: Expr,
    b: Vec<u8>,
}

impl Data {
    pub fn new(x: usize, e: Expr, b: Vec<u8>) -> Self {
        Self { x, e, b }
    }

    pub fn mem_idx(&self) -> usize {
        self.x
    }

    pub fn expr(&self) -> &Expr {
        &self.e
    }
}
