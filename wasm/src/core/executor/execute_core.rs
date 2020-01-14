use std::convert::TryFrom;

use crate::core::{stack_entry::StackEntry, BlockType, Stack};
use crate::parser::{Instruction, InstructionSource, Opcode};
use anyhow::{anyhow, Result};

use super::memory_access::{mem_load, mem_store};
use super::stack_ops::{binary_boolean_op, binary_op, get_stack_top, unary_boolean_op, unary_op};

pub use super::store_access::{
    CellRefMutType, CellRefType, ConstantExpressionStore, ExpressionStore, LifetimeToRef,
    RefMutType, RefType,
};

fn execute_single_constant_instruction(
    instruction: Instruction,
    stack: &mut Stack,
    store: &impl ConstantExpressionStore,
) -> Result<()> {
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

        o => {
            return Err(anyhow!(
                "Opcode {:?} is not valid in constant expression",
                o
            ));
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum InstructionResult {
    Block,
    Loop,
    If,
    Br,
    BrIf,
    BrTable,
    Return,
    Call,
    CallIndirect,
}

#[derive(Debug, Clone, PartialEq)]
enum SingleInstructionResult {
    Done,
    ControlInstruction(InstructionResult),
}

fn execute_single_instruction(
    instruction: &Instruction,
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
) -> Result<SingleInstructionResult> {
    match instruction.opcode() {
        Opcode::Unreachable => return Err(anyhow!("Unreachable opcode")),
        Opcode::Nop => {}
        Opcode::Block => {
            return Ok(SingleInstructionResult::ControlInstruction(
                InstructionResult::Block,
            ))
        }
        Opcode::Loop => {
            return Ok(SingleInstructionResult::ControlInstruction(
                InstructionResult::Loop,
            ))
        }
        Opcode::If => {
            return Ok(SingleInstructionResult::ControlInstruction(
                InstructionResult::If,
            ))
        }
        Opcode::Else => panic!("Else opcode should not pass through opcode iterator"),
        Opcode::End => panic!("End opcode should not pass through opcode iterator"),
        Opcode::Br => {
            return Ok(SingleInstructionResult::ControlInstruction(
                InstructionResult::Br,
            ))
        }
        Opcode::BrIf => {
            return Ok(SingleInstructionResult::ControlInstruction(
                InstructionResult::BrIf,
            ))
        }
        Opcode::BrTable => {
            return Ok(SingleInstructionResult::ControlInstruction(
                InstructionResult::BrTable,
            ))
        }
        Opcode::Return => {
            return Ok(SingleInstructionResult::ControlInstruction(
                InstructionResult::Return,
            ))
        }
        Opcode::Call => {
            return Ok(SingleInstructionResult::ControlInstruction(
                InstructionResult::Call,
            ))
        }
        Opcode::CallIndirect => {
            return Ok(SingleInstructionResult::ControlInstruction(
                InstructionResult::CallIndirect,
            ))
        }

        Opcode::Drop => {
            // Probe the stack top to make sure there is a value there. We don't care what it is.
            get_stack_top(stack, 1)?;
            stack.pop();
        }
        Opcode::Select => {
            let selector = get_stack_top(stack, 1)?[0];
            let selector = i32::try_from(selector)?;
            stack.pop();

            let arguments = get_stack_top(stack, 2)?;
            if !arguments[0].is_same_type(&arguments[1]) {
                return Err(anyhow!("Select types do not match"));
            }
            let arguments = [arguments[0], arguments[1]];
            stack.pop_n(2);

            if selector == 0 {
                stack.push(arguments[1]);
            } else {
                stack.push(arguments[0]);
            }
        }

        Opcode::I32Load => {
            mem_load(instruction, stack, store, |v: u32| v)?;
        }
        Opcode::I64Load => {
            mem_load(instruction, stack, store, |v: u64| v)?;
        }
        Opcode::F32Load => {
            mem_load(instruction, stack, store, |v: f32| v)?;
        }
        Opcode::F64Load => {
            mem_load(instruction, stack, store, |v: f64| v)?;
        }

        Opcode::I32Load8S => {
            mem_load(instruction, stack, store, |v: i8| i32::from(v))?;
        }
        Opcode::I32Load8U => {
            mem_load(instruction, stack, store, |v: u8| u32::from(v))?;
        }
        Opcode::I32Load16S => {
            mem_load(instruction, stack, store, |v: i16| i32::from(v))?;
        }
        Opcode::I32Load16U => {
            mem_load(instruction, stack, store, |v: u16| u32::from(v))?;
        }
        Opcode::I64Load8S => {
            mem_load(instruction, stack, store, |v: i8| i64::from(v))?;
        }
        Opcode::I64Load8U => {
            mem_load(instruction, stack, store, |v: u8| u64::from(v))?;
        }
        Opcode::I64Load16S => {
            mem_load(instruction, stack, store, |v: i16| i64::from(v))?;
        }
        Opcode::I64Load16U => {
            mem_load(instruction, stack, store, |v: u16| u64::from(v))?;
        }
        Opcode::I64Load32S => {
            mem_load(instruction, stack, store, |v: i32| i64::from(v))?;
        }
        Opcode::I64Load32U => {
            mem_load(instruction, stack, store, |v: u32| u64::from(v))?;
        }

        Opcode::I32Store => {
            mem_store(instruction, stack, store, |v: u32| v)?;
        }
        Opcode::I64Store => {
            mem_store(instruction, stack, store, |v: u64| v)?;
        }
        Opcode::F32Store => {
            mem_store(instruction, stack, store, |v: f32| v)?;
        }
        Opcode::F64Store => {
            mem_store(instruction, stack, store, |v: f64| v)?;
        }

        Opcode::I32Store8 => {
            mem_store(instruction, stack, store, |v: u32| {
                u8::try_from(v & 0xff).unwrap()
            })?;
        }
        Opcode::I32Store16 => {
            mem_store(instruction, stack, store, |v: u32| {
                u16::try_from(v & 0xffff).unwrap()
            })?;
        }
        Opcode::I64Store8 => {
            mem_store(instruction, stack, store, |v: u64| {
                u8::try_from(v & 0xff).unwrap()
            })?;
        }
        Opcode::I64Store16 => {
            mem_store(instruction, stack, store, |v: u64| {
                u16::try_from(v & 0xffff).unwrap()
            })?;
        }
        Opcode::I64Store32 => {
            mem_store(instruction, stack, store, |v: u64| {
                u32::try_from(v & 0xffffffff).unwrap()
            })?;
        }

        Opcode::MemorySize => {
            let memory_idx = instruction.get_single_u32_as_usize_arg();
            let size = store.get_memory_size(memory_idx)? as u32;
            stack.push(size.into());
        }
        Opcode::MemoryGrow => {
            let memory_idx = instruction.get_single_u32_as_usize_arg();
            let original_size = store.get_memory_size(memory_idx)? as u32;

            let grow_by = get_stack_top(stack, 1)?[0];
            let grow_by = u32::try_from(grow_by)?;
            let grow_by = usize::try_from(grow_by).unwrap();
            stack.pop();

            if store.grow_memory_by(memory_idx, grow_by).is_ok() {
                stack.push(original_size.into());
            } else {
                stack.push(StackEntry::from(-1i32));
            }
        }

        Opcode::LocalGet => {
            let local_idx = instruction.get_single_u32_as_usize_arg();
            if local_idx >= stack.parameter_count() + stack.local_count() {
                return Err(anyhow!("Local index out of range"));
            }

            stack.push(stack.local()[local_idx]);
        }
        opcode @ Opcode::LocalSet | opcode @ Opcode::LocalTee => {
            let arg = get_stack_top(stack, 1)?[0];
            stack.pop();

            let local_idx = instruction.get_single_u32_as_usize_arg();
            if local_idx >= stack.parameter_count() + stack.local_count() {
                return Err(anyhow!("Local index out of range"));
            }

            stack.local_mut()[local_idx] = arg;

            if opcode == Opcode::LocalTee {
                stack.push(arg);
            }
        }
        Opcode::GlobalGet => {
            stack.push(store.get_global_value(instruction.get_single_u32_as_usize_arg())?)
        }
        Opcode::GlobalSet => {
            let arg = get_stack_top(stack, 1)?[0];
            stack.pop();

            store.set_global_value(instruction.get_single_u32_as_usize_arg(), arg)?;
        }

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

        Opcode::I32Eqz => unary_boolean_op(stack, |a: u32| a == 0)?,
        Opcode::I32Eq => binary_boolean_op(stack, |a: u32, b| a == b)?,
        Opcode::I32Ne => binary_boolean_op(stack, |a: u32, b| a != b)?,
        Opcode::I32LtS => binary_boolean_op(stack, |a: i32, b| a < b)?,
        Opcode::I32LtU => binary_boolean_op(stack, |a: u32, b| a < b)?,
        Opcode::I32GtS => binary_boolean_op(stack, |a: i32, b| a > b)?,
        Opcode::I32GtU => binary_boolean_op(stack, |a: u32, b| a > b)?,
        Opcode::I32LeS => binary_boolean_op(stack, |a: i32, b| a <= b)?,
        Opcode::I32LeU => binary_boolean_op(stack, |a: u32, b| a <= b)?,
        Opcode::I32GeS => binary_boolean_op(stack, |a: i32, b| a >= b)?,
        Opcode::I32GeU => binary_boolean_op(stack, |a: u32, b| a >= b)?,

        Opcode::I64Eqz => unary_boolean_op(stack, |a: u64| a == 0)?,
        Opcode::I64Eq => binary_boolean_op(stack, |a: u64, b| a == b)?,
        Opcode::I64Ne => binary_boolean_op(stack, |a: u64, b| a != b)?,
        Opcode::I64LtS => binary_boolean_op(stack, |a: i64, b| a < b)?,
        Opcode::I64LtU => binary_boolean_op(stack, |a: u64, b| a < b)?,
        Opcode::I64GtS => binary_boolean_op(stack, |a: i64, b| a > b)?,
        Opcode::I64GtU => binary_boolean_op(stack, |a: u64, b| a > b)?,
        Opcode::I64LeS => binary_boolean_op(stack, |a: i64, b| a <= b)?,
        Opcode::I64LeU => binary_boolean_op(stack, |a: u64, b| a <= b)?,
        Opcode::I64GeS => binary_boolean_op(stack, |a: i64, b| a >= b)?,
        Opcode::I64GeU => binary_boolean_op(stack, |a: u64, b| a >= b)?,

        Opcode::F32Eq => binary_boolean_op(stack, |a: f32, b| a == b)?,
        Opcode::F32Ne => binary_boolean_op(stack, |a: f32, b| a != b)?,
        Opcode::F32Lt => binary_boolean_op(stack, |a: f32, b| a < b)?,
        Opcode::F32Gt => binary_boolean_op(stack, |a: f32, b| a > b)?,
        Opcode::F32Le => binary_boolean_op(stack, |a: f32, b| a <= b)?,
        Opcode::F32Ge => binary_boolean_op(stack, |a: f32, b| a >= b)?,

        Opcode::F64Eq => binary_boolean_op(stack, |a: f64, b| a == b)?,
        Opcode::F64Ne => binary_boolean_op(stack, |a: f64, b| a != b)?,
        Opcode::F64Lt => binary_boolean_op(stack, |a: f64, b| a < b)?,
        Opcode::F64Gt => binary_boolean_op(stack, |a: f64, b| a > b)?,
        Opcode::F64Le => binary_boolean_op(stack, |a: f64, b| a <= b)?,
        Opcode::F64Ge => binary_boolean_op(stack, |a: f64, b| a >= b)?,

        Opcode::I32Clz => unary_op(stack, |a: u32| u32::from(a.leading_zeros()))?,
        Opcode::I32Ctz => unary_op(stack, |a: u32| u32::from(a.trailing_zeros()))?,
        Opcode::I32Popcnt => unary_op(stack, |a: u32| u32::from(a.count_ones()))?,
        Opcode::I32Add => binary_op(stack, |a: u32, b| a.wrapping_add(b))?,
        Opcode::I32Sub => binary_op(stack, |a: u32, b| a.wrapping_sub(b))?,
        Opcode::I32Mul => binary_op(stack, |a: u32, b| a.wrapping_mul(b))?,
        Opcode::I32DivS => binary_op(stack, |a: i32, b| a.wrapping_div(b))?,
        Opcode::I32DivU => binary_op(stack, |a: u32, b| a.wrapping_div(b))?,
        Opcode::I32RemS => binary_op(stack, |a: i32, b| a.wrapping_rem(b))?,
        Opcode::I32RemU => binary_op(stack, |a: u32, b| a.wrapping_rem(b))?,
        Opcode::I32And => binary_op(stack, |a: u32, b: u32| a & b)?,
        Opcode::I32Or => binary_op(stack, |a: u32, b: u32| a | b)?,
        Opcode::I32Xor => binary_op(stack, |a: u32, b: u32| a ^ b)?,
        Opcode::I32Shl => binary_op(stack, |a: u32, b: u32| a << (b % 32))?,
        Opcode::I32ShrS => binary_op(stack, |a: i32, b: i32| a >> (b % 32))?,
        Opcode::I32ShrU => binary_op(stack, |a: u32, b: u32| a >> (b % 32))?,
        Opcode::I32Rotl => binary_op(stack, |a: u32, b: u32| a.rotate_left(b % 32))?,
        Opcode::I32Rotr => binary_op(stack, |a: u32, b: u32| a.rotate_right(b % 32))?,

        Opcode::I64Clz => unary_op(stack, |a: u64| u64::from(a.leading_zeros()))?,
        Opcode::I64Ctz => unary_op(stack, |a: u64| u64::from(a.trailing_zeros()))?,
        Opcode::I64Popcnt => unary_op(stack, |a: u64| u64::from(a.count_ones()))?,
        Opcode::I64Add => binary_op(stack, |a: u64, b| a.wrapping_add(b))?,
        Opcode::I64Sub => binary_op(stack, |a: u64, b| a.wrapping_sub(b))?,
        Opcode::I64Mul => binary_op(stack, |a: u64, b| a.wrapping_mul(b))?,
        Opcode::I64DivS => binary_op(stack, |a: i64, b| a.wrapping_div(b))?,
        Opcode::I64DivU => binary_op(stack, |a: u64, b| a.wrapping_div(b))?,
        Opcode::I64RemS => binary_op(stack, |a: i64, b| a.wrapping_rem(b))?,
        Opcode::I64RemU => binary_op(stack, |a: u64, b| a.wrapping_rem(b))?,
        Opcode::I64And => binary_op(stack, |a: u64, b: u64| a & b)?,
        Opcode::I64Or => binary_op(stack, |a: u64, b: u64| a | b)?,
        Opcode::I64Xor => binary_op(stack, |a: u64, b: u64| a ^ b)?,
        Opcode::I64Shl => binary_op(stack, |a: u64, b: u64| a << (b % 32))?,
        Opcode::I64ShrS => binary_op(stack, |a: i64, b: i64| a >> (b % 32))?,
        Opcode::I64ShrU => binary_op(stack, |a: u64, b: u64| a >> (b % 32))?,
        Opcode::I64Rotl => binary_op(stack, |a: u64, b: u64| {
            a.rotate_left(u32::try_from(b % 32).unwrap())
        })?,
        Opcode::I64Rotr => binary_op(stack, |a: u64, b: u64| {
            a.rotate_right(u32::try_from(b % 32).unwrap())
        })?,

        Opcode::F32Abs => unary_op(stack, |a: f32| a.abs())?,
        Opcode::F32Neg => unary_op(stack, |a: f32| -a)?,
        Opcode::F32Ceil => unary_op(stack, |a: f32| a.ceil())?,
        Opcode::F32Floor => unary_op(stack, |a: f32| a.floor())?,
        Opcode::F32Trunc => unary_op(stack, |a: f32| a.trunc())?,
        Opcode::F32Nearest => unary_op(stack, |a: f32| a.round())?,
        Opcode::F32Sqrt => unary_op(stack, |a: f32| a.sqrt())?,
        Opcode::F32Add => binary_op(stack, |a: f32, b: f32| a + b)?,
        Opcode::F32Sub => binary_op(stack, |a: f32, b: f32| a - b)?,
        Opcode::F32Mul => binary_op(stack, |a: f32, b: f32| a * b)?,
        Opcode::F32Div => binary_op(stack, |a: f32, b: f32| a / b)?,
        Opcode::F32Min => binary_op(stack, |a: f32, b: f32| a.min(b))?,
        Opcode::F32Max => binary_op(stack, |a: f32, b: f32| a.max(b))?,
        Opcode::F32CopySign => binary_op(stack, |a: f32, b: f32| a.copysign(b))?,

        Opcode::F64Abs => unary_op(stack, |a: f64| a.abs())?,
        Opcode::F64Neg => unary_op(stack, |a: f64| -a)?,
        Opcode::F64Ceil => unary_op(stack, |a: f64| a.ceil())?,
        Opcode::F64Floor => unary_op(stack, |a: f64| a.floor())?,
        Opcode::F64Trunc => unary_op(stack, |a: f64| a.trunc())?,
        Opcode::F64Nearest => unary_op(stack, |a: f64| a.round())?,
        Opcode::F64Sqrt => unary_op(stack, |a: f64| a.sqrt())?,
        Opcode::F64Add => binary_op(stack, |a: f64, b: f64| a + b)?,
        Opcode::F64Sub => binary_op(stack, |a: f64, b: f64| a - b)?,
        Opcode::F64Mul => binary_op(stack, |a: f64, b: f64| a * b)?,
        Opcode::F64Div => binary_op(stack, |a: f64, b: f64| a / b)?,
        Opcode::F64Min => binary_op(stack, |a: f64, b: f64| a.min(b))?,
        Opcode::F64Max => binary_op(stack, |a: f64, b: f64| a.max(b))?,
        Opcode::F64CopySign => binary_op(stack, |a: f64, b: f64| a.copysign(b))?,

        Opcode::I32WrapI64 => unary_op(stack, |a: u64| a as u32)?,
        Opcode::I32TruncF32S => unary_op(stack, |a: f32| a as i32)?,
        Opcode::I32TruncF32U => unary_op(stack, |a: f32| a as u32)?,
        Opcode::I32TruncF64S => unary_op(stack, |a: f64| a as i32)?,
        Opcode::I32TruncF64U => unary_op(stack, |a: f64| a as u32)?,
        Opcode::I64ExtendI32S => unary_op(stack, |a: i32| a as i64)?,
        Opcode::I64ExtendI32U => unary_op(stack, |a: u32| a as u64)?,
        Opcode::I64TruncF32S => unary_op(stack, |a: f32| a as i64)?,
        Opcode::I64TruncF32U => unary_op(stack, |a: f32| a as u64)?,
        Opcode::I64TruncF64S => unary_op(stack, |a: f64| a as i64)?,
        Opcode::I64TruncF64U => unary_op(stack, |a: f64| a as u64)?,
        Opcode::F32ConvertI32S => unary_op(stack, |a: i32| a as f32)?,
        Opcode::F32ConvertI32U => unary_op(stack, |a: u32| a as f32)?,
        Opcode::F32ConvertI64S => unary_op(stack, |a: i64| a as f32)?,
        Opcode::F32ConvertI64U => unary_op(stack, |a: u64| a as f32)?,
        Opcode::F32DemoteF64 => unary_op(stack, |a: f64| a as f32)?,
        Opcode::F64ConvertI32S => unary_op(stack, |a: i32| a as f64)?,
        Opcode::F64ConvertI32U => unary_op(stack, |a: u32| a as f64)?,
        Opcode::F64ConvertI64S => unary_op(stack, |a: i64| a as f64)?,
        Opcode::F64ConvertI64U => unary_op(stack, |a: u64| a as f64)?,
        Opcode::F64PromoteF32 => unary_op(stack, |a: f32| a as f64)?,
        Opcode::I32ReinterpretF32 => {
            unary_op(stack, |a: f32| -> u32 { unsafe { std::mem::transmute(a) } })?
        }
        Opcode::I64ReinterpretF64 => {
            unary_op(stack, |a: f64| -> u64 { unsafe { std::mem::transmute(a) } })?
        }
        Opcode::F32ReinterpretI32 => {
            unary_op(stack, |a: i32| -> f32 { unsafe { std::mem::transmute(a) } })?
        }
        Opcode::F64ReinterpretI64 => {
            unary_op(stack, |a: i64| -> f64 { unsafe { std::mem::transmute(a) } })?
        }
    }

    Ok(SingleInstructionResult::Done)
}

pub fn execute_constant_expression(
    expr: &impl InstructionSource,
    stack: &mut Stack,
    store: &impl ConstantExpressionStore,
) -> Result<()> {
    for instruction in expr.iter() {
        execute_single_constant_instruction(instruction?, stack, store)?;
    }
    Ok(())
}

pub fn evaluate_constant_expression(
    expr: &impl InstructionSource,
    store: &impl ConstantExpressionStore,
    arity: usize,
) -> Result<Vec<StackEntry>> {
    let mut stack = Stack::new();

    execute_constant_expression(expr, &mut stack, store)?;

    if stack.working_count() < arity {
        return Err(anyhow!("Not enough values returned by constant expression"));
    }

    Ok(stack.frame()[stack.working_limit() - arity..stack.working_limit()].to_vec())
}

fn execute_inner_loop<'a>(
    iter: &'_ mut impl Iterator<Item = Result<Instruction<'a>>>,
    stack: &'_ mut Stack,
    store: &'_ mut impl ExpressionStore,
) -> Option<Result<(InstructionResult, Instruction<'a>)>> {
    loop {
        match iter.next() {
            None => {
                return None;
            }
            Some(Err(e)) => {
                return Some(Err(e));
            }
            Some(Ok(instruction)) => {
                match execute_single_instruction(&instruction, stack, store) {
                    Ok(SingleInstructionResult::Done) => {} // Normal instruction executed normally
                    Ok(SingleInstructionResult::ControlInstruction(ir)) => {
                        return Some(Ok((ir, instruction)));
                    }
                    Err(e) => {
                        return Some(Err(e));
                    }
                }
            }
        }
    }
}

struct BranchControl {
    is_branch: bool,
    label_idx: usize,
    label_cnt: usize,
}

impl BranchControl {
    pub fn no_branch() -> Self {
        BranchControl {
            is_branch: false,
            label_idx: 0,
            label_cnt: 0,
        }
    }

    pub fn branch_target(label_idx: usize) -> Self {
        BranchControl {
            is_branch: true,
            label_idx,
            label_cnt: label_idx,
        }
    }
}

fn execute_block_expression(
    block_type: BlockType,
    is_loop: bool,
    expr: &(impl InstructionSource + ?Sized),
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
) -> Result<BranchControl> {
    loop {
        // Push a label on to the stack. This is mainly used as a stack guard, since we will probably
        // end up using the rust stack to handle actual branching. Loops ignore the return type, even
        // though it is present in the grammar
        let block_arity = if is_loop || block_type == BlockType::None {
            0
        } else {
            1
        };
        stack.push_label(block_arity);

        // Now execute the expression
        let branch_control = execute_expression_internal(expr, stack, store)?;

        if !branch_control.is_branch || branch_control.label_idx == 0 {
            // Either this isn't a branch, or this is a branch to here. We have to
            // distinguish between no branch, and branch to zero because of the loop
            // instruction where we behave differently.

            // Walk all of the labels back off the stack. We add one to account for the lable we're
            // going to
            stack.pop_n_labels(branch_control.label_cnt + 1);

            // If this is not a loop, then return no branch to indicate we're done, otherwise go around
            // the loop again
            if !branch_control.is_branch || !is_loop {
                return Ok(BranchControl::no_branch());
            }
        } else {
            return Ok(BranchControl {
                is_branch: true,
                label_idx: branch_control.label_idx - 1,
                label_cnt: branch_control.label_cnt,
            });
        }
    }
}

fn execute_if<'a>(
    instruction: &'a Instruction<'a>,
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
) -> Result<BranchControl> {
    let condition = u32::try_from(get_stack_top(stack, 1)?[0])?;
    stack.pop();

    if condition != 0 {
        execute_block_expression(
            instruction.get_block_type(),
            false,
            instruction.get_block(),
            stack,
            store,
        )
    } else if instruction.has_else_block() {
        execute_block_expression(
            instruction.get_block_type(),
            false,
            instruction.get_else_block(),
            stack,
            store,
        )
    } else if instruction.get_block_type() != BlockType::None {
        Err(anyhow!("If instruction with block type other than none should have an else block (shouldn't it?)"))
    } else {
        Ok(BranchControl::no_branch())
    }
}

