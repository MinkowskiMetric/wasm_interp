use crate::core::{stack_entry::StackEntry, Stack};
use anyhow::Result;

pub trait ConstantDataStore {
    fn get_global_value(&self, idx: usize) -> Result<StackEntry>;
}

pub trait DataStore: ConstantDataStore {
    fn set_global_value(&mut self, idx: usize, value: StackEntry) -> Result<()>;
    fn read_data(&self, mem_idx: usize, offset: usize, data: &mut [u8]) -> Result<()>;
    fn write_data(&mut self, mem_idx: usize, offset: usize, data: &[u8]) -> Result<()>;
    fn get_memory_size(&self, mem_idx: usize) -> Result<usize>;
    fn grow_memory_by(&mut self, mem_idx: usize, grow_by: usize) -> Result<()>;
}

pub trait FunctionStore {
    fn execute_function(
        &self,
        fn_idx: usize,
        stack: &mut Stack,
        data_store: &mut impl DataStore,
    ) -> Result<()>;
    fn execute_indirect_function(
        &self,
        func_type_idx: usize,
        table_idx: usize,
        elem_idx: usize,
        stack: &mut Stack,
        data_store: &mut impl DataStore,
    ) -> Result<()>;
}
