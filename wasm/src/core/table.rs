use std::{
    cell::RefCell,
    ops::{Index, IndexMut},
    rc::Rc,
    slice::SliceIndex,
};

use crate::core::{Callable, ElemType, Limits, TableType};

type RefCallable = Rc<RefCell<Callable>>;
type OptRefCallable = Option<RefCallable>;

#[derive(Debug)]
pub struct Table {
    minimum_entries: usize,
    maximum_entries: Option<usize>,
    entries: Vec<OptRefCallable>,
}

impl Table {
    pub fn new(table_type: TableType) -> Self {
        assert!(*table_type.elem_type() == ElemType::FuncRef);

        let (minimum_entries, maximum_entries): (usize, Option<usize>) = match table_type.limits() {
            Limits::Bounded(minimum_entries, maximum_entries) => {
                (*minimum_entries, Some(*maximum_entries))
            }
            Limits::Unbounded(minimum_entries) => (*minimum_entries, None),
        };

        let mut entries = Vec::with_capacity(minimum_entries);
        for _ in 0..minimum_entries {
            entries.push(None)
        }

        // Make the memory object
        Table {
            minimum_entries,
            maximum_entries,
            entries,
        }
    }

    #[allow(dead_code)]
    pub fn min_size(&self) -> usize {
        self.minimum_entries
    }

    #[allow(dead_code)]
    pub fn max_size(&self) -> Option<usize> {
        self.maximum_entries
    }

    #[allow(dead_code)]
    pub fn current_size(&self) -> usize {
        self.entries.len()
    }

    pub fn set_entries(&mut self, offset: usize, functions: &[RefCallable]) {
        for (idx, value) in functions.iter().enumerate() {
            self.entries[offset + idx] = Some(value.clone());
        }
    }
}

impl<I: SliceIndex<[OptRefCallable]>> Index<I> for Table {
    type Output = I::Output;

    fn index(&self, idx: I) -> &Self::Output {
        &self.entries[idx]
    }
}

impl<I: SliceIndex<[OptRefCallable]>> IndexMut<I> for Table {
    fn index_mut(&mut self, idx: I) -> &mut Self::Output {
        &mut self.entries[idx]
    }
}