fn execute_block<'a>(
    instruction: &'a Instruction<'a>,
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
) -> Result<BranchControl> {
    execute_block_expression(
        instruction.get_block_type(),
        instruction.opcode() == Opcode::Loop,
        instruction.get_block(),
        stack,
        store,
    )
}

fn execute_br(
    label: usize,
    _stack: &mut Stack,
    _store: &mut impl ExpressionStore,
) -> Result<BranchControl> {
    // Taking the branch is pretty easy
    Ok(BranchControl::branch_target(label))
}

fn execute_br_if(
    label: usize,
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
) -> Result<BranchControl> {
    let condition = u32::try_from(get_stack_top(stack, 1)?[0])?;
    stack.pop();

    if condition != 0 {
        Ok(execute_br(label, stack, store)?)
    } else {
        Ok(BranchControl::no_branch())
    }
}

fn execute_br_table(
    labels: &[usize],
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
) -> Result<BranchControl> {
    assert!(labels.len() > 0);

    let index = u32::try_from(get_stack_top(stack, 1)?[0])?;
    let index = usize::try_from(index).unwrap();
    stack.pop();

    let index = std::cmp::min(index, labels.len() - 1);
    execute_br(labels[index], stack, store)
}

fn execute_call(
    idx: usize,
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
) -> Result<BranchControl> {
    let callable = store.callable_idx(idx)?.clone();
    callable.call(stack, store)?;
    Ok(BranchControl::no_branch())
}

