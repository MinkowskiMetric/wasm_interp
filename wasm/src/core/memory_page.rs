use std::{
    fmt,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

const WASM_PAGE_SHIFT: usize = 16;
pub const WASM_PAGE_SIZE_IN_BYTES: usize = (1 << WASM_PAGE_SHIFT);
const WASM_PAGE_OFFSET_MASK: usize = WASM_PAGE_SIZE_IN_BYTES - 1;

pub fn split_page_from_address(address: usize) -> (usize, usize) {
    (address >> WASM_PAGE_SHIFT, address & WASM_PAGE_OFFSET_MASK)
}

// We allocate memory in pages. By having individual pages, it
// simplifies the lookup logic whilst keeping the grow time simple
pub struct MemoryPage {
    // Rust makes this super difficult to do because making a boxed array requires you to allocate
    // the array on the stack and then clone it into the box. Using a vector is annoying because
    // you need to check the size
    bytes: Vec<u8>,
}

impl MemoryPage {
    pub fn new() -> Self {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.resize(WASM_PAGE_SIZE_IN_BYTES, 0);
        MemoryPage { bytes }
    }
}

impl fmt::Debug for MemoryPage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MemoryPage {{ ... }}")
    }
}

impl Deref for MemoryPage {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl DerefMut for MemoryPage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for MemoryPage {
    type Output = <I as SliceIndex<[u8]>>::Output;

    fn index(&self, idx: I) -> &Self::Output {
        &self.bytes[idx]
    }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for MemoryPage {
    fn index_mut(&mut self, idx: I) -> &mut Self::Output {
        &mut self.bytes[idx]
    }
}
