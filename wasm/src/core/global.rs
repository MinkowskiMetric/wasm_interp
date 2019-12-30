use crate::core::{GlobalDef, GlobalType};

#[derive(Debug)]
pub struct Global {}

impl Global {
    pub fn new(_global_def: GlobalDef) -> Self {
        Global {}
    }

    pub fn new_dummy(_global_type: GlobalType) -> Self {
        Global {}
    }
}
