use anyhow::{anyhow, Result};

use super::super::{
    store_access::{RefMutType, RefType},
    ConstantExpressionStore, ExpressionStore,
};
use crate::core::{Callable, FuncType, Global, Locals, Memory, Table, WasmExprCallable};
use crate::parser::InstructionSource;

pub struct TestStore {
    memory: Memory,
    memory_enabled: bool,
    functions: Vec<Callable>,
    func_types: Vec<FuncType>,
    table: Option<Table>,
}

impl TestStore {
    pub fn new() -> Self {
        Self {
            memory: Memory::new_from_bounds(1, Some(3)),
            memory_enabled: false,
            functions: Vec::new(),
            func_types: Vec::new(),
            table: None,
        }
    }

    pub fn enable_memory(&mut self) {
        self.memory_enabled = true;
    }

    pub fn add_function(
        &mut self,
        expr: impl InstructionSource,
        func_type: FuncType,
        locals: Vec<Locals>,
    ) -> usize {
        self.functions.push(WasmExprCallable::new_base(
            func_type,
            locals,
            expr.as_expr(),
        ));
        self.functions.len() - 1
    }

    pub fn set_func_types(&mut self, func_types: Vec<FuncType>) {
        self.func_types = func_types;
    }

    pub fn set_table(&mut self, table: Table) {
        self.table = Some(table);
    }
}

impl ConstantExpressionStore for TestStore {
    type GlobalRef = RefType<Global>;

    fn global_idx<'a>(&'a self, _idx: usize) -> Result<&'a Global> {
        Err(anyhow!("Global value not present in test store"))
    }
}

impl ExpressionStore for TestStore {
    type GlobalRefMut = RefMutType<Global>;
    type FuncTypeRef = RefType<FuncType>;
    type TableRef = RefType<Table>;
    type CallableRef = RefType<Callable>;
    type MemoryRef = RefType<Memory>;
    type MemoryRefMut = RefMutType<Memory>;

    fn global_idx_mut<'a>(&'a mut self, _idx: usize) -> Result<&'a mut Global> {
        Err(anyhow!("Global value not present in test store"))
    }

    fn func_type_idx<'a>(&'a self, idx: usize) -> Result<&'a FuncType> {
        if idx < self.func_types.len() {
            Ok(&self.func_types[idx])
        } else {
            Err(anyhow!("Function type index out of range"))
        }
    }

    fn table_idx<'a>(&'a self, idx: usize) -> Result<&'a Table> {
        if idx == 0 {
            if let Some(table) = &self.table {
                Ok(table)
            } else {
                Err(anyhow!("Table index out of range"))
            }
        } else {
            Err(anyhow!("Table index out of range"))
        }
    }

    fn callable_idx<'a>(&'a self, idx: usize) -> Result<&'a Callable> {
        if idx < self.functions.len() {
            Ok(&self.functions[idx])
        } else {
            Err(anyhow!("Function index out of range"))
        }
    }

    fn mem_idx<'a>(&'a self, idx: usize) -> Result<&'a Memory> {
        if self.memory_enabled && idx == 0 {
            Ok(&self.memory)
        } else {
            Err(anyhow!("Memory not present in store"))
        }
    }

    fn mem_idx_mut<'a>(&'a mut self, idx: usize) -> Result<&'a mut Memory> {
        if self.memory_enabled && idx == 0 {
            Ok(&mut self.memory)
        } else {
            Err(anyhow!("Memory not present in store"))
        }
    }
}
