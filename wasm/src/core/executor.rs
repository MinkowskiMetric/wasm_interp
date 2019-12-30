use std::io;

use crate::core;
use crate::parser;

pub struct ConstantExpressionExecutor {

}

static CONSTANT_EXPRESSION_EXECUTOR_INSTANCE: ConstantExpressionExecutor = ConstantExpressionExecutor { };

impl ConstantExpressionExecutor {
    pub fn instance() -> &'static Self {
        &CONSTANT_EXPRESSION_EXECUTOR_INSTANCE
    }

    // Not totally sure on the return type here right now.
    pub fn execute_constant_expression<ExprType: parser::InstructionSource>(&self, expr: &ExprType, _module: &core::Module) -> io::Result<u32> {
        for instruction in expr.iter() {
            println!("Instruction: {:?}", instruction);
        }

        unimplemented!()
    }
}