use std::convert::{TryFrom, TryInto};

use crate::core::{stack_entry::StackEntry, Stack};
use anyhow::{anyhow, Result};

pub fn get_stack_top(stack: &mut Stack, n: usize) -> Result<&[StackEntry]> {
    if stack.working_count() < n {
        Err(anyhow!("Not enough values on stack"))
    } else {
        Ok(stack.working_top(n))
    }
}

pub fn unary_op<
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

pub fn unary_boolean_op<
    ParamType: Sized + TryFrom<StackEntry, Error = anyhow::Error>,
    Func: Fn(ParamType) -> bool,
>(
    stack: &mut Stack,
    func: Func,
) -> Result<()> {
    unary_op(stack, |p: ParamType| if func(p) { 1u32 } else { 0u32 })
}

pub fn binary_op<
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

pub fn binary_boolean_op<
    ParamType: Sized + TryFrom<StackEntry, Error = anyhow::Error>,
    Func: Fn(ParamType, ParamType) -> bool,
>(
    stack: &mut Stack,
    func: Func,
) -> Result<()> {
    binary_op(
        stack,
        |p1: ParamType, p2: ParamType| if func(p1, p2) { 1u32 } else { 0u32 },
    )
}
