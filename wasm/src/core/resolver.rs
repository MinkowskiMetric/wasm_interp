use std::io;

use crate::core;

pub trait Resolver {
    fn resolve_function(&self, mod_name: &str, name: &str, func_type: &core::FuncType) -> io::Result<core::RcCallable>;
    fn resolve_table(&self, mod_name: &str, name: &str, table_type: &core::TableType) -> io::Result<core::RcTable>;
    fn resolve_memory(&self, mod_name: &str, name: &str, mem_type: &core::MemType) -> io::Result<core::RcMemory>;
    fn resolve_global(&self, mod_name: &str, name: &str, global_type: &core::GlobalType) -> io::Result<core::RcGlobal>;
}

pub struct EmptyResolver {

}

impl Resolver for EmptyResolver {
    fn resolve_function(&self, mod_name: &str, name: &str, func_type: &core::FuncType) -> io::Result<core::RcCallable> {
        Ok(std::rc::Rc::new(core::DummyCallable::new(mod_name, name, func_type)))
    }
    fn resolve_table(&self, _mod_name: &str, _name: &str, table_type: &core::TableType) -> io::Result<core::RcTable> {
        Ok(std::rc::Rc::new(core::Table::new(table_type.clone())))
    }
    fn resolve_memory(&self, _mod_name: &str, _name: &str, mem_type: &core::MemType) -> io::Result<core::RcMemory> {
        Ok(std::rc::Rc::new(core::Memory::new(mem_type.clone())))
    }
    fn resolve_global(&self, _mod_name: &str, _name: &str, global_type: &core::GlobalType) -> io::Result<core::RcGlobal> {
        Ok(std::rc::Rc::new(core::Global::new_dummy(global_type.clone())))
    }
}

static EMPTY_RESOLVER_INSTANCE: EmptyResolver = EmptyResolver { };

impl EmptyResolver {
    pub fn instance() -> &'static Self {
        &EMPTY_RESOLVER_INSTANCE
    }
}