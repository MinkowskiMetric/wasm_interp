use crate::core::{stack_entry::StackEntry, Memory};
use anyhow::Result;
use std::{cell::RefCell, rc::Rc};

pub trait ConstantExpressionStore {
    fn get_global_value(&self, idx: usize) -> Result<StackEntry>;
}

pub trait ExpressionStore: ConstantExpressionStore {
    fn set_global_value(&mut self, idx: usize, value: StackEntry) -> Result<()>;

    fn get_memory(&self, idx: usize) -> Result<Rc<RefCell<Memory>>>;

    fn read_data(&self, mem_idx: usize, offset: usize, data: &mut [u8]) -> Result<()> {
        let memory = self.get_memory(mem_idx)?;
        let memory = &memory.borrow();
        memory.get_data(offset, data)
    }

    fn write_data(&mut self, mem_idx: usize, offset: usize, data: &[u8]) -> Result<()> {
        let memory = self.get_memory(mem_idx)?;
        let memory = &mut memory.borrow_mut();
        memory.set_data(offset, data)
    }

    fn get_memory_size(&self, mem_idx: usize) -> Result<usize> {
        let memory = self.get_memory(mem_idx)?;
        let memory = &memory.borrow();
        Ok(memory.current_size())
    }

    fn grow_memory_by(&mut self, mem_idx: usize, grow_by: usize) -> Result<()> {
        let memory = self.get_memory(mem_idx)?;
        let memory = &mut memory.borrow_mut();
        memory.grow_by(grow_by)
    }
}
