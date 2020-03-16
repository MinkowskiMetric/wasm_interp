use anyhow::{anyhow, Result};

use super::super::{ConstantDataStore, DataStore, FunctionStore};
use crate::core::{
    stack_entry::StackEntry, Callable, FuncType, Locals, Memory, Stack, Table, WasmExprCallable,
};
use crate::parser::InstructionSource;

pub struct TestDataStore {
    memory: Memory,
    memory_enabled: bool,
}

impl TestDataStore {
    pub fn new() -> Self {
        Self {
            memory: Memory::new_from_bounds(1, Some(3)),
            memory_enabled: false,
        }
    }

    pub fn enable_memory(&mut self) {
        self.memory_enabled = true;
    }
}

impl ConstantDataStore for TestDataStore {
    fn get_global_value(&self, _idx: usize) -> Result<StackEntry> {
        Err(anyhow!("Global value not present in test store"))
    }
}

impl DataStore for TestDataStore {
    fn set_global_value(&mut self, _idx: usize, _value: StackEntry) -> Result<()> {
        Err(anyhow!("Global value not present in test store"))
    }

    fn read_data(&self, mem_idx: usize, offset: usize, data: &mut [u8]) -> Result<()> {
        if self.memory_enabled && mem_idx == 0 {
            self.memory.get_data(offset, data)
        } else {
            Err(anyhow!("Memory index out of range"))
        }
    }

    fn write_data(&mut self, mem_idx: usize, offset: usize, data: &[u8]) -> Result<()> {
        if self.memory_enabled && mem_idx == 0 {
            self.memory.set_data(offset, data)
        } else {
            Err(anyhow!("Memory index out of range"))
        }
    }

    fn get_memory_size(&self, mem_idx: usize) -> Result<usize> {
        if self.memory_enabled && mem_idx == 0 {
            Ok(self.memory.current_size())
        } else {
            Err(anyhow!("Memory index out of range"))
        }
    }

    fn grow_memory_by(&mut self, mem_idx: usize, grow_by: usize) -> Result<()> {
        if self.memory_enabled && mem_idx == 0 {
            self.memory.grow_by(grow_by)
        } else {
            Err(anyhow!("Memory index out of range"))
        }
    }
}

pub struct TestFunctionStore {
    functions: Vec<Callable>,
    func_types: Vec<FuncType>,
    table: Option<Table>,
}

impl TestFunctionStore {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            func_types: Vec::new(),
            table: None,
        }
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

impl FunctionStore for TestFunctionStore {
    fn execute_function(
        &self,
        idx: usize,
        stack: &mut Stack,
        data_store: &mut impl DataStore,
    ) -> Result<()> {
        if idx < self.functions.len() {
            let callable = &self.functions[idx];
            callable.call(stack, self, data_store)
        } else {
            Err(anyhow!("Callable index out of range"))
        }
    }

    fn execute_indirect_function(
        &self,
        func_type_idx: usize,
        table_idx: usize,
        elem_idx: usize,
        stack: &mut Stack,
        data_store: &mut impl DataStore,
    ) -> Result<()> {
        if func_type_idx >= self.func_types.len() {
            Err(anyhow!("FuncType index out of range"))
        } else if table_idx != 0 || self.table.is_none() {
            Err(anyhow!("Table index out of range"))
        } else {
            let callable = self.table.as_ref().unwrap().get_entry(elem_idx)?;
            let callable = callable.borrow();

            if *callable.func_type() != self.func_types[func_type_idx] {
                Err(anyhow!("Indirect function call type does not match"))
            } else {
                callable.call(stack, self, data_store)
            }
        }
    }
}

pub fn make_test_store() -> (TestFunctionStore, TestDataStore) {
    (TestFunctionStore::new(), TestDataStore::new())
}
