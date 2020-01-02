use crate::core::{Func, FuncType};

#[derive(Debug)]
pub struct WasmExprCallable {}

#[derive(Debug)]
pub enum Callable {
    WasmExpr(WasmExprCallable),
}

impl WasmExprCallable {
    pub fn new(_func_type: FuncType, _func: Func) -> Callable {
        Callable::WasmExpr(Self {})
    }
}
