use std::{
    cmp::min,
    ops::{Index, IndexMut},
};

use crate::core::{memory_page::*, Limits, MemType};
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Memory {
    minimum_pages: usize,
    maximum_pages: Option<usize>,
    pages: Vec<MemoryPage>,
}

impl Memory {
    pub fn new(mem_type: MemType) -> Self {
        let (minimum_pages, maximum_pages): (usize, Option<usize>) = match mem_type.limits() {
            Limits::Bounded(minimum_pages, maximum_pages) => (*minimum_pages, Some(*maximum_pages)),
            Limits::Unbounded(minimum_pages) => (*minimum_pages, None),
        };

        Self::new_from_bounds(minimum_pages, maximum_pages)
    }

    pub fn new_from_bounds(minimum_pages: usize, maximum_pages: Option<usize>) -> Self {
        let mut pages = Vec::with_capacity(minimum_pages);
        for _ in 0..minimum_pages {
            pages.push(MemoryPage::new())
        }

        // Make the memory object
        Memory {
            minimum_pages,
            maximum_pages,
            pages,
        }
    }

    #[allow(dead_code)]
    pub fn min_size(&self) -> usize {
        self.minimum_pages
    }

    #[allow(dead_code)]
    pub fn max_size(&self) -> Option<usize> {
        self.maximum_pages
    }

    #[allow(dead_code)]
    pub fn current_size(&self) -> usize {
        self.pages.len()
    }

    pub fn grow_by(&mut self, grow_by: usize) -> Result<()> {
        match self.current_size().checked_add(grow_by) {
            Some(new_size) if new_size <= self.max_size().unwrap_or(new_size) => {
                for _ in 0..grow_by {
                    self.pages.push(MemoryPage::new())
                }

                Ok(())
            }

            _ => Err(anyhow!("New memory is too big")),
        }
    }

    pub fn set_data(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        self.check_bounds(offset, data.len())?;

        let (mut current_page, mut current_page_offset) = split_page_from_address(offset);
        let mut data_start = 0;
        let mut data_remaining = data.len();

        while data_remaining > 0 {
            let bytes_to_copy = min(
                data_remaining,
                WASM_PAGE_SIZE_IN_BYTES - current_page_offset,
            );
            let page = &mut self.pages[current_page];

            page[current_page_offset..current_page_offset + bytes_to_copy]
                .copy_from_slice(&data[data_start..data_start + bytes_to_copy]);

            data_start += bytes_to_copy;
            data_remaining -= bytes_to_copy;
            current_page += 1;
            current_page_offset = 0;
        }

        Ok(())
    }

    pub fn get_data(&self, offset: usize, data: &mut [u8]) -> Result<()> {
        self.check_bounds(offset, data.len())?;

        let (mut current_page, mut current_page_offset) = split_page_from_address(offset);
        let mut data_start = 0;
        let mut data_remaining = data.len();

        while data_remaining > 0 {
            let bytes_to_copy = min(
                data_remaining,
                WASM_PAGE_SIZE_IN_BYTES - current_page_offset,
            );
            let page = &self.pages[current_page];

            data[data_start..data_start + bytes_to_copy]
                .copy_from_slice(&page[current_page_offset..current_page_offset + bytes_to_copy]);

            data_start += bytes_to_copy;
            data_remaining -= bytes_to_copy;
            current_page += 1;
            current_page_offset = 0;
        }

        Ok(())
    }

    fn check_bounds(&self, offset: usize, length: usize) -> Result<()> {
        match offset.checked_add(length) {
            None => Err(anyhow!("Length overflow when accessing memory")),
            Some(end) if end > self.current_size() * WASM_PAGE_SIZE_IN_BYTES => Err(anyhow!("Attempting to access outside allocated memory")),
            _ => Ok(()),
        }
    }
}

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, address: usize) -> &Self::Output {
        let (page, offset) = split_page_from_address(address);

        let page = &self.pages[page];
        &page[offset]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, address: usize) -> &mut Self::Output {
        let (page, offset) = split_page_from_address(address);

        let page = &mut self.pages[page];
        &mut page[offset]
    }
}
