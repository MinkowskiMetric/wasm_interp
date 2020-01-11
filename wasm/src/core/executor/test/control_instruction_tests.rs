/*use crate::core::{
    executor::{execute_expression, ExpressionStore},
    stack_entry::StackEntry,
    Stack,
};
use crate::parser::Opcode;

use super::instruction_generator::*;
use super::instruction_test_helpers::*;
use super::test_store::*;

*/
use crate::core::BlockType;
use crate::parser::Opcode;

use super::instruction_generator::*;
use super::instruction_test_helpers::*;

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
    let mut block_expr = expr.write_block_instruction(Opcode::Loop, BlockType::I32);
    block_expr.write_const_instruction(1_u32);
    let expr = block_expr.do_end();

    test_single_return_expression!(expr, 1_u32);
}
