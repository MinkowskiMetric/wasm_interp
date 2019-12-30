use crate::core::MemType;

#[derive(Debug)]
pub struct Memory {}

impl Memory {
    pub fn new(_mem_type: MemType) -> Self {
        Memory {}
    }

    pub fn set_data(&mut self, offset: usize, data: &[u8]) {
        println!("mem: {:?} offset: {:?} data: {:?}", self, offset, data);
    }
}
