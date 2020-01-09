use crate::core::{execute_expression, Expr, ExpressionStore, Func, FuncType, Locals, Stack};
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
    pub fn call<Store: ExpressionStore>(&self, stack: &mut Stack, store: &mut Store) -> Result<()> {
        match &self {
            Callable::WasmExpr(e) => e.call(stack, store),
        }
    }
}

impl WasmExprCallable {
    pub fn new(func_type: FuncType, func: Func) -> Callable {
        Callable::WasmExpr(Self {
            func_type,
            locals: func.locals().clone(),
            expr: func.expr().clone(),
        })
    }

    fn call<Store: ExpressionStore>(&self, stack: &mut Stack, store: &mut Store) -> Result<()> {
        // I haven't done any support for locals or parameters or returns at this point.
        // Mostly just because I don't need to do the support for it yet to test simple value setting
        // and it is a lot of type checking boilerplate
        assert!(self.func_type.arg_types().is_empty());
        assert!(self.func_type.return_types().is_empty());
        assert!(self.locals.is_empty());

        // Create the call frame for the function on the stack
        stack.push_frame(0, 0);

        // Now execute the function on the stack
        let result = execute_expression(&self.expr, stack, store);

        // Pop the function frame off the stack
        stack.pop_frame(0);

        // And we're done
        result
    }
}
