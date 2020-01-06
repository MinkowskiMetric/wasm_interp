use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::rc::Rc;

use crate::core::{Callable, FuncType, Global, GlobalType, MemType, Memory, Table, TableType};

pub trait Resolver {
    fn resolve_function(
        &self,
        mod_name: &str,
        name: &str,
        func_type: &FuncType,
    ) -> Result<Rc<RefCell<Callable>>>;
    fn resolve_table(
        &self,
        mod_name: &str,
        name: &str,
        table_type: &TableType,
    ) -> Result<Rc<RefCell<Table>>>;
    fn resolve_memory(
        &self,
        mod_name: &str,
        name: &str,
        mem_type: &MemType,
    ) -> Result<Rc<RefCell<Memory>>>;
    fn resolve_global(
        &self,
        mod_name: &str,
        name: &str,
        global_type: &GlobalType,
    ) -> Result<Rc<RefCell<Global>>>;
}

pub struct EmptyResolver {}

impl Resolver for EmptyResolver {
    fn resolve_function(
        &self,
        mod_name: &str,
        name: &str,
        _func_type: &FuncType,
    ) -> Result<Rc<RefCell<Callable>>> {
        Err(anyhow!("Imported function {}:{} not found", mod_name, name))
    }
    fn resolve_table(
        &self,
        mod_name: &str,
        name: &str,
        _table_type: &TableType,
    ) -> Result<Rc<RefCell<Table>>> {
        Err(anyhow!("Imported table {}:{} not found", mod_name, name))
    }
    fn resolve_memory(
        &self,
        mod_name: &str,
        name: &str,
        _mem_type: &MemType,
    ) -> Result<Rc<RefCell<Memory>>> {
        Err(anyhow!("Imported memory {}:{} not found", mod_name, name))
    }
    fn resolve_global(
        &self,
        mod_name: &str,
        name: &str,
        _global_type: &GlobalType,
    ) -> Result<Rc<RefCell<Global>>> {
        Err(anyhow!("Imported global {}:{} not found", mod_name, name))
    }
}

static EMPTY_RESOLVER_INSTANCE: EmptyResolver = EmptyResolver {};

impl EmptyResolver {
    pub fn instance() -> &'static Self {
        &EMPTY_RESOLVER_INSTANCE
    }
}
