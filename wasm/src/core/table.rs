use std::cell::RefCell;
use std::rc::Rc;

use crate::core::{Callable, TableType};

#[derive(Debug)]
pub struct Table {}

impl Table {
    pub fn new(_table_type: TableType) -> Self {
        Table {}
    }

    pub fn set_entries(&mut self, offset: usize, functions: &[Rc<RefCell<Callable>>]) {
        println!(
            "table: {:?} offset: {:?} functions: {:?}",
            self, offset, functions
        );
    }
}
