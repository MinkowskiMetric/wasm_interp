use crate::core::{stack_entry::StackEntry, stack_entry::StackEntryValueType, Stack};
use crate::parser::{self, Opcode};
use anyhow::{anyhow, Result};

fn convert_stack_entry_to_value<ParamType: StackEntryValueType>(
    e: StackEntry,
) -> Result<ParamType> {
    // This is only necessary because I don't have a clean sensible error handling strategy
    match ParamType::try_into_value(e) {
        Ok(v) => Ok(v),
        _ => Err(anyhow!("Failed to convert stack value")),
    }
}

fn binary_op<ParamType: StackEntryValueType, Func: Fn(ParamType, ParamType) -> ParamType>(
    stack: &mut Stack,
    func: Func,
) -> Result<()> {
    if stack.working_count() < 2 {
        Err(anyhow!("Not enough items on stack for op"))
    } else {
        let ops = stack.working_top(2);
        let ret = func(
            convert_stack_entry_to_value(ops[0])?,
            convert_stack_entry_to_value(ops[1])?,
        );

        stack.push(ret.from_value());
        stack.drop_entries(2, 1);
        Ok(())
    }
}

pub struct ConstantExpressionExecutor {}
pub struct ExpressionExecutor {}

static CONSTANT_EXPRESSION_EXECUTOR_INSTANCE: ConstantExpressionExecutor =
    ConstantExpressionExecutor {};
static EXPRESSION_EXECUTOR_INSTANCE: ExpressionExecutor = ExpressionExecutor {};

pub trait ConstantExpressionStore {
    fn get_global_value(&self, idx: usize) -> Result<StackEntry>;
}

pub trait ExpressionStore: ConstantExpressionStore {
    fn set_global_value(&mut self, idx: usize, value: StackEntry) -> Result<()>;
}

impl ConstantExpressionExecutor {
    pub fn instance() -> &'static Self {
        &CONSTANT_EXPRESSION_EXECUTOR_INSTANCE
    }

    // Not totally sure on the return type here right now.
    pub fn execute_constant_expression<
        ExprType: parser::InstructionSource,
        StoreType: ConstantExpressionStore,
    >(
        &self,
        expr: &ExprType,
        store: &StoreType,
        arity: usize,
    ) -> Result<Vec<StackEntry>> {
        let mut stack = Stack::new();

        for instruction in expr.iter() {
            let instruction = instruction?;

            match instruction.opcode() {
                // There is only a very limited set of instructions that are allowed in a constant expression
                Opcode::I32Const => {
                    stack.push(instruction.get_single_i32_arg().into());
                }
                Opcode::I64Const => {
                    stack.push(instruction.get_single_i64_arg().into());
                }
                Opcode::F32Const => {
                    stack.push(instruction.get_single_f32_arg().into());
                }
                Opcode::F64Const => {
                    stack.push(instruction.get_single_f64_arg().into());
                }

                Opcode::GlobalGet => {
                    stack.push(store.get_global_value(instruction.get_single_u32_as_usize_arg())?);
                }

                _ => {
                    return Err(anyhow!("Instruction is not valid in constant expression"));
                }
            }
        }

        if stack.working_count() < arity {
            return Err(anyhow!("Not enough values returned by constant expression"));
        }

        Ok(stack.frame()[stack.working_limit() - arity..stack.working_limit()].to_vec())
    }
}

impl ExpressionExecutor {
    pub fn instance() -> &'static Self {
        &EXPRESSION_EXECUTOR_INSTANCE
    }

    fn get_stack_top(stack: &mut Stack, n: usize) -> Result<&[StackEntry]> {
        if stack.working_count() < n {
            Err(anyhow!("Not enough values on stack"))
        } else {
            Ok(stack.working_top(n))
        }
    }

    pub fn execute<ExprType: parser::InstructionSource, StoreType: ExpressionStore>(
        &self,
        expr: &ExprType,
        stack: &mut Stack,
        store: &mut StoreType,
    ) -> anyhow::Result<()> {
        for instruction in expr.iter() {
            let instruction = instruction?;

            match instruction.opcode() {
                // There is only a very limited set of instructions that are allowed in a constant expression
                Opcode::I32Const => {
                    stack.push(instruction.get_single_i32_arg().into());
                }
                Opcode::I64Const => {
                    stack.push(instruction.get_single_i64_arg().into());
                }
                Opcode::F32Const => {
                    stack.push(instruction.get_single_f32_arg().into());
                }
                Opcode::F64Const => {
                    stack.push(instruction.get_single_f64_arg().into());
                }

                Opcode::GlobalGet => {
                    stack.push(store.get_global_value(instruction.get_single_u32_as_usize_arg())?)
                }
                Opcode::GlobalSet => {
                    let arg = Self::get_stack_top(stack, 1)?[0];
                    stack.pop();

                    store.set_global_value(instruction.get_single_u32_as_usize_arg(), arg)?;
                }

                Opcode::I32Add => binary_op(stack, |a: u32, b| a.wrapping_add(b))?,

                _ => {
                    return Err(anyhow!(
                        "Instruction {:?} is not valid in constant expression",
                        instruction
                    ));
                }
            }
        }

        Ok(())
    }
}
