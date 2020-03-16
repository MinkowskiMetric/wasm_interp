use crate::core::{
    execute_expression, DataStore, Expr, Func, FuncType, FunctionStore, Locals, Stack,
};
use anyhow::Result;

#[derive(Debug)]
pub struct WasmExprCallable {
    func_type: FuncType,
    locals: Vec<Locals>,
    expr: Expr,
}

#[derive(Debug)]
pub enum Callable {
    WasmExpr(WasmExprCallable),
}

impl Callable {
    pub fn call(
        &self,
        stack: &mut Stack,
        function_store: &impl FunctionStore,
        data_store: &mut impl DataStore,
    ) -> Result<()> {
        match &self {
            Callable::WasmExpr(e) => e.call(stack, function_store, data_store),
        }
    }

    pub fn func_type(&self) -> &FuncType {
        match &self {
            Callable::WasmExpr(e) => &e.func_type,
        }
    }
}

impl WasmExprCallable {
    pub fn new(func_type: FuncType, func: Func) -> Callable {
        Self::new_base(func_type, func.locals().clone(), func.expr().clone())
    }

    pub fn new_base(func_type: FuncType, locals: Vec<Locals>, expr: Expr) -> Callable {
        Callable::WasmExpr(Self {
            func_type,
            locals,
            expr,
        })
    }

    fn call(
        &self,
        stack: &mut Stack,
        function_store: &impl FunctionStore,
        data_store: &mut impl DataStore,
    ) -> Result<()> {
        // Create the call frame for the function on the stack
        stack.push_typed_frame(&self.func_type, &self.locals)?;

        // Now execute the function on the stack
        let result = execute_expression(&self.expr, stack, function_store, data_store);

        // Pop the function frame off the stack
        stack.pop_typed_frame()?;

        // And we're done
        result
    }
}
