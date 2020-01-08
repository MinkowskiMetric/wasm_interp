use crate::core::{stack_entry::StackEntry, Memory};
use anyhow::Result;
use std::{cell::RefCell, rc::Rc};

pub trait ConstantExpressionStore {
    fn get_global_value(&self, idx: usize) -> Result<StackEntry>;
}

pub trait ExpressionStore: ConstantExpressionStore {
    fn set_global_value(&mut self, idx: usize, value: StackEntry) -> Result<()>;

    fn get_memory(&self, idx: usize) -> Result<Rc<RefCell<Memory>>>;
}
