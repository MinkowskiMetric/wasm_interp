use crate::core::{execute_expression, Expr, ExpressionStore, Func, FuncType, Locals, Stack};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct WasmExprCallable {
    func_type: FuncType,
    locals: Vec<Locals>,
    expr: Expr,
}

#[derive(Debug, Clone)]
pub enum Callable {
    WasmExpr(WasmExprCallable),
}

impl Callable {
    pub fn call<Store: ExpressionStore>(&self, stack: &mut Stack, store: &mut Store) -> Result<()> {
        match &self {
            Callable::WasmExpr(e) => e.call(stack, store),
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

    fn call<Store: ExpressionStore>(&self, stack: &mut Stack, store: &mut Store) -> Result<()> {
        // Create the call frame for the function on the stack
        stack.push_typed_frame(&self.func_type, &self.locals)?;

        // Now execute the function on the stack
        let result = execute_expression(&self.expr, stack, store);

        // Pop the function frame off the stack
        stack.pop_typed_frame()?;

        // And we're done
        result
    }
}
