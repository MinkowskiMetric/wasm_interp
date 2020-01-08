use anyhow::{anyhow, Result};
use std::{cell::RefCell, rc::Rc};

use super::super::{ConstantExpressionStore, ExpressionStore};
use crate::core::{stack_entry::StackEntry, Memory};

pub struct TestStore {
    memory: Rc<RefCell<Memory>>,
    memory_enabled: bool,
}

impl TestStore {
    pub fn new() -> Self {
        Self {
            memory: Rc::new(RefCell::new(Memory::new_from_bounds(1, Some(3)))),
            memory_enabled: false,
        }
    }

    pub fn enable_memory(&mut self) {
        self.memory_enabled = true;
    }
}

impl ConstantExpressionStore for TestStore {
    fn get_global_value(&self, _idx: usize) -> Result<StackEntry> {
        Err(anyhow!("Global value not present in test store"))
    }
}

impl ExpressionStore for TestStore {
    fn set_global_value(&mut self, _idx: usize, _value: StackEntry) -> Result<()> {
        Err(anyhow!("Global value not present in test store"))
    }

    fn get_memory(&self, idx: usize) -> Result<Rc<RefCell<Memory>>> {
        if self.memory_enabled {
            if idx == 0 {
                Ok(self.memory.clone())
            } else {
                Err(anyhow!("Memory out of range"))
            }
        } else {
            Err(anyhow!("Memory not present in store"))
        }
    }
}
