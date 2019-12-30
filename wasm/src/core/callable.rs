use crate::core::{Func, FuncType};

#[derive(Debug)]
pub struct WasmExprCallable {}

#[derive(Debug)]
pub struct DummyCallable {}

#[derive(Debug)]
pub enum Callable {
    WasmExpr(WasmExprCallable),
    DummyCallable(DummyCallable), // This is temporary
}

impl WasmExprCallable {
    pub fn new(_func_type: FuncType, _func: Func) -> Callable {
        Callable::WasmExpr(Self {})
    }
}

impl DummyCallable {
    pub fn new(_mod_name: &str, _name: &str, _func_type: &FuncType) -> Callable {
        Callable::DummyCallable(Self {})
    }
}
