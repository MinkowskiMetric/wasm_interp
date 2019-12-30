use crate::core;

#[derive(Debug)]
pub struct Table {}

impl Table {
    pub fn new(_table_type: core::TableType) -> Self {
        Table {}
    }
}

pub type RcTable = std::rc::Rc<Table>;
