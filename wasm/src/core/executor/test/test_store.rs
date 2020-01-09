use anyhow::{anyhow, Result};

use super::super::{
    store_access::{RefMutType, RefType},
    ConstantExpressionStore, ExpressionStore,
};
use crate::core::{Global, Memory};

pub struct TestStore {
    memory: Memory,
    memory_enabled: bool,
}

impl TestStore {
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

impl ConstantExpressionStore for TestStore {
    type GlobalRef = RefType<Global>;

    fn global_idx<'a>(&'a self, _idx: usize) -> Result<&'a Global> {
        Err(anyhow!("Global value not present in test store"))
    }
}

impl ExpressionStore for TestStore {
    type GlobalRefMut = RefMutType<Global>;
    type MemoryRef = RefType<Memory>;
    type MemoryRefMut = RefMutType<Memory>;

    fn global_idx_mut<'a>(&'a mut self, _idx: usize) -> Result<&'a mut Global> {
        Err(anyhow!("Global value not present in test store"))
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
