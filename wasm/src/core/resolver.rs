use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use crate::core::{
    Callable, DummyCallable, FuncType, Global, GlobalType, MemType, Memory, Table, TableType,
};

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
        func_type: &FuncType,
    ) -> io::Result<Rc<RefCell<Callable>>> {
        Ok(Rc::new(RefCell::new(DummyCallable::new(
            mod_name, name, func_type,
        ))))
    }
    fn resolve_table(
        &self,
        _mod_name: &str,
        _name: &str,
        table_type: &TableType,
    ) -> io::Result<Rc<RefCell<Table>>> {
        Ok(Rc::new(RefCell::new(Table::new(table_type.clone()))))
    }
    fn resolve_memory(
        &self,
        _mod_name: &str,
        _name: &str,
        mem_type: &MemType,
    ) -> io::Result<Rc<RefCell<Memory>>> {
        Ok(Rc::new(RefCell::new(Memory::new(mem_type.clone()))))
    }
    fn resolve_global(
        &self,
        _mod_name: &str,
        _name: &str,
        global_type: &GlobalType,
    ) -> io::Result<Rc<RefCell<Global>>> {
        Ok(Rc::new(RefCell::new(Global::new_dummy(
            global_type.clone(),
        ))))
    }
}

static EMPTY_RESOLVER_INSTANCE: EmptyResolver = EmptyResolver {};

impl EmptyResolver {
    pub fn instance() -> &'static Self {
        &EMPTY_RESOLVER_INSTANCE
    }
}
