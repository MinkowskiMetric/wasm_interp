use crate::core::{stack_entry::StackEntry, Stack};
use crate::parser::Opcode;

use super::instruction_generator::*;
use super::test_store::*;

use super::super::execute_core::execute_expression;

pub fn test_constant_opcode_impl(p1: StackEntry) -> Option<StackEntry> {
    // Allocate a byte vector and generate an instruction stream that will execute the op
    let mut expr_bytes: Vec<u8> = Vec::new();
    write_const_instruction(&mut expr_bytes, p1);

    // Now we need a stack and a store to run the op against
    let mut stack = Stack::new();
    let mut test_store = TestStore::new();

    if let Err(_) = execute_expression(&expr_bytes, &mut stack, &mut test_store) {
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
        assert_eq!(test_constant_opcode_impl($p1.into()), Some($p1.into()));
    };
}

pub fn test_unary_opcode_impl(p1: StackEntry, opcode: Opcode) -> Option<StackEntry> {
    // Allocate a byte vector and generate an instruction stream that will execute the op
    let mut expr_bytes: Vec<u8> = Vec::new();
    write_const_instruction(&mut expr_bytes, p1);
    expr_bytes.push(opcode.into());

    // Now we need a stack and a store to run the op against
    let mut stack = Stack::new();
    let mut test_store = TestStore::new();

    if let Err(_) = execute_expression(&expr_bytes, &mut stack, &mut test_store) {
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
macro_rules! test_unary_opcode {
    ($p1:expr, $opcode:expr, $r:expr) => {
        assert_eq!(
            test_unary_opcode_impl($p1.into(), $opcode.into()),
            Some($r.into())
        );
    };
}

pub fn test_binary_opcode_impl(
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

    if let Err(_) = execute_expression(&expr_bytes, &mut stack, &mut test_store) {
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
macro_rules! test_binary_opcode {
    ($p1:expr, $p2:expr, $opcode:expr, $r:expr) => {
        assert_eq!(
            test_binary_opcode_impl($p1.into(), $p2.into(), $opcode),
            Some($r.into())
        );
    };
}
