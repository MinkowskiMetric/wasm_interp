use std::{
    cell::RefCell,
    convert::{TryFrom, TryInto},
    rc::Rc,
};

use crate::core::{stack_entry::StackEntry, Memory, Stack};
use crate::parser::{self, Instruction, Opcode};
use anyhow::{anyhow, Result};
use generic_array::typenum::consts::{U1, U2, U4, U8};
use generic_array::{ArrayLength, GenericArray};

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

    fn get_memory(&self, idx: usize) -> Result<Rc<RefCell<Memory>>>;
}

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

fn unary_boolean_op<
    ParamType: Sized + TryFrom<StackEntry, Error = anyhow::Error>,
    Func: Fn(ParamType) -> bool,
>(
    stack: &mut Stack,
    func: Func,
) -> Result<()> {
    unary_op(stack, |p: ParamType| if func(p) { 1u32 } else { 0u32 })
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

fn binary_boolean_op<
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

trait LEByteConvert {
    type ArrayLength: ArrayLength<u8>;

    fn from_bytes(bytes: GenericArray<u8, Self::ArrayLength>) -> Self;
    fn to_bytes(&self) -> GenericArray<u8, Self::ArrayLength>;
}

impl LEByteConvert for i8 {
    type ArrayLength = U1;

    fn from_bytes(bytes: GenericArray<u8, Self::ArrayLength>) -> Self {
        Self::from_le_bytes(bytes.into())
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::ArrayLength> {
        self.to_le_bytes().into()
    }
}

impl LEByteConvert for u8 {
    type ArrayLength = U1;

    fn from_bytes(bytes: GenericArray<u8, Self::ArrayLength>) -> Self {
        Self::from_le_bytes(bytes.into())
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::ArrayLength> {
        self.to_le_bytes().into()
    }
}

impl LEByteConvert for i16 {
    type ArrayLength = U2;

    fn from_bytes(bytes: GenericArray<u8, Self::ArrayLength>) -> Self {
        Self::from_le_bytes(bytes.into())
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::ArrayLength> {
        self.to_le_bytes().into()
    }
}

impl LEByteConvert for u16 {
    type ArrayLength = U2;

    fn from_bytes(bytes: GenericArray<u8, Self::ArrayLength>) -> Self {
        Self::from_le_bytes(bytes.into())
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::ArrayLength> {
        self.to_le_bytes().into()
    }
}

impl LEByteConvert for i32 {
    type ArrayLength = U4;

    fn from_bytes(bytes: GenericArray<u8, Self::ArrayLength>) -> Self {
        Self::from_le_bytes(bytes.into())
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::ArrayLength> {
        self.to_le_bytes().into()
    }
}

impl LEByteConvert for u32 {
    type ArrayLength = U4;

    fn from_bytes(bytes: GenericArray<u8, Self::ArrayLength>) -> Self {
        Self::from_le_bytes(bytes.into())
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::ArrayLength> {
        self.to_le_bytes().into()
    }
}

impl LEByteConvert for u64 {
    type ArrayLength = U8;

    fn from_bytes(bytes: GenericArray<u8, Self::ArrayLength>) -> Self {
        Self::from_le_bytes(bytes.into())
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::ArrayLength> {
        self.to_le_bytes().into()
    }
}

impl LEByteConvert for f32 {
    type ArrayLength = U4;

    fn from_bytes(bytes: GenericArray<u8, Self::ArrayLength>) -> Self {
        Self::from_le_bytes(bytes.into())
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::ArrayLength> {
        self.to_le_bytes().into()
    }
}

impl LEByteConvert for f64 {
    type ArrayLength = U8;

    fn from_bytes(bytes: GenericArray<u8, Self::ArrayLength>) -> Self {
        Self::from_le_bytes(bytes.into())
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::ArrayLength> {
        self.to_le_bytes().into()
    }
}

fn mem_load<
    ValueType: Sized + Into<StackEntry>,
    IntType: Sized + LEByteConvert,
    FuncType: Fn(IntType) -> ValueType,
    Store: ExpressionStore,
>(
    instruction: Instruction,
    stack: &mut Stack,
    store: &mut Store,
    func: FuncType,
) -> Result<()> {
    let (mem_idx, offset) = instruction.get_pair_u32_as_usize_arg();
    let memory = store.get_memory(mem_idx)?;
    let memory = memory.borrow();

    let base_address = get_stack_top(stack, 1)?[0];
    let base_address = usize::try_from(u32::try_from(base_address)?).unwrap();
    stack.pop();

    let final_address = base_address + offset;

    // A limitaton of the rust syntax here means you can't make the array the correct
    // size. Which is a bit annoying, but not very.
    let mut bytes: GenericArray<u8, IntType::ArrayLength> =
        unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    memory.get_data(final_address, &mut bytes)?;

    let int_value = IntType::from_bytes(bytes);
    let ret_value = func(int_value);

    stack.push(ret_value.into());

    Ok(())
}

fn mem_store<
    ValueType: Sized + TryFrom<StackEntry, Error = anyhow::Error>,
    IntType: Sized + LEByteConvert,
    FuncType: Fn(ValueType) -> IntType,
    Store: ExpressionStore,
>(
    instruction: Instruction,
    stack: &mut Stack,
    store: &mut Store,
    func: FuncType,
) -> Result<()> {
    let (mem_idx, offset) = instruction.get_pair_u32_as_usize_arg();
    let memory = store.get_memory(mem_idx)?;
    let memory = &mut memory.borrow_mut();

    let value = get_stack_top(stack, 1)?[0];
    let value = ValueType::try_from(value)?;
    stack.pop();

    let base_address = get_stack_top(stack, 1)?[0];
    let base_address = usize::try_from(u32::try_from(base_address)?).unwrap();
    stack.pop();

    let final_address = base_address + offset;

    let bytes = func(value).to_bytes();
    memory.set_data(final_address, &bytes)?;

    Ok(())
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
                Opcode::Unreachable => return Err(anyhow!("Unreachable opcode")),
                Opcode::Nop => {}
                Opcode::Block => unimplemented!(),
                Opcode::Loop => unimplemented!(),
                Opcode::If => unimplemented!(),
                Opcode::Else => panic!("Else opcode should not pass through opcode iterator"),
                Opcode::End => panic!("End opcode should not pass through opcode iterator"),
                Opcode::Br => unimplemented!(),
                Opcode::BrIf => unimplemented!(),
                Opcode::BrTable => unimplemented!(),
                Opcode::Return => unimplemented!(),
                Opcode::Call => unimplemented!(),
                Opcode::CallIndirect => unimplemented!(),

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
                    let memory = store.get_memory(memory_idx)?;
                    let memory = memory.borrow();

                    let size = u32::try_from(memory.current_size()).unwrap();
                    stack.push(size.into());
                }
                Opcode::MemoryGrow => {
                    let memory_idx = instruction.get_single_u32_as_usize_arg();
                    let memory = store.get_memory(memory_idx)?;
                    let memory = &mut memory.borrow_mut();

                    let grow_by = get_stack_top(stack, 1)?[0];
                    let grow_by = u32::try_from(grow_by)?;
                    let grow_by = usize::try_from(grow_by).unwrap();
                    stack.pop();

                    let original_size = u32::try_from(memory.current_size()).unwrap();

                    if memory.grow_by(grow_by).is_ok() {
                        stack.push(original_size.into());
                    } else {
                        stack.push(StackEntry::from(-1i32));
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
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::convert::TryInto;

    struct TestStore {
        memory: Rc<RefCell<Memory>>,
        memory_enabled: bool,
    }

    impl TestStore {
        pub fn new() -> Self {
            Self {
                memory: Rc::new(RefCell::new(Memory::new_from_bounds(1, Some(3)))),
                memory_enabled: false,
            }
        }

        pub fn enable_memory(&mut self) {
            self.memory_enabled = true;
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

        fn get_memory(&self, idx: usize) -> Result<Rc<RefCell<Memory>>> {
            if self.memory_enabled {
                if idx == 0 {
                    Ok(self.memory.clone())
                } else {
                    Err(anyhow!("Memory out of range"))
                }
            } else {
                Err(anyhow!("Memory not present in store"))
            }
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

    #[test]
    fn test_memory() {
        let mut stack = Stack::new();
        let mut store = TestStore::new();

        store.enable_memory();

        static FIXED_DATA: [u8; 8] = [0x0d, 0xf0, 0xad, 0xba, 0x0d, 0xf0, 0xad, 0xba];
        store
            .get_memory(0)
            .unwrap()
            .borrow_mut()
            .set_data(0, &FIXED_DATA)
            .unwrap();

        let mut expr_bytes = Vec::new();
        write_const_instruction(&mut expr_bytes, 0u32.into());

        expr_bytes.push(Opcode::I32Load.into());
        write_leb(&mut expr_bytes, 0, false);
        write_leb(&mut expr_bytes, 0, false);

        assert!(ExpressionExecutor::instance()
            .execute(&expr_bytes, &mut stack, &mut store)
            .is_ok());
        assert_eq!(stack.working_count(), 1);
        assert_eq!(stack.working_top(1)[0], 0xbaadf00du32.into());
        stack.pop();

        let mut expr_bytes = Vec::new();
        write_const_instruction(&mut expr_bytes, 0u32.into());
        write_const_instruction(&mut expr_bytes, 42.0f32.into());

        expr_bytes.push(Opcode::F32Store.into());
        write_leb(&mut expr_bytes, 0, false);
        write_leb(&mut expr_bytes, 0, false);

        assert!(ExpressionExecutor::instance()
            .execute(&expr_bytes, &mut stack, &mut store)
            .is_ok());
        assert_eq!(stack.working_count(), 0);

        let mut expr_bytes = Vec::new();
        write_const_instruction(&mut expr_bytes, 0u32.into());

        expr_bytes.push(Opcode::F32Load.into());
        write_leb(&mut expr_bytes, 0, false);
        write_leb(&mut expr_bytes, 0, false);

        assert!(ExpressionExecutor::instance()
            .execute(&expr_bytes, &mut stack, &mut store)
            .is_ok());
        assert_eq!(stack.working_count(), 1);
        assert_eq!(stack.working_top(1)[0], 42.0f32.into());
        stack.pop();

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

            store
                .get_memory(0)
                .unwrap()
                .borrow_mut()
                .set_data(128, &unsigned_bytes)
                .unwrap();
            store
                .get_memory(0)
                .unwrap()
                .borrow_mut()
                .set_data(256, &signed_bytes)
                .unwrap();

            let mut expr_bytes = Vec::new();
            write_const_instruction(&mut expr_bytes, 128u32.into());

            expr_bytes.push(unsigned_opcode.clone().into());
            write_leb(&mut expr_bytes, 0, false);
            write_leb(&mut expr_bytes, 0, false);

            assert!(ExpressionExecutor::instance()
                .execute(&expr_bytes, &mut stack, &mut store)
                .is_ok());
            assert_eq!(stack.working_count(), 1);
            assert_eq!(stack.working_top(1)[0], 0u32.into());
            stack.pop();

            let mut expr_bytes = Vec::new();
            write_const_instruction(&mut expr_bytes, 256u32.into());

            expr_bytes.push(signed_opcode.clone().into());
            write_leb(&mut expr_bytes, 0, false);
            write_leb(&mut expr_bytes, 0, false);

            assert!(ExpressionExecutor::instance()
                .execute(&expr_bytes, &mut stack, &mut store)
                .is_ok());
            assert_eq!(stack.working_count(), 1);
            assert_eq!(stack.working_top(1)[0], StackEntry::from(-1i32));
            stack.pop();
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

            store
                .get_memory(0)
                .unwrap()
                .borrow_mut()
                .set_data(128, &unsigned_bytes)
                .unwrap();
            store
                .get_memory(0)
                .unwrap()
                .borrow_mut()
                .set_data(256, &signed_bytes)
                .unwrap();

            let mut expr_bytes = Vec::new();
            write_const_instruction(&mut expr_bytes, 128u32.into());

            expr_bytes.push(unsigned_opcode.clone().into());
            write_leb(&mut expr_bytes, 0, false);
            write_leb(&mut expr_bytes, 0, false);

            assert!(ExpressionExecutor::instance()
                .execute(&expr_bytes, &mut stack, &mut store)
                .is_ok());
            assert_eq!(stack.working_count(), 1);
            assert_eq!(stack.working_top(1)[0], 0u64.into());
            stack.pop();

            let mut expr_bytes = Vec::new();
            write_const_instruction(&mut expr_bytes, 256u32.into());

            expr_bytes.push(signed_opcode.clone().into());
            write_leb(&mut expr_bytes, 0, false);
            write_leb(&mut expr_bytes, 0, false);

            assert!(ExpressionExecutor::instance()
                .execute(&expr_bytes, &mut stack, &mut store)
                .is_ok());
            assert_eq!(stack.working_count(), 1);
            assert_eq!(stack.working_top(1)[0], StackEntry::from(-1i64));
            stack.pop();
        }

        for (opcode, byte_count) in &[
            (Opcode::I32Store8, 1),
            (Opcode::I32Store16, 2),
            (Opcode::I32Store, 4),
        ] {
            let set_bytes: [u8; 8] = [0xff; 8];

            store
                .get_memory(0)
                .unwrap()
                .borrow_mut()
                .set_data(128, &set_bytes)
                .unwrap();

            let mut expr_bytes = Vec::new();
            write_const_instruction(&mut expr_bytes, 128u32.into());
            write_const_instruction(&mut expr_bytes, 0u32.into());

            expr_bytes.push(opcode.clone().into());
            write_leb(&mut expr_bytes, 0, false);
            write_leb(&mut expr_bytes, 0, false);

            assert!(ExpressionExecutor::instance()
                .execute(&expr_bytes, &mut stack, &mut store)
                .is_ok());
            assert_eq!(stack.working_count(), 0);

            let mut check_bytes: [u8; 8] = [0xff; 8];
            store
                .get_memory(0)
                .unwrap()
                .borrow()
                .get_data(128, &mut check_bytes)
                .unwrap();

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

            store
                .get_memory(0)
                .unwrap()
                .borrow_mut()
                .set_data(128, &set_bytes)
                .unwrap();

            let mut expr_bytes = Vec::new();
            write_const_instruction(&mut expr_bytes, 128u32.into());
            write_const_instruction(&mut expr_bytes, 0u64.into());

            expr_bytes.push(opcode.clone().into());
            write_leb(&mut expr_bytes, 0, false);
            write_leb(&mut expr_bytes, 0, false);

            assert!(ExpressionExecutor::instance()
                .execute(&expr_bytes, &mut stack, &mut store)
                .is_ok());
            assert_eq!(stack.working_count(), 0);

            let mut check_bytes: [u8; 8] = [0xff; 8];
            store
                .get_memory(0)
                .unwrap()
                .borrow()
                .get_data(128, &mut check_bytes)
                .unwrap();

            for i in 0..8 {
                if i < *byte_count {
                    assert_eq!(check_bytes[i], 0x00);
                } else {
                    assert_eq!(check_bytes[i], 0xff);
                }
            }
        }

        let mut expr_bytes = Vec::new();
        expr_bytes.push(Opcode::MemorySize.into());
        write_leb(&mut expr_bytes, 0, false);

        assert!(ExpressionExecutor::instance()
            .execute(&expr_bytes, &mut stack, &mut store)
            .is_ok());
        assert_eq!(stack.working_count(), 1);
        assert_eq!(stack.working_top(1)[0], 1u32.into());
        stack.pop();

        let mut expr_bytes = Vec::new();
        write_const_instruction(&mut expr_bytes, 1i32.into());
        expr_bytes.push(Opcode::MemoryGrow.into());
        write_leb(&mut expr_bytes, 0, false);

        assert!(ExpressionExecutor::instance()
            .execute(&expr_bytes, &mut stack, &mut store)
            .is_ok());
        assert_eq!(stack.working_count(), 1);
        assert_eq!(stack.working_top(1)[0], 1u32.into());
        stack.pop();

        assert_eq!(store.get_memory(0).unwrap().borrow().current_size(), 2);

        let mut expr_bytes = Vec::new();
        write_const_instruction(&mut expr_bytes, 10i32.into());
        expr_bytes.push(Opcode::MemoryGrow.into());
        write_leb(&mut expr_bytes, 0, false);

        assert!(ExpressionExecutor::instance()
            .execute(&expr_bytes, &mut stack, &mut store)
            .is_ok());
        assert_eq!(stack.working_count(), 1);
        assert_eq!(stack.working_top(1)[0], StackEntry::from(-1i32));
        stack.pop();

        assert_eq!(store.get_memory(0).unwrap().borrow().current_size(), 2);
    }
}
