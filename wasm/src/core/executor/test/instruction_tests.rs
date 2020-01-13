use crate::core::{
    executor::{execute_expression, ExpressionStore},
    stack_entry::StackEntry,
    Stack,
};
use crate::parser::Opcode;

use super::instruction_generator::make_expression_writer;
use super::instruction_test_helpers::*;
use super::test_store::*;

#[test]
fn test_drop_op() {
    // Allocate a byte vector and generate an instruction stream that will execute the op
    let mut expr = make_expression_writer();
    expr.write_const_instruction(42i32);
    expr.write_const_instruction(2.0f64);
    expr.write_single_byte_instruction(Opcode::Drop);

    // Now we need a stack and a store to run the op against
    let mut stack = Stack::new();
    let mut test_store = TestStore::new();

    assert!(execute_expression(&expr, &mut stack, &mut test_store).is_ok());

    assert_eq!(stack.working_count(), 1);
    assert_eq!(stack.working_top(1)[0], 42i32.into());
}

#[test]
fn test_select_op() {
    // Allocate a byte vector and generate an instruction stream that will execute the op
    let mut expr = make_expression_writer();
    expr.write_const_instruction(42i32);
    expr.write_const_instruction(42.0f64);
    expr.write_const_instruction(1i32);
    expr.write_single_byte_instruction(Opcode::Select);

    // Now we need a stack and a store to run the op against
    let mut stack = Stack::new();
    let mut test_store = TestStore::new();

    assert!(execute_expression(&expr, &mut stack, &mut test_store).is_err());

    // It should have failed part way through the instruction, leaving two operands on the stack
    assert_eq!(stack.working_count(), 2);
    assert_eq!(stack.working_top(2)[0], 42i32.into());
    assert_eq!(stack.working_top(2)[1], 42.0f64.into());

    stack.pop_n(2);

    let mut expr = make_expression_writer();
    expr.write_const_instruction(42i32);
    expr.write_const_instruction(69i32);
    expr.write_const_instruction(1i32);
    expr.write_single_byte_instruction(Opcode::Select);

    assert!(execute_expression(&expr, &mut stack, &mut test_store).is_ok());

    assert_eq!(stack.working_count(), 1);
    assert_eq!(stack.working_top(1)[0], 42i32.into());

    stack.pop();

    let mut expr = make_expression_writer();
    expr.write_const_instruction(42i32);
    expr.write_const_instruction(69i32);
    expr.write_const_instruction(0i32);
    expr.write_single_byte_instruction(Opcode::Select);

    assert!(execute_expression(&expr, &mut stack, &mut test_store).is_ok());

    assert_eq!(stack.working_count(), 1);
    assert_eq!(stack.working_top(1)[0], 69i32.into());
}

