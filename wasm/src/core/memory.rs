use crate::core;

#[derive(Debug)]
pub struct Memory {

}

impl Memory {
    pub fn new(_mem_type: core::MemType) -> Self {
        Memory { }
    }
}

pub type RcMemory = std::rc::Rc<Memory>;
