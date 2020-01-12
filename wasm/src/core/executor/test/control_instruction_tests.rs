use crate::core::{executor::execute_expression, BlockType, Stack};
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

#[test]
fn test_loop_block() {
    let mut expr = make_expression_writer();

    // Write the number 10 into local 0
    expr.write_const_instruction(10u64);
    expr.write_single_leb_instruction(Opcode::LocalSet, 0);

    // Enter a loop
    let mut loop_expr = expr.write_block_instruction(Opcode::Loop, BlockType::None);

    // Get the value from local 0
    loop_expr.write_single_leb_instruction(Opcode::LocalGet, 0);

    // subtract 1
    loop_expr.write_const_instruction(1u64);
    loop_expr.write_single_byte_instruction(Opcode::I64Sub);

    // Store the value but keep it locally
    loop_expr.write_single_leb_instruction(Opcode::LocalTee, 0);

    // Compare the value with zero
    loop_expr.write_const_instruction(0u64);
    loop_expr.write_single_byte_instruction(Opcode::I64Ne);

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
    stack.push_frame(0, 1);

    assert!(execute_expression(&expr, &mut stack, &mut store).is_ok());
    assert_eq!(stack.working_count(), 1);
    assert_eq!(stack.working_top(1)[0], 0u64.into());
}
