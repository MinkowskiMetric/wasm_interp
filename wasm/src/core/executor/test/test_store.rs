use anyhow::{anyhow, Result};

use super::super::{
    store_access::{RefMutType, RefType},
    ConstantExpressionStore, ExpressionStore,
};
use crate::core::{Callable, FuncType, Global, Locals, Memory, WasmExprCallable};
use crate::parser::InstructionSource;

pub struct TestStore {
    memory: Memory,
    memory_enabled: bool,
    functions: Vec<Callable>,
}

impl TestStore {
    pub fn new() -> Self {
        Self {
            memory: Memory::new_from_bounds(1, Some(3)),
            memory_enabled: false,
            functions: Vec::new(),
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
}

impl ConstantExpressionStore for TestStore {
    type GlobalRef = RefType<Global>;

    fn global_idx<'a>(&'a self, _idx: usize) -> Result<&'a Global> {
        Err(anyhow!("Global value not present in test store"))
    }
}

impl ExpressionStore for TestStore {
    type GlobalRefMut = RefMutType<Global>;
    type CallableRef = RefType<Callable>;
    type MemoryRef = RefType<Memory>;
    type MemoryRefMut = RefMutType<Memory>;

    fn global_idx_mut<'a>(&'a mut self, _idx: usize) -> Result<&'a mut Global> {
        Err(anyhow!("Global value not present in test store"))
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
