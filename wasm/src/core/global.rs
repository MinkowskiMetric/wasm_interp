use crate::core;

#[derive(Debug)]
pub struct Global {}

impl Global {
    pub fn new(_global_def: core::GlobalDef) -> Self {
        Global {}
    }

    pub fn new_dummy(_global_type: core::GlobalType) -> Self {
        Global {}
    }
}

pub type RcGlobal = std::rc::Rc<Global>;
