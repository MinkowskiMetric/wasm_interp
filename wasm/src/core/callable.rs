use crate::core;

#[derive(Debug)]
pub struct WasmExprCallable {

}

#[derive(Debug)]
pub struct DummyCallable {

}

#[derive(Debug)]
pub enum Callable {
    WasmExpr(WasmExprCallable),
    DummyCallable(DummyCallable),       // This is temporary
}

pub type RcCallable = std::rc::Rc<Callable>;

impl WasmExprCallable {
    pub fn new(_func_type: core::FuncType, _func: core::Func) -> Callable {
        Callable::WasmExpr(Self { })
    }
}

impl DummyCallable {
    pub fn new(_mod_name: &str, _name: &str, _func_type: &core::FuncType) -> Callable {
        Callable::DummyCallable(Self { })
    }
}
