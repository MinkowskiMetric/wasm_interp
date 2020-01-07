use std::convert::{TryFrom, TryInto};

use crate::core::{stack_entry::StackEntry, Stack};
use crate::parser::{self, Opcode};
use anyhow::{anyhow, Result};

fn get_stack_top(stack: &mut Stack, n: usize) -> Result<&[StackEntry]> {
    if stack.working_count() < n {
        Err(anyhow!("Not enough values on stack"))
    } else {
        Ok(stack.working_top(n))
    }
}

fn unary_op<
    ParamType: Sized + TryFrom<StackEntry, Error = anyhow::Error>,
    RetType: Into<StackEntry>,
    Func: Fn(ParamType) -> RetType,
>(
    stack: &mut Stack,
    func: Func,
) -> Result<()> {
    let arg = get_stack_top(stack, 1)?[0];
    stack.pop();

    let ret = func(arg.try_into()?);
    stack.push(ret.into());
    Ok(())
}

fn binary_op<
    ParamType: Sized + TryFrom<StackEntry, Error = anyhow::Error>,
    RetType: Into<StackEntry>,
    Func: Fn(ParamType, ParamType) -> RetType,
>(
    stack: &mut Stack,
    func: Func,
) -> Result<()> {
    let args = get_stack_top(stack, 2)?;
    let args = [args[0], args[1]];
    stack.pop_n(2);

    let ret = func(args[0].try_into()?, args[1].try_into()?);
    stack.push(ret.into());
    Ok(())
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

    pub fn execute<ExprType: parser::InstructionSource, StoreType: ExpressionStore>(
        &self,
        expr: &ExprType,
        stack: &mut Stack,
        store: &mut StoreType,
    ) -> Result<()> {
        for instruction in expr.iter() {
            let instruction = instruction?;

            match instruction.opcode() {
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
                Opcode::LocalGet => {
                    let local_idx = instruction.get_single_u32_as_usize_arg();
                    if local_idx >= stack.local_count() {
                        return Err(anyhow!("Local index out of range"));
                    }

                    stack.push(stack.local()[local_idx]);
                }
                opcode @ Opcode::LocalSet | opcode @ Opcode::LocalTee => {
                    let arg = get_stack_top(stack, 1)?[0];
                    stack.pop();

                    let local_idx = instruction.get_single_u32_as_usize_arg();
                    if local_idx >= stack.local_count() {
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

                Opcode::F32Add => binary_op(stack, |a: f32, b: f32| a + b)?,

                Opcode::F64Add => binary_op(stack, |a: f64, b: f64| a + b)?,

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

#[cfg(test)]
mod test {
    use super::*;

    use std::convert::TryInto;

    struct TestStore {}

    impl TestStore {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl ConstantExpressionStore for TestStore {
        fn get_global_value(&self, _idx: usize) -> Result<StackEntry> {
            Err(anyhow!("Global value not present in test store"))
        }
    }

    impl ExpressionStore for TestStore {
        fn set_global_value(&mut self, _idx: usize, _value: StackEntry) -> Result<()> {
            Err(anyhow!("Global value not present in test store"))
        }
    }

    fn write_leb(expr_bytes: &mut Vec<u8>, val: u64, signed: bool) {
        let mut encoded_bytes: [u8; 10] = [
            (0x80 | (val & 0x7f)).try_into().unwrap(),
            (0x80 | ((val >> 7) & 0x7f)).try_into().unwrap(),
            (0x80 | ((val >> 14) & 0x7f)).try_into().unwrap(),
            (0x80 | ((val >> 21) & 0x7f)).try_into().unwrap(),
            (0x80 | ((val >> 28) & 0x7f)).try_into().unwrap(),
            (0x80 | ((val >> 35) & 0x7f)).try_into().unwrap(),
            (0x80 | ((val >> 42) & 0x7f)).try_into().unwrap(),
            (0x80 | ((val >> 49) & 0x7f)).try_into().unwrap(),
            (0x80 | ((val >> 56) & 0x7f)).try_into().unwrap(),
            ((val >> 63) & 0x7f).try_into().unwrap(),
        ];

        let is_positive = !signed || 0 == (val & 0x8000000000000000);

        let mut required_length: usize = 10;
        while required_length > 1 {
            let last_byte = encoded_bytes[required_length - 1];
            let penultimate_byte = encoded_bytes[required_length - 2];

            // check that the signal bits are set the way we expect
            assert!((last_byte & 0x80) == 0);
            assert!((penultimate_byte & 0x80) == 0x80);

            // We have to check the high bit of the previous byte as we
            // scan because it has to match the bits we're dropping, otherwise
            // when it gets sign extended it will go wrong
            let can_drop_byte = if is_positive {
                last_byte == 0x00 && (penultimate_byte & 0xC0) == 0x80
            } else if required_length == 10 {
                // We have a special case for the highest byte of a negative number just to cope with the
                // single set bit
                last_byte == 0x01 && (penultimate_byte & 0xC0) == 0xC0
            } else {
                last_byte == 0x7F && (penultimate_byte & 0xC0) == 0xC0
            };

            if can_drop_byte {
                // If we can drop the byte, then we can decrement the required length and
                // clear the top bit of the new last byte
                encoded_bytes[required_length - 2] = penultimate_byte & 0x7f;
                required_length -= 1;
            } else {
                // Otherwise, break out because we can't drop this byte
                break;
            }
        }

        // Write the bytes out as is
        expr_bytes.extend_from_slice(&encoded_bytes[0..required_length]);
    }

    fn write_leb_as_vector(i: u64, signed: bool) -> Vec<u8> {
        let mut vec = Vec::new();
        write_leb(&mut vec, i, signed);
        vec
    }

    fn write_signed_leb_as_vector(i: u64) -> Vec<u8> {
        write_leb_as_vector(i, true)
    }

    #[test]
    fn test_leb_writer() {
        assert_eq!(write_signed_leb_as_vector(0), [0x00]);
        assert_eq!(write_signed_leb_as_vector(1), [0x01]);
        assert_eq!(write_signed_leb_as_vector(0x80), [0x80, 0x01]);
        assert_eq!(write_signed_leb_as_vector(0xFF), [0xFF, 0x01]);
        assert_eq!(write_signed_leb_as_vector(0xFFFF), [0xFF, 0xFF, 0x03]);
        assert_eq!(
            write_signed_leb_as_vector(unsafe { std::mem::transmute(-1i64) }),
            [0x7F]
        );
        assert_eq!(
            write_signed_leb_as_vector(unsafe { std::mem::transmute(-2i64) }),
            [0x7E]
        );
        assert_eq!(
            write_signed_leb_as_vector(unsafe { std::mem::transmute(-256i64) }),
            [0x80, 0x7E]
        );
        assert_eq!(
            write_signed_leb_as_vector(unsafe { std::mem::transmute(-65536i64) }),
            [0x80, 0x80, 0x7C]
        );
    }

    fn write_const_instruction(expr_bytes: &mut Vec<u8>, val: StackEntry) {
        match val {
            StackEntry::I32Entry(i) => {
                expr_bytes.push(Opcode::I32Const.into());
                write_leb(expr_bytes, i.into(), true);
            }
            StackEntry::I64Entry(i) => {
                expr_bytes.push(Opcode::I64Const.into());
                write_leb(expr_bytes, i.into(), true);
            }
            StackEntry::F32Entry(i) => {
                expr_bytes.push(Opcode::F32Const.into());
                expr_bytes.extend_from_slice(&i.to_le_bytes());
            }
            StackEntry::F64Entry(i) => {
                expr_bytes.push(Opcode::F64Const.into());
                expr_bytes.extend_from_slice(&i.to_le_bytes());
            }

            _ => panic!("Unsupported stack entry type"),
        }
    }

    fn test_constant_opcode_impl(p1: StackEntry) -> Option<StackEntry> {
        // Allocate a byte vector and generate an instruction stream that will execute the op
        let mut expr_bytes: Vec<u8> = Vec::new();
        write_const_instruction(&mut expr_bytes, p1);

        // Now we need a stack and a store to run the op against
        let mut stack = Stack::new();
        let mut test_store = TestStore::new();

        if let Err(_) =
            ExpressionExecutor::instance().execute(&expr_bytes, &mut stack, &mut test_store)
        {
            None
        } else {
            if stack.working_count() == 1 {
                Some(stack.working_top(1)[0])
            } else {
                None
            }
        }
    }

    macro_rules! test_constant_opcode {
        ($p1:expr) => {
            assert_eq!(test_constant_opcode_impl($p1.into()), Some($p1.into()));
        };
    }

    fn test_unary_opcode_impl(p1: StackEntry, opcode: Opcode) -> Option<StackEntry> {
        // Allocate a byte vector and generate an instruction stream that will execute the op
        let mut expr_bytes: Vec<u8> = Vec::new();
        write_const_instruction(&mut expr_bytes, p1);
        expr_bytes.push(opcode.into());

        // Now we need a stack and a store to run the op against
        let mut stack = Stack::new();
        let mut test_store = TestStore::new();

        if let Err(_) =
            ExpressionExecutor::instance().execute(&expr_bytes, &mut stack, &mut test_store)
        {
            None
        } else {
            if stack.working_count() == 1 {
                Some(stack.working_top(1)[0])
            } else {
                None
            }
        }
    }

    macro_rules! test_unary_opcode {
        ($p1:expr, $opcode:expr, $r:expr) => {
            assert_eq!(
                test_unary_opcode_impl($p1.into(), $opcode.into()),
                Some($r.into())
            );
        };
    }

    fn test_binary_opcode_impl(
        p1: StackEntry,
        p2: StackEntry,
        opcode: Opcode,
    ) -> Option<StackEntry> {
        // Allocate a byte vector and generate an instruction stream that will execute the op
        let mut expr_bytes: Vec<u8> = Vec::new();
        write_const_instruction(&mut expr_bytes, p1);
        write_const_instruction(&mut expr_bytes, p2);
        expr_bytes.push(opcode.into());

        // Now we need a stack and a store to run the op against
        let mut stack = Stack::new();
        let mut test_store = TestStore::new();

        if let Err(_) =
            ExpressionExecutor::instance().execute(&expr_bytes, &mut stack, &mut test_store)
        {
            None
        } else {
            if stack.working_count() == 1 {
                Some(stack.working_top(1)[0])
            } else {
                None
            }
        }
    }

    macro_rules! test_binary_opcode {
        ($p1:expr, $p2:expr, $opcode:expr, $r:expr) => {
            assert_eq!(
                test_binary_opcode_impl($p1.into(), $p2.into(), $opcode),
                Some($r.into())
            );
        };
    }

    #[test]
    fn test_drop() {
        // Allocate a byte vector and generate an instruction stream that will execute the op
        let mut expr_bytes: Vec<u8> = Vec::new();
        write_const_instruction(&mut expr_bytes, 42i32.into());
        write_const_instruction(&mut expr_bytes, 42.0f64.into());
        expr_bytes.push(Opcode::Drop.into());

        // Now we need a stack and a store to run the op against
        let mut stack = Stack::new();
        let mut test_store = TestStore::new();

        assert!(ExpressionExecutor::instance()
            .execute(&expr_bytes, &mut stack, &mut test_store)
            .is_ok());

        assert_eq!(stack.working_count(), 1);
        assert_eq!(stack.working_top(1)[0], 42i32.into());
    }

    #[test]
    fn test_select() {
        // Allocate a byte vector and generate an instruction stream that will execute the op
        let mut expr_bytes: Vec<u8> = Vec::new();
        write_const_instruction(&mut expr_bytes, 42i32.into());
        write_const_instruction(&mut expr_bytes, 42.0f64.into());
        write_const_instruction(&mut expr_bytes, 1i32.into());
        expr_bytes.push(Opcode::Select.into());

        // Now we need a stack and a store to run the op against
        let mut stack = Stack::new();
        let mut test_store = TestStore::new();

        assert!(ExpressionExecutor::instance()
            .execute(&expr_bytes, &mut stack, &mut test_store)
            .is_err());

        // It should have failed part way through the instruction, leaving two operands on the stack
        assert_eq!(stack.working_count(), 2);
        assert_eq!(stack.working_top(2)[0], 42i32.into());
        assert_eq!(stack.working_top(2)[1], 42.0f64.into());

        stack.pop_n(2);

        expr_bytes.clear();
        write_const_instruction(&mut expr_bytes, 42i32.into());
        write_const_instruction(&mut expr_bytes, 69i32.into());
        write_const_instruction(&mut expr_bytes, 1i32.into());
        expr_bytes.push(Opcode::Select.into());

        assert!(ExpressionExecutor::instance()
            .execute(&expr_bytes, &mut stack, &mut test_store)
            .is_ok());

        assert_eq!(stack.working_count(), 1);
        assert_eq!(stack.working_top(1)[0], 42i32.into());

        stack.pop();

        expr_bytes.clear();
        write_const_instruction(&mut expr_bytes, 42i32.into());
        write_const_instruction(&mut expr_bytes, 69i32.into());
        write_const_instruction(&mut expr_bytes, 0i32.into());
        expr_bytes.push(Opcode::Select.into());

        assert!(ExpressionExecutor::instance()
            .execute(&expr_bytes, &mut stack, &mut test_store)
            .is_ok());

        assert_eq!(stack.working_count(), 1);
        assert_eq!(stack.working_top(1)[0], 69i32.into());
    }

    #[test]
    fn test_opcodes() {
        test_constant_opcode!(0i32);
        test_constant_opcode!(1i32);
        test_constant_opcode!(2i32);
        test_constant_opcode!(-1i32);
        test_constant_opcode!(-256i32);
        test_constant_opcode!(-65536i32);
        test_constant_opcode!(0u32);
        test_constant_opcode!(1u32);
        test_constant_opcode!(2u32);
        test_constant_opcode!(256u32);
        test_constant_opcode!(0i64);
        test_constant_opcode!(0u64);
        test_constant_opcode!(0.0f32);
        test_constant_opcode!(0.0f64);
        // TODOTODOTODO - should test for integers that are too big for the opcodes to make sure they're handled properly.
        // I haven't written that test because currently it panics in the instruction accumulator which is obviously not the
        // right thing to do.

        test_unary_opcode!(7i32, Opcode::I32Clz, 29u32);
        test_unary_opcode!(7i32, Opcode::I32Ctz, 0u32);
        test_unary_opcode!(7i32, Opcode::I32Popcnt, 3u32);
        test_binary_opcode!(7i32, 8i32, Opcode::I32Add, 15u32);
        test_binary_opcode!(7i32, -1i32, Opcode::I32Add, 6u32);
        test_binary_opcode!(7i32, -1i32, Opcode::I32Sub, 8u32);
        test_binary_opcode!(-1i32, 7i32, Opcode::I32Sub, -8i32);
        test_binary_opcode!(7i32, -4i32, Opcode::I32Mul, -28i32);
        test_binary_opcode!(7i32, -4i32, Opcode::I32DivS, -1i32);
        test_binary_opcode!(7i32, -4i32, Opcode::I32DivU, 0i32);
        test_binary_opcode!(7i32, -4i32, Opcode::I32RemS, 3i32);
        test_binary_opcode!(7i32, -4i32, Opcode::I32RemU, 7i32);
        test_binary_opcode!(7i32, 3i32, Opcode::I32And, 3i32);
        test_binary_opcode!(7i32, 15i32, Opcode::I32Or, 15i32);
        test_binary_opcode!(7i32, 2i32, Opcode::I32Xor, 5i32);
        test_binary_opcode!(0x00000080u32, 2u32, Opcode::I32Shl, 0x00000200u32);
        test_binary_opcode!(0x80000000u32, 2u32, Opcode::I32ShrU, 0x20000000u32);
        test_binary_opcode!(0x80000000u32, 2u32, Opcode::I32ShrS, 0xE0000000u32);
        test_binary_opcode!(0x40000000u32, 2u32, Opcode::I32Rotl, 0x00000001u32);
        test_binary_opcode!(0x00000002u32, 2u32, Opcode::I32Rotr, 0x80000000u32);

        test_unary_opcode!(7i64, Opcode::I64Clz, 61u64);
        test_unary_opcode!(7i64, Opcode::I64Ctz, 0u64);
        test_unary_opcode!(7i64, Opcode::I64Popcnt, 3u64);
        test_binary_opcode!(7i64, 8i64, Opcode::I64Add, 15u64);
        test_binary_opcode!(7i64, -1i64, Opcode::I64Add, 6u64);
        test_binary_opcode!(7i64, -1i64, Opcode::I64Sub, 8u64);
        test_binary_opcode!(-1i64, 7i64, Opcode::I64Sub, -8i64);
        test_binary_opcode!(7i64, -4i64, Opcode::I64Mul, -28i64);
        test_binary_opcode!(7i64, -4i64, Opcode::I64DivS, -1i64);
        test_binary_opcode!(7i64, -4i64, Opcode::I64DivU, 0i64);
        test_binary_opcode!(7i64, -4i64, Opcode::I64RemS, 3i64);
        test_binary_opcode!(7i64, -4i64, Opcode::I64RemU, 7i64);
        test_binary_opcode!(7i64, 3i64, Opcode::I64And, 3i64);
        test_binary_opcode!(7i64, 15i64, Opcode::I64Or, 15i64);
        test_binary_opcode!(7i64, 2i64, Opcode::I64Xor, 5i64);
        test_binary_opcode!(
            0x0000000000000080u64,
            2u64,
            Opcode::I64Shl,
            0x0000000000000200u64
        );
        test_binary_opcode!(
            0x8000000000000000u64,
            2u64,
            Opcode::I64ShrU,
            0x2000000000000000u64
        );
        test_binary_opcode!(
            0x8000000000000000u64,
            2u64,
            Opcode::I64ShrS,
            0xE000000000000000u64
        );
        test_binary_opcode!(
            0x4000000000000000u64,
            2u64,
            Opcode::I64Rotl,
            0x0000000000000001u64
        );
        test_binary_opcode!(
            0x0000000000000002u64,
            2u64,
            Opcode::I64Rotr,
            0x8000000000000000u64
        );

        test_binary_opcode!(7.0f32, 8.0f32, Opcode::F32Add, 15.0f32);
        test_binary_opcode!(7.0f32, -1.0f32, Opcode::F32Add, 6.0f32);

        test_binary_opcode!(7.0f64, 8.0f64, Opcode::F64Add, 15.0f64);
        test_binary_opcode!(7.0f64, -1.0f64, Opcode::F64Add, 6.0f64);
    }

    fn do_local_get<Store: ExpressionStore>(
        stack: &mut Stack,
        store: &mut Store,
        index: u32,
    ) -> Option<StackEntry> {
        let mut expr_bytes = Vec::new();
        expr_bytes.push(Opcode::LocalGet.into());
        write_leb(&mut expr_bytes, index.into(), false);

        let original_working_count = stack.working_count();

        if let Err(_) = ExpressionExecutor::instance().execute(&expr_bytes, stack, store) {
            None
        } else {
            if stack.working_count() == original_working_count + 1 {
                let result = stack.working_top(1)[0];
                stack.pop();
                Some(result)
            } else {
                None
            }
        }
    }

    fn do_local_set<Store: ExpressionStore>(
        stack: &mut Stack,
        store: &mut Store,
        index: u32,
        value: StackEntry,
    ) -> Option<()> {
        let mut expr_bytes = Vec::new();
        write_const_instruction(&mut expr_bytes, value);
        expr_bytes.push(Opcode::LocalSet.into());
        write_leb(&mut expr_bytes, index.into(), false);

        let original_working_count = stack.working_count();

        if let Err(_) = ExpressionExecutor::instance().execute(&expr_bytes, stack, store) {
            None
        } else {
            if stack.working_count() == original_working_count {
                Some(())
            } else {
                None
            }
        }
    }

    fn do_local_tee<Store: ExpressionStore>(
        stack: &mut Stack,
        store: &mut Store,
        index: u32,
        value: StackEntry,
    ) -> Option<StackEntry> {
        let mut expr_bytes = Vec::new();
        write_const_instruction(&mut expr_bytes, value);
        expr_bytes.push(Opcode::LocalTee.into());
        write_leb(&mut expr_bytes, index.into(), false);

        let original_working_count = stack.working_count();

        if let Err(_) = ExpressionExecutor::instance().execute(&expr_bytes, stack, store) {
            None
        } else {
            if stack.working_count() == original_working_count + 1 {
                let result = stack.working_top(1)[0];
                stack.pop();
                Some(result)
            } else {
                None
            }
        }
    }

    #[test]
    fn test_locals() {
        let mut stack = Stack::new();
        let mut store = TestStore::new();

        // Create a frame with room for five locals
        // TODOTODOTODO - this is actually nonsense. Locals are typed by the function that
        // creates them, and so therefore the "unused" stack entry value is completely unnecessary.
        // For now, however, I will work with what I have because there is no need to fix the frame
        // typing until I implement function calls.
        stack.push_frame(0, 5);

        assert_eq!(
            do_local_get(&mut stack, &mut store, 0),
            Some(StackEntry::Unused)
        );
        assert_eq!(
            do_local_get(&mut stack, &mut store, 1),
            Some(StackEntry::Unused)
        );
        assert_eq!(
            do_local_get(&mut stack, &mut store, 2),
            Some(StackEntry::Unused)
        );
        assert_eq!(
            do_local_get(&mut stack, &mut store, 3),
            Some(StackEntry::Unused)
        );
        assert_eq!(
            do_local_get(&mut stack, &mut store, 4),
            Some(StackEntry::Unused)
        );
        assert_eq!(do_local_get(&mut stack, &mut store, 5), None);

        assert_eq!(
            do_local_set(&mut stack, &mut store, 0, 42i32.into()),
            Some(())
        );
        assert_eq!(
            do_local_set(&mut stack, &mut store, 5, 42.0f32.into()),
            None
        );

        assert_eq!(do_local_get(&mut stack, &mut store, 0), Some(42i32.into()));

        assert_eq!(
            do_local_tee(&mut stack, &mut store, 0, 42i32.into()),
            Some(42i32.into())
        );
        assert_eq!(
            do_local_tee(&mut stack, &mut store, 5, 42.0f32.into()),
            None
        );

        assert_eq!(do_local_get(&mut stack, &mut store, 0), Some(42i32.into()));

        // Check that locals still work as expected when there is a working value on the stack
        stack.push(42.0f32.into());
        println!("{:?}", stack);
        assert_eq!(do_local_get(&mut stack, &mut store, 0), Some(42i32.into()));
    }
}
