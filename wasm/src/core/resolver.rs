use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use crate::core::{Callable, FuncType, Global, GlobalType, MemType, Memory, Table, TableType};

pub trait Resolver {
    fn resolve_function(
        &self,
        mod_name: &str,
        name: &str,
        func_type: &FuncType,
    ) -> io::Result<Rc<RefCell<Callable>>>;
    fn resolve_table(
        &self,
        mod_name: &str,
        name: &str,
        table_type: &TableType,
    ) -> io::Result<Rc<RefCell<Table>>>;
    fn resolve_memory(
        &self,
        mod_name: &str,
        name: &str,
        mem_type: &MemType,
    ) -> io::Result<Rc<RefCell<Memory>>>;
    fn resolve_global(
        &self,
        mod_name: &str,
        name: &str,
        global_type: &GlobalType,
    ) -> io::Result<Rc<RefCell<Global>>>;
}

pub struct EmptyResolver {}

impl Resolver for EmptyResolver {
    fn resolve_function(
        &self,
        mod_name: &str,
        name: &str,
        _func_type: &FuncType,
    ) -> io::Result<Rc<RefCell<Callable>>> {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Imported function {}:{} not found", mod_name, name),
        ))
    }
    fn resolve_table(
        &self,
        mod_name: &str,
        name: &str,
        _table_type: &TableType,
    ) -> io::Result<Rc<RefCell<Table>>> {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Imported table {}:{} not found", mod_name, name),
        ))
    }
    fn resolve_memory(
        &self,
        mod_name: &str,
        name: &str,
        _mem_type: &MemType,
    ) -> io::Result<Rc<RefCell<Memory>>> {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Imported memory {}:{} not found", mod_name, name),
        ))
    }
    fn resolve_global(
        &self,
        mod_name: &str,
        name: &str,
        _global_type: &GlobalType,
    ) -> io::Result<Rc<RefCell<Global>>> {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Imported global {}:{} not found", mod_name, name),
        ))
    }
}

static EMPTY_RESOLVER_INSTANCE: EmptyResolver = EmptyResolver {};

impl EmptyResolver {
    pub fn instance() -> &'static Self {
        &EMPTY_RESOLVER_INSTANCE
    }
}
