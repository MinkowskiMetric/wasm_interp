use crate::core::{stack_entry::StackEntry, Global, Memory};
use anyhow::Result;
use std::{
    cell::{Ref, RefMut},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub trait LifetimeToRef<'a, T> {
    type Output: Deref<Target = T>;
}

pub trait LifetimeToRefMut<'a, T> {
    type Output: DerefMut<Target = T>;
}

#[allow(dead_code)]
pub struct RefType<T>(PhantomData<T>);

impl<'a, T: 'a> LifetimeToRef<'a, T> for RefType<T> {
    type Output = &'a T;
}

#[allow(dead_code)]
pub struct RefMutType<T>(PhantomData<T>);

impl<'a, T: 'a> LifetimeToRefMut<'a, T> for RefMutType<T> {
    type Output = &'a mut T;
}

#[allow(dead_code)]
pub struct CellRefType<T>(PhantomData<T>);

impl<'a, T: 'a> LifetimeToRef<'a, T> for CellRefType<T> {
    type Output = Ref<'a, T>;
}

#[allow(dead_code)]
pub struct CellRefMutType<T>(PhantomData<T>);

impl<'a, T: 'a> LifetimeToRefMut<'a, T> for CellRefMutType<T> {
    type Output = RefMut<'a, T>;
}

pub trait ConstantExpressionStore {
    type GlobalRef: for<'a> LifetimeToRef<'a, Global>;

    fn global_idx<'a>(
        &'a self,
        idx: usize,
    ) -> Result<<Self::GlobalRef as LifetimeToRef<'a, Global>>::Output>;

    fn get_global_value(&self, idx: usize) -> Result<StackEntry> {
        Ok(self.global_idx(idx)?.get_value().clone())
    }
}

pub trait ExpressionStore: ConstantExpressionStore {
    type GlobalRefMut: for<'a> LifetimeToRefMut<'a, Global>;

    type MemoryRef: for<'a> LifetimeToRef<'a, Memory>;
    type MemoryRefMut: for<'a> LifetimeToRefMut<'a, Memory>;

    fn global_idx_mut<'a>(
        &'a mut self,
        idx: usize,
    ) -> Result<<Self::GlobalRefMut as LifetimeToRefMut<'a, Global>>::Output>;

    fn mem_idx<'a>(
        &'a self,
        idx: usize,
    ) -> Result<<Self::MemoryRef as LifetimeToRef<'a, Memory>>::Output>;
    fn mem_idx_mut<'a>(
        &'a mut self,
        idx: usize,
    ) -> Result<<Self::MemoryRefMut as LifetimeToRefMut<'a, Memory>>::Output>;

    fn set_global_value(&mut self, idx: usize, value: StackEntry) -> Result<()> {
        self.global_idx_mut(idx)?.set_value(value)
    }

    fn read_data(&self, mem_idx: usize, offset: usize, data: &mut [u8]) -> Result<()> {
        self.mem_idx(mem_idx)?.get_data(offset, data)
    }

    fn write_data(&mut self, mem_idx: usize, offset: usize, data: &[u8]) -> Result<()> {
        self.mem_idx_mut(mem_idx)?.set_data(offset, data)
    }

    fn get_memory_size(&self, mem_idx: usize) -> Result<usize> {
        Ok(self.mem_idx(mem_idx)?.current_size())
    }

    fn grow_memory_by(&mut self, mem_idx: usize, grow_by: usize) -> Result<()> {
        self.mem_idx_mut(mem_idx)?.grow_by(grow_by)
    }
}