#[test]
fn test_basic_ops() {
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

    test_unary_opcode!(7i32, Opcode::I32Eqz, 0u32);
    test_unary_opcode!(0i32, Opcode::I32Eqz, 1u32);
    test_binary_opcode!(7i32, 0i32, Opcode::I32Eq, 0u32);
    test_binary_opcode!(7i32, 7i32, Opcode::I32Eq, 1u32);
    test_binary_opcode!(7i32, 0i32, Opcode::I32Ne, 1u32);
    test_binary_opcode!(7i32, 7i32, Opcode::I32Ne, 0u32);
    test_binary_opcode!(-1i32, 0i32, Opcode::I32LtS, 1u32);
    test_binary_opcode!(0i32, -1i32, Opcode::I32LtS, 0u32);
    test_binary_opcode!(-1i32, 0i32, Opcode::I32LtU, 0u32);
    test_binary_opcode!(0i32, -1i32, Opcode::I32LtU, 1u32);
    test_binary_opcode!(-1i32, 0i32, Opcode::I32GtS, 0u32);
    test_binary_opcode!(0i32, -1i32, Opcode::I32GtS, 1u32);
    test_binary_opcode!(-1i32, 0i32, Opcode::I32GtU, 1u32);
    test_binary_opcode!(0i32, -1i32, Opcode::I32GtU, 0u32);
    test_binary_opcode!(-1i32, 0i32, Opcode::I32LeS, 1u32);
    test_binary_opcode!(0i32, -1i32, Opcode::I32LeS, 0u32);
    test_binary_opcode!(-1i32, 0i32, Opcode::I32LeU, 0u32);
    test_binary_opcode!(0i32, -1i32, Opcode::I32LeU, 1u32);
    test_binary_opcode!(-1i32, 0i32, Opcode::I32GeS, 0u32);
    test_binary_opcode!(0i32, -1i32, Opcode::I32GeS, 1u32);
    test_binary_opcode!(-1i32, 0i32, Opcode::I32GeU, 1u32);
    test_binary_opcode!(0i32, -1i32, Opcode::I32GeU, 0u32);

    test_unary_opcode!(7i64, Opcode::I64Eqz, 0u32);
    test_unary_opcode!(0i64, Opcode::I64Eqz, 1u32);
    test_binary_opcode!(7i64, 0i64, Opcode::I64Eq, 0u32);
    test_binary_opcode!(7i64, 7i64, Opcode::I64Eq, 1u32);
    test_binary_opcode!(7i64, 0i64, Opcode::I64Ne, 1u32);
    test_binary_opcode!(7i64, 7i64, Opcode::I64Ne, 0u32);
    test_binary_opcode!(-1i64, 0i64, Opcode::I64LtS, 1u32);
    test_binary_opcode!(0i64, -1i64, Opcode::I64LtS, 0u32);
    test_binary_opcode!(-1i64, 0i64, Opcode::I64LtU, 0u32);
    test_binary_opcode!(0i64, -1i64, Opcode::I64LtU, 1u32);
    test_binary_opcode!(-1i64, 0i64, Opcode::I64GtS, 0u32);
    test_binary_opcode!(0i64, -1i64, Opcode::I64GtS, 1u32);
    test_binary_opcode!(-1i64, 0i64, Opcode::I64GtU, 1u32);
    test_binary_opcode!(0i64, -1i64, Opcode::I64GtU, 0u32);
    test_binary_opcode!(-1i64, 0i64, Opcode::I64LeS, 1u32);
    test_binary_opcode!(0i64, -1i64, Opcode::I64LeS, 0u32);
    test_binary_opcode!(-1i64, 0i64, Opcode::I64LeU, 0u32);
    test_binary_opcode!(0i64, -1i64, Opcode::I64LeU, 1u32);
    test_binary_opcode!(-1i64, 0i64, Opcode::I64GeS, 0u32);
    test_binary_opcode!(0i64, -1i64, Opcode::I64GeS, 1u32);
    test_binary_opcode!(-1i64, 0i64, Opcode::I64GeU, 1u32);
    test_binary_opcode!(0i64, -1i64, Opcode::I64GeU, 0u32);

    test_binary_opcode!(7.0f32, 0.0f32, Opcode::F32Eq, 0u32);
    test_binary_opcode!(7.0f32, 7.0f32, Opcode::F32Eq, 1u32);
    test_binary_opcode!(7.0f32, 0.0f32, Opcode::F32Ne, 1u32);
    test_binary_opcode!(7.0f32, 7.0f32, Opcode::F32Ne, 0u32);
    test_binary_opcode!(-1.0f32, 0.0f32, Opcode::F32Lt, 1u32);
    test_binary_opcode!(0.0f32, -1.0f32, Opcode::F32Lt, 0u32);
    test_binary_opcode!(-1.0f32, 0.0f32, Opcode::F32Gt, 0u32);
    test_binary_opcode!(0.0f32, -1.0f32, Opcode::F32Gt, 1u32);
    test_binary_opcode!(-1.0f32, 0.0f32, Opcode::F32Le, 1u32);
    test_binary_opcode!(0.0f32, -1.0f32, Opcode::F32Le, 0u32);
    test_binary_opcode!(-1.0f32, 0.0f32, Opcode::F32Ge, 0u32);
    test_binary_opcode!(0.0f32, -1.0f32, Opcode::F32Ge, 1u32);

    test_binary_opcode!(7.0f64, 0.0f64, Opcode::F64Eq, 0u32);
    test_binary_opcode!(7.0f64, 7.0f64, Opcode::F64Eq, 1u32);
    test_binary_opcode!(7.0f64, 0.0f64, Opcode::F64Ne, 1u32);
    test_binary_opcode!(7.0f64, 7.0f64, Opcode::F64Ne, 0u32);
    test_binary_opcode!(-1.0f64, 0.0f64, Opcode::F64Lt, 1u32);
    test_binary_opcode!(0.0f64, -1.0f64, Opcode::F64Lt, 0u32);
    test_binary_opcode!(-1.0f64, 0.0f64, Opcode::F64Gt, 0u32);
    test_binary_opcode!(0.0f64, -1.0f64, Opcode::F64Gt, 1u32);
    test_binary_opcode!(-1.0f64, 0.0f64, Opcode::F64Le, 1u32);
    test_binary_opcode!(0.0f64, -1.0f64, Opcode::F64Le, 0u32);
    test_binary_opcode!(-1.0f64, 0.0f64, Opcode::F64Ge, 0u32);
    test_binary_opcode!(0.0f64, -1.0f64, Opcode::F64Ge, 1u32);

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

    test_unary_opcode!(7.0f32, Opcode::F32Abs, 7.0f32);
    test_unary_opcode!(-7.0f32, Opcode::F32Abs, 7.0f32);
    test_unary_opcode!(7.0f32, Opcode::F32Neg, -7.0f32);
    test_unary_opcode!(-7.0f32, Opcode::F32Neg, 7.0f32);
    test_unary_opcode!(7.1f32, Opcode::F32Ceil, 8.0f32);
    test_unary_opcode!(-7.1f32, Opcode::F32Ceil, -7.0f32);
    test_unary_opcode!(7.1f32, Opcode::F32Floor, 7.0f32);
    test_unary_opcode!(-7.1f32, Opcode::F32Floor, -8.0f32);
    test_unary_opcode!(7.1f32, Opcode::F32Nearest, 7.0f32);
    test_unary_opcode!(-7.1f32, Opcode::F32Nearest, -7.0f32);
    test_unary_opcode!(64.0f32, Opcode::F32Sqrt, 8.0f32);
    test_binary_opcode!(7.0f32, 8.0f32, Opcode::F32Add, 15.0f32);
    test_binary_opcode!(7.0f32, -1.0f32, Opcode::F32Add, 6.0f32);
    test_binary_opcode!(7.0f32, 8.0f32, Opcode::F32Sub, -1.0f32);
    test_binary_opcode!(7.0f32, -1.0f32, Opcode::F32Sub, 8.0f32);
    test_binary_opcode!(7.0f32, 8.0f32, Opcode::F32Mul, 56.0f32);
    test_binary_opcode!(7.0f32, -1.0f32, Opcode::F32Mul, -7.0f32);
    test_binary_opcode!(16.0f32, 8.0f32, Opcode::F32Div, 2.0f32);
    test_binary_opcode!(7.0f32, -1.0f32, Opcode::F32Div, -7.0f32);
    test_binary_opcode!(16.0f32, 8.0f32, Opcode::F32Min, 8.0f32);
    test_binary_opcode!(16.0f32, 8.0f32, Opcode::F32Max, 16.0f32);
    test_binary_opcode!(7.0f32, -1.0f32, Opcode::F32CopySign, -7.0f32);
    test_binary_opcode!(7.0f32, 1.0f32, Opcode::F32CopySign, 7.0f32);
    test_binary_opcode!(-7.0f32, 1.0f32, Opcode::F32CopySign, 7.0f32);

    test_unary_opcode!(7.0f64, Opcode::F64Abs, 7.0f64);
    test_unary_opcode!(-7.0f64, Opcode::F64Abs, 7.0f64);
    test_unary_opcode!(7.0f64, Opcode::F64Neg, -7.0f64);
    test_unary_opcode!(-7.0f64, Opcode::F64Neg, 7.0f64);
    test_unary_opcode!(7.1f64, Opcode::F64Ceil, 8.0f64);
    test_unary_opcode!(-7.1f64, Opcode::F64Ceil, -7.0f64);
    test_unary_opcode!(7.1f64, Opcode::F64Floor, 7.0f64);
    test_unary_opcode!(-7.1f64, Opcode::F64Floor, -8.0f64);
    test_unary_opcode!(7.1f64, Opcode::F64Nearest, 7.0f64);
    test_unary_opcode!(-7.1f64, Opcode::F64Nearest, -7.0f64);
    test_unary_opcode!(64.0f64, Opcode::F64Sqrt, 8.0f64);
    test_binary_opcode!(7.0f64, 8.0f64, Opcode::F64Add, 15.0f64);
    test_binary_opcode!(7.0f64, -1.0f64, Opcode::F64Add, 6.0f64);
    test_binary_opcode!(7.0f64, 8.0f64, Opcode::F64Sub, -1.0f64);
    test_binary_opcode!(7.0f64, -1.0f64, Opcode::F64Sub, 8.0f64);
    test_binary_opcode!(7.0f64, 8.0f64, Opcode::F64Mul, 56.0f64);
    test_binary_opcode!(7.0f64, -1.0f64, Opcode::F64Mul, -7.0f64);
    test_binary_opcode!(16.0f64, 8.0f64, Opcode::F64Div, 2.0f64);
    test_binary_opcode!(7.0f64, -1.0f64, Opcode::F64Div, -7.0f64);
    test_binary_opcode!(16.0f64, 8.0f64, Opcode::F64Min, 8.0f64);
    test_binary_opcode!(16.0f64, 8.0f64, Opcode::F64Max, 16.0f64);
    test_binary_opcode!(7.0f64, -1.0f64, Opcode::F64CopySign, -7.0f64);
    test_binary_opcode!(7.0f64, 1.0f64, Opcode::F64CopySign, 7.0f64);
    test_binary_opcode!(-7.0f64, 1.0f64, Opcode::F64CopySign, 7.0f64);

    test_unary_opcode!(0xFFFFFFFF00110011u64, Opcode::I32WrapI64, 0x00110011u32);
    test_unary_opcode!(-7.5f32, Opcode::I32TruncF32S, -7i32);
    test_unary_opcode!(3000000000.0f32, Opcode::I32TruncF32U, 3000000000u32);
    test_unary_opcode!(-7.5f64, Opcode::I32TruncF64S, -7i32);
    test_unary_opcode!(3000000000.0f64, Opcode::I32TruncF64U, 3000000000u32);
    test_unary_opcode!(-1i32, Opcode::I64ExtendI32S, -1i64);
    test_unary_opcode!(-1i32, Opcode::I64ExtendI32U, 0xFFFFFFFFi64);
    test_unary_opcode!(-7.5f32, Opcode::I64TruncF32S, -7i64);
    test_unary_opcode!(3000000000.0f32, Opcode::I64TruncF32U, 3000000000u64);
    test_unary_opcode!(-7.5f64, Opcode::I64TruncF64S, -7i64);
    test_unary_opcode!(3000000000.0f64, Opcode::I64TruncF64U, 3000000000u64);
    test_unary_opcode!(-1i32, Opcode::F32ConvertI32S, -1.0f32);
    test_unary_opcode!(-1i32, Opcode::F32ConvertI32U, 4294967295.0f32);
    test_unary_opcode!(-1i64, Opcode::F32ConvertI64S, -1.0f32);
    test_unary_opcode!(-1i64, Opcode::F32ConvertI64U, 18446744073709551615.0f32);
    test_unary_opcode!(-1f64, Opcode::F32DemoteF64, -1f32);
    test_unary_opcode!(-1i32, Opcode::F64ConvertI32S, -1.0f64);
    test_unary_opcode!(-1i32, Opcode::F64ConvertI32U, 4294967295.0f64);
    test_unary_opcode!(-1i64, Opcode::F64ConvertI64S, -1.0f64);
    test_unary_opcode!(-1i64, Opcode::F64ConvertI64U, 18446744073709551615.0f64);
    test_unary_opcode!(-1f32, Opcode::F64PromoteF32, -1f64);
    test_unary_opcode!(-1.0f32, Opcode::I32ReinterpretF32, 0xbf800000u32);
    test_unary_opcode!(-1.0f64, Opcode::I64ReinterpretF64, 0xbff0000000000000u64);
    test_unary_opcode!(0xbf800000u32, Opcode::F32ReinterpretI32, -1.0f32);
    test_unary_opcode!(0xbff0000000000000u64, Opcode::F64ReinterpretI64, -1.0f64);
}

