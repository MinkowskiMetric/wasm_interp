use std::convert::TryInto;

use crate::core::{stack_entry::StackEntry, ExpressionStore, Stack};
use crate::parser::{InstructionSource, Opcode};

use super::instruction_generator::make_expression_writer;
use super::test_store::*;

use super::super::execute_core::execute_expression;

pub fn test_constant_opcode_impl(p1: impl Into<StackEntry>) -> Option<StackEntry> {
    // Allocate a byte vector and generate an instruction stream that will execute the op
    let mut expr = make_expression_writer();
    expr.write_const_instruction(p1);

    // Now we need a stack and a store to run the op against
    let mut stack = Stack::new();
    let mut test_store = TestStore::new();

    if let Err(_) = execute_expression(&expr, &mut stack, &mut test_store) {
        None
    } else {
        if stack.working_count() == 1 {
            Some(stack.working_top(1)[0])
        } else {
            None
        }
    }
}

#[macro_export]
macro_rules! test_constant_opcode {
    ($p1:expr) => {
        assert_eq!(test_constant_opcode_impl($p1), Some($p1.into()));
    };
}

pub fn test_no_return_expression_impl(expr: impl InstructionSource) -> Option<()> {
    // Now we need a stack and a store to run the op against
    let mut stack = Stack::new();
    let mut test_store = TestStore::new();

    // We push a frame onto the stack. This helps the expressions in the case they might need
    // to use a block
    stack.push_frame(0, 0);

    if let Err(_) = execute_expression(&expr, &mut stack, &mut test_store) {
        None
    } else {
        if stack.working_count() == 0 {
            Some(())
        } else {
            None
        }
    }
}

#[macro_export]
macro_rules! test_no_return_expression {
    ($expr:expr) => {
        assert_eq!(test_no_return_expression_impl($expr), Some(()));
    };
}

pub fn test_single_return_expression_impl(expr: impl InstructionSource) -> Option<StackEntry> {
    // Now we need a stack and a store to run the op against
    let mut stack = Stack::new();
    let mut test_store = TestStore::new();

    // We push a frame onto the stack. This helps the expressions in the case they might need
    // to use a block
    stack.push_frame(0, 0);

    if let Err(_) = execute_expression(&expr, &mut stack, &mut test_store) {
        None
    } else {
        if stack.working_count() == 1 {
            Some(stack.working_top(1)[0])
        } else {
            None
        }
    }
}

#[macro_export]
macro_rules! test_single_return_expression {
    ($expr:expr, $r:expr) => {
        assert_eq!(test_single_return_expression_impl($expr), Some($r.into()));
    };
}

pub fn test_unary_opcode_impl(p1: impl Into<StackEntry>, opcode: Opcode) -> Option<StackEntry> {
    // Allocate a byte vector and generate an instruction stream that will execute the op
    let mut expr = make_expression_writer();
    expr.write_const_instruction(p1.into());
    expr.write_single_byte_instruction(opcode);

    test_single_return_expression_impl(expr)
}

#[macro_export]
macro_rules! test_unary_opcode {
    ($p1:expr, $opcode:expr, $r:expr) => {
        assert_eq!(test_unary_opcode_impl($p1, $opcode.into()), Some($r.into()));
    };
}

pub fn test_binary_opcode_impl(
    p1: impl Into<StackEntry>,
    p2: impl Into<StackEntry>,
    opcode: Opcode,
) -> Option<StackEntry> {
    // Allocate a byte vector and generate an instruction stream that will execute the op
    let mut expr = make_expression_writer();
    expr.write_const_instruction(p1);
    expr.write_const_instruction(p2);
    expr.write_single_byte_instruction(opcode);

    test_single_return_expression_impl(expr)
}

#[macro_export]
macro_rules! test_binary_opcode {
    ($p1:expr, $p2:expr, $opcode:expr, $r:expr) => {
        assert_eq!(test_binary_opcode_impl($p1, $p2, $opcode), Some($r.into()));
    };
}

fn memory_load_expression(
    opcode: Opcode,
    address: u32,
    mem_idx: usize,
    offset: u32,
) -> impl InstructionSource {
    let mut expr = make_expression_writer();
    expr.write_const_instruction(address);
    expr.write_two_leb_instruction(opcode, mem_idx.try_into().unwrap(), offset.into());
    expr
}

pub fn test_memory_load_impl(
    opcode: Opcode,
    address: u32,
    mem_idx: usize,
    offset: u32,
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
) -> Option<StackEntry> {
    let expr = memory_load_expression(opcode, address, mem_idx, offset);
    if let Err(_) = execute_expression(&expr, stack, store) {
        None
    } else {
        if stack.working_count() == 1 {
            let r = Some(stack.working_top(1)[0]);
            stack.pop();
            r
        } else {
            None
        }
    }
}

#[macro_export]
macro_rules! test_memory_load {
    ($opcode:expr, $address:expr, $mem_idx:expr, $offset:expr, $stack:expr, $store:expr, $r:expr) => {
        assert_eq!(
            test_memory_load_impl($opcode, $address, $mem_idx, $offset, $stack, $store),
            Some($r.into())
        );
    };
}

fn memory_store_expression(
    opcode: Opcode,
    address: u32,
    mem_idx: usize,
    offset: u32,
    value: impl Into<StackEntry>,
) -> impl InstructionSource {
    let mut expr = make_expression_writer();
    expr.write_const_instruction(address);
    expr.write_const_instruction(value);
    expr.write_two_leb_instruction(opcode, mem_idx.try_into().unwrap(), offset.into());
    expr
}

pub fn test_memory_store_impl(
    opcode: Opcode,
    address: u32,
    mem_idx: usize,
    offset: u32,
    value: impl Into<StackEntry>,
    stack: &mut Stack,
    store: &mut impl ExpressionStore,
) -> Option<()> {
    let expr = memory_store_expression(opcode, address, mem_idx, offset, value);
    if let Err(_) = execute_expression(&expr, stack, store) {
        None
    } else {
        if stack.working_count() == 0 {
            Some(())
        } else {
            None
        }
    }
}

#[macro_export]
macro_rules! test_memory_store {
    ($opcode:expr, $address:expr, $mem_idx:expr, $offset:expr, $value:expr, $stack:expr, $store:expr) => {
        assert_eq!(
            test_memory_store_impl($opcode, $address, $mem_idx, $offset, $value, $stack, $store),
            Some(())
        );
    };
}