fn execute_expression_internal(
    expr: &(impl InstructionSource + ?Sized),
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
) -> Result<BranchControl> {
    let mut iter = expr.iter();
    loop {
        let branch_control = match execute_inner_loop(&mut iter, stack, store) {
            None => {
                return Ok(BranchControl::no_branch());
            }
            Some(Err(e)) => {
                return Err(e);
            }

            Some(Ok((InstructionResult::If, instruction))) => {
                execute_if(&instruction, stack, store)?
            }
            Some(Ok((InstructionResult::Block, instruction)))
            | Some(Ok((InstructionResult::Loop, instruction))) => {
                execute_block(&instruction, stack, store)?
            }

            Some(Ok((InstructionResult::Br, instruction))) => {
                execute_br(instruction.get_single_u32_as_usize_arg(), stack, store)?
            }
            Some(Ok((InstructionResult::BrIf, instruction))) => {
                execute_br_if(instruction.get_single_u32_as_usize_arg(), stack, store)?
            }
            Some(Ok((InstructionResult::BrTable, instruction))) => {
                execute_br_table(&instruction.get_block_table_targets(), stack, store)?
            }

            Some(Ok((InstructionResult::Call, instruction))) => {
                execute_call(instruction.get_single_u32_as_usize_arg(), stack, store)?
            }
            Some(Ok((InstructionResult::CallIndirect, _))) => {
                unimplemented!("Call not implemented")
            }
            Some(Ok((InstructionResult::Return, _))) => unimplemented!("Call not implemented"),
        };

        // If we're branching, then propagate the branch to the caller
        if branch_control.is_branch {
            return Ok(branch_control);
        }
    }
}

pub fn execute_expression(
    expr: &(impl InstructionSource + ?Sized),
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
) -> Result<()> {
    execute_expression_internal(expr, stack, store)?;
    Ok(())
}