fn do_local_get(
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
    index: u32,
) -> Option<StackEntry> {
    let mut expr = make_expression_writer();
    expr.write_single_leb_instruction(Opcode::LocalGet, index.into());

    let original_working_count = stack.working_count();

    if let Err(_) = execute_expression(&expr, stack, store) {
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

fn do_local_set(
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
    index: u32,
    value: StackEntry,
) -> Option<()> {
    let mut expr = make_expression_writer();
    expr.write_const_instruction(value);
    expr.write_single_leb_instruction(Opcode::LocalSet, index.into());

    let original_working_count = stack.working_count();

    if let Err(_) = execute_expression(&expr, stack, store) {
        None
    } else {
        if stack.working_count() == original_working_count {
            Some(())
        } else {
            None
        }
    }
}

fn do_local_tee(
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
    index: u32,
    value: StackEntry,
) -> Option<StackEntry> {
    let mut expr = make_expression_writer();
    expr.write_const_instruction(value);
    expr.write_single_leb_instruction(Opcode::LocalTee, index.into());

    let original_working_count = stack.working_count();

    if let Err(_) = execute_expression(&expr, stack, store) {
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
fn test_locals_ops() {
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
    assert_eq!(do_local_get(&mut stack, &mut store, 0), Some(42i32.into()));
}

#[test]
fn test_memory_ops() {
    let mut stack = Stack::new();
    let mut store = TestStore::new();

    store.enable_memory();

    static FIXED_DATA: [u8; 8] = [0x0d, 0xf0, 0xad, 0xba, 0x0d, 0xf0, 0xad, 0xba];
    store.write_data(0, 0, &FIXED_DATA).unwrap();

    test_memory_load!(
        Opcode::I32Load,
        0,
        0,
        0,
        &mut stack,
        &mut store,
        0xbaadf00d_u32
    );

    test_memory_store!(Opcode::F32Store, 0, 0, 0, 42.0_f32, &mut stack, &mut store);
    test_memory_load!(Opcode::F32Load, 0, 0, 0, &mut stack, &mut store, 42.0_f32);

    for (unsigned_opcode, signed_opcode, byte_count) in &[
        (Opcode::I32Load8U, Opcode::I32Load8S, 1),
        (Opcode::I32Load16U, Opcode::I32Load16S, 2),
        (Opcode::I32Load, Opcode::I32Load, 4),
    ] {
        let mut unsigned_bytes: [u8; 8] = [0; 8];
        let mut signed_bytes: [u8; 8] = [0; 8];

        for i in 0..8 {
            if i < *byte_count {
                unsigned_bytes[i] = 0;
                signed_bytes[i] = 0xff;
            } else {
                unsigned_bytes[i] = 0xff;
                signed_bytes[i] = 0;
            }
        }

        store.write_data(0, 128, &unsigned_bytes).unwrap();
        store.write_data(0, 256, &signed_bytes).unwrap();

        test_memory_load!(*unsigned_opcode, 128, 0, 0, &mut stack, &mut store, 0_u32);
        test_memory_load!(*signed_opcode, 256, 0, 0, &mut stack, &mut store, -1_i32);
    }

    for (unsigned_opcode, signed_opcode, byte_count) in &[
        (Opcode::I64Load8U, Opcode::I64Load8S, 1),
        (Opcode::I64Load16U, Opcode::I64Load16S, 2),
        (Opcode::I64Load32U, Opcode::I64Load32S, 4),
        (Opcode::I64Load, Opcode::I64Load, 8),
    ] {
        let mut unsigned_bytes: [u8; 8] = [0; 8];
        let mut signed_bytes: [u8; 8] = [0; 8];

        for i in 0..8 {
            if i < *byte_count {
                unsigned_bytes[i] = 0;
                signed_bytes[i] = 0xff;
            } else {
                unsigned_bytes[i] = 0xff;
                signed_bytes[i] = 0;
            }
        }

        store.write_data(0, 128, &unsigned_bytes).unwrap();
        store.write_data(0, 256, &signed_bytes).unwrap();

        test_memory_load!(*unsigned_opcode, 128, 0, 0, &mut stack, &mut store, 0_u64);
        test_memory_load!(*signed_opcode, 256, 0, 0, &mut stack, &mut store, -1_i64);
    }

    for (opcode, byte_count) in &[
        (Opcode::I32Store8, 1),
        (Opcode::I32Store16, 2),
        (Opcode::I32Store, 4),
    ] {
        let set_bytes: [u8; 8] = [0xff; 8];

        store.write_data(0, 128, &set_bytes).unwrap();

        test_memory_store!(*opcode, 128, 0, 0, 0_u32, &mut stack, &mut store);

        let mut check_bytes: [u8; 8] = [0xff; 8];
        store.read_data(0, 128, &mut check_bytes).unwrap();

        for i in 0..8 {
            if i < *byte_count {
                assert_eq!(check_bytes[i], 0x00);
            } else {
                assert_eq!(check_bytes[i], 0xff);
            }
        }
    }

    for (opcode, byte_count) in &[
        (Opcode::I64Store8, 1),
        (Opcode::I64Store16, 2),
        (Opcode::I64Store32, 4),
        (Opcode::I64Store, 8),
    ] {
        let set_bytes: [u8; 8] = [0xff; 8];

        store.write_data(0, 128, &set_bytes).unwrap();

        test_memory_store!(*opcode, 128, 0, 0, 0_u64, &mut stack, &mut store);

        let mut check_bytes: [u8; 8] = [0xff; 8];
        store.read_data(0, 128, &mut check_bytes).unwrap();

        for i in 0..8 {
            if i < *byte_count {
                assert_eq!(check_bytes[i], 0x00);
            } else {
                assert_eq!(check_bytes[i], 0xff);
            }
        }
    }

    let mut expr = make_expression_writer();
    expr.write_single_leb_instruction(Opcode::MemorySize, 0);

    assert!(execute_expression(&expr, &mut stack, &mut store).is_ok());
    assert_eq!(stack.working_count(), 1);
    assert_eq!(stack.working_top(1)[0], 1u32.into());
    stack.pop();

    let mut expr = make_expression_writer();
    expr.write_const_instruction(1_i32);
    expr.write_single_leb_instruction(Opcode::MemoryGrow, 0);

    assert!(execute_expression(&expr, &mut stack, &mut store).is_ok());
    assert_eq!(stack.working_count(), 1);
    assert_eq!(stack.working_top(1)[0], 1u32.into());
    stack.pop();

    assert_eq!(store.get_memory_size(0).ok(), Some(2));

    let mut expr = make_expression_writer();
    expr.write_const_instruction(10_i32);
    expr.write_single_leb_instruction(Opcode::MemoryGrow, 0);

    assert!(execute_expression(&expr, &mut stack, &mut store).is_ok());
    assert_eq!(stack.working_count(), 1);
    assert_eq!(stack.working_top(1)[0], StackEntry::from(-1i32));
    stack.pop();

    assert_eq!(store.get_memory_size(0).ok(), Some(2));
}
