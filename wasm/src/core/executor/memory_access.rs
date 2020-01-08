use std::convert::TryFrom;

use crate::core::{stack_entry::StackEntry, Stack};
use crate::parser::Instruction;
use anyhow::Result;
use generic_array::typenum::consts::{U1, U2, U4, U8};
use generic_array::{ArrayLength, GenericArray};

use super::stack_ops::get_stack_top;
use super::ExpressionStore;

pub trait LEByteConvert {
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

pub fn mem_load<
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

    let base_address = get_stack_top(stack, 1)?[0];
    let base_address = usize::try_from(u32::try_from(base_address)?).unwrap();
    stack.pop();

    let final_address = base_address + offset;

    // A limitaton of the rust syntax here means you can't make the array the correct
    // size. Which is a bit annoying, but not very.
    let mut bytes: GenericArray<u8, IntType::ArrayLength> =
        unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    store.read_data(mem_idx, final_address, &mut bytes)?;

    let int_value = IntType::from_bytes(bytes);
    let ret_value = func(int_value);

    stack.push(ret_value.into());

    Ok(())
}

pub fn mem_store<
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

    let value = get_stack_top(stack, 1)?[0];
    let value = ValueType::try_from(value)?;
    stack.pop();

    let base_address = get_stack_top(stack, 1)?[0];
    let base_address = usize::try_from(u32::try_from(base_address)?).unwrap();
    stack.pop();

    let final_address = base_address + offset;

    let bytes = func(value).to_bytes();
    store.write_data(mem_idx, final_address, &bytes)?;

    Ok(())
}
