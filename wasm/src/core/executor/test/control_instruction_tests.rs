use crate::core::{
    executor::execute_expression, stack_entry::StackEntry, BlockType, FuncType, Locals, Stack,
    ValueType,
};
use crate::parser::Opcode;

use super::instruction_generator::*;
use super::instruction_test_helpers::*;
use super::test_store::*;

#[test]
fn test_if_block() {
    let mut expr = make_expression_writer();
    expr.write_const_instruction(1_u32);
    let mut if_block_expr = expr.write_block_instruction(Opcode::If, BlockType::I32);
    if_block_expr.write_const_instruction(1_u32);
    let mut else_block_expr = if_block_expr.do_else();
    else_block_expr.write_const_instruction(2_u32);
    let expr = else_block_expr.do_end();

    test_single_return_expression!(expr, 1_u32);
}

#[test]
fn test_else_block() {
    let mut expr = make_expression_writer();
    expr.write_const_instruction(0_u32);
    let mut if_block_expr = expr.write_block_instruction(Opcode::If, BlockType::I32);
    if_block_expr.write_const_instruction(1_u32);
    let mut else_block_expr = if_block_expr.do_else();
    else_block_expr.write_const_instruction(2_u32);
    let expr = else_block_expr.do_end();

    test_single_return_expression!(expr, 2_u32);
}

#[test]
fn test_block_block_no_branches() {
    let expr = make_expression_writer();
    let mut block_expr = expr.write_block_instruction(Opcode::Block, BlockType::I32);
    block_expr.write_const_instruction(1_u32);
    let expr = block_expr.do_end();

    test_single_return_expression!(expr, 1_u32);
}

#[test]
fn test_loop_block_no_branches() {
    let expr = make_expression_writer();
    let mut block_expr = expr.write_block_instruction(Opcode::Loop, BlockType::I32); // The block type should be ignored for loops
    block_expr.write_const_instruction(1_u32);
    let expr = block_expr.do_end();

    test_no_return_expression!(expr);
}

fn write_local_value(
    expr: &mut ExpressionWriter,
    local_index: u64,
    local_value: impl Into<StackEntry>,
) {
    expr.write_const_instruction(local_value);
    expr.write_single_leb_instruction(Opcode::LocalSet, local_index);
}

fn modify_local_value(
    expr: &mut ExpressionWriter,
    local_index: u64,
    value: impl Into<StackEntry>,
    op: Opcode,
) {
    expr.write_single_leb_instruction(Opcode::LocalGet, local_index);

    expr.write_const_instruction(value);
    expr.write_single_byte_instruction(op);

    expr.write_single_leb_instruction(Opcode::LocalSet, local_index);
}

fn compare_local_value(
    expr: &mut ExpressionWriter,
    local_index: u64,
    comparand: impl Into<StackEntry>,
    op: Opcode,
) {
    expr.write_single_leb_instruction(Opcode::LocalGet, local_index);

    expr.write_const_instruction(comparand);
    expr.write_single_byte_instruction(op);
}

#[test]
fn test_loop_block() {
    let mut expr = make_expression_writer();

    // Write the number 10 into local 0
    write_local_value(&mut expr, 0, 10u64);

    // Enter a loop
    let mut loop_expr = expr.write_block_instruction(Opcode::Loop, BlockType::None);

    modify_local_value(&mut loop_expr, 0, 1_u64, Opcode::I64Sub);
    compare_local_value(&mut loop_expr, 0, 0_i64, Opcode::I64Ne);

    // If the value isn't zero, then branch back to the loop
    loop_expr.write_single_leb_instruction(Opcode::BrIf, 0);

    // Otherwise, we fall out of the end of the loop
    let mut expr = loop_expr.do_end();
    // Get the value from the local
    expr.write_single_leb_instruction(Opcode::LocalGet, 0);

    // Make a stack and a store
    let mut stack = Stack::new();
    let mut store = TestStore::new();

    // We push a frame onto the stack with the one local we use
    assert!(stack.push_test_frame(1).is_ok());

    assert!(execute_expression(&expr, &mut stack, &mut store).is_ok());
    assert_eq!(stack.working_count(), 1);
    assert_eq!(stack.working_top(1)[0], 0u64.into());
}

fn write_branch_tier(writer: ExpressionWriter, depth: u64, max_depth: u64) -> ExpressionWriter {
    // Make a nested block
    let mut nested_writer = writer.write_block_instruction(Opcode::Block, BlockType::None);

    if depth > 0 {
        // Inside each nested block, have another nested block as appropriate
        let mut nested_writer = write_branch_tier(nested_writer, depth - 1, max_depth);

        // Then increment the local
        modify_local_value(&mut nested_writer, 0, 1_u32, Opcode::I32Add);

        nested_writer.do_end()
    } else {
        // In the leaf, load local value 1 and use it as a branch target
        nested_writer.write_single_leb_instruction(Opcode::LocalGet, 1);

        // And generate a branch if using all of the integers
        let values: Vec<u64> = (0..max_depth).collect();
        nested_writer.write_branch_table(Opcode::BrTable, &values);

        nested_writer.do_end()
    }
}

#[test]
fn test_branch_table() {
    const MAX_DEPTH: u32 = 10;

    for jump_target in 0..15_u32 {
        let mut expr = make_expression_writer();

        write_local_value(&mut expr, 0, 0_u32);
        write_local_value(&mut expr, 1, jump_target);

        let mut expr = write_branch_tier(expr, MAX_DEPTH as u64, MAX_DEPTH as u64);

        // After the branch, load the value of local 0
        expr.write_single_leb_instruction(Opcode::LocalGet, 0);

        // Make a stack and a store
        let mut stack = Stack::new();
        let mut store = TestStore::new();

        // We push a frame onto the stack with the one local we use
        assert!(stack.push_test_frame(2).is_ok());

        assert!(execute_expression(&expr, &mut stack, &mut store).is_ok());
        assert_eq!(stack.working_count(), 1);
        assert_eq!(
            stack.working_top(1)[0],
            (MAX_DEPTH - std::cmp::min(jump_target, MAX_DEPTH - 1)).into()
        );
    }
}

#[test]
fn test_call() {
    let mut stack = Stack::new();
    let mut store = TestStore::new();

    let mut func_writer = make_expression_writer();
    func_writer.write_single_leb_instruction(Opcode::LocalGet, 0);
    func_writer.write_single_leb_instruction(Opcode::LocalGet, 1);
    func_writer.write_single_byte_instruction(Opcode::I32Add);
    func_writer.write_single_leb_instruction(Opcode::LocalTee, 2);

    assert_eq!(
        store.add_function(
            func_writer,
            FuncType::new(vec![ValueType::I32, ValueType::I32], vec![ValueType::I32]),
            vec![Locals::new(1, ValueType::I32)]
        ),
        0
    );

    let mut test_writer = make_expression_writer();
    test_writer.write_const_instruction(26_i32);
    test_writer.write_const_instruction(17_i32);
    test_writer.write_single_leb_instruction(Opcode::Call, 0);

    assert!(execute_expression(&test_writer, &mut stack, &mut store).is_ok());
    assert_eq!(stack.working_count(), 1);
    assert_eq!(stack.working_top(1)[0], 43_i32.into());
}
