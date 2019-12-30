use std::io;

use crate::core;
use crate::parser::{self, Opcode};

pub struct ConstantExpressionExecutor {}

static CONSTANT_EXPRESSION_EXECUTOR_INSTANCE: ConstantExpressionExecutor =
    ConstantExpressionExecutor {};

impl ConstantExpressionExecutor {
    pub fn instance() -> &'static Self {
        &CONSTANT_EXPRESSION_EXECUTOR_INSTANCE
    }

    // Not totally sure on the return type here right now.
    pub fn execute_constant_expression<ExprType: parser::InstructionSource>(
        &self,
        expr: &ExprType,
        _module: &core::Module,
    ) -> io::Result<u32> {
        let mut ret: Option<u32> = None;

        for instruction in expr.iter() {
            let instruction = instruction?;

            match instruction.opcode() {
                // There is only a very limited set of instructions that are allowed in a constant expression
                Opcode::I32Const => {
                    println!("I32Const {}", instruction.get_single_u32_arg());
                    ret = Some(instruction.get_single_u32_arg());
                }
                Opcode::I64Const => {
                    println!("I64Const {}", instruction.get_single_u64_arg());
                    ret = None;
                }
                Opcode::F32Const => {
                    println!("F32Const {}", instruction.get_single_f32_arg());
                    ret = None;
                }
                Opcode::F64Const => {
                    println!("F64Const {}", instruction.get_single_f64_arg());
                    ret = None;
                }

                Opcode::GlobalGet => {
                    println!("GlobalGet {}", instruction.get_single_usize_arg());
                    unimplemented!();
                }

                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Instruction is not valid in constant expression",
                    ));
                }
            }
        }

        match ret {
            Some(v) => Ok(v),
            None => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Constant expression did not yield a useful result",
            )),
        }
    }
}
