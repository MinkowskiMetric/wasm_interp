use std::io;

use crate::core::{stack_entry::StackEntry, GlobalType, ValueType};

#[derive(Debug)]
pub struct Global {
    global_type: GlobalType,
    value: StackEntry,
}

fn check_value_type(global_type: &GlobalType, value: StackEntry) -> io::Result<StackEntry> {
    match global_type.value_type() {
        ValueType::I32 => match value {
            StackEntry::I32Entry(i) => Ok(StackEntry::I32Entry(i)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Global value type mismatch",
            )),
        },
        ValueType::I64 => match value {
            StackEntry::I64Entry(i) => Ok(StackEntry::I64Entry(i)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Global value type mismatch",
            )),
        },
        ValueType::F32 => match value {
            StackEntry::F32Entry(i) => Ok(StackEntry::F32Entry(i)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Global value type mismatch",
            )),
        },
        ValueType::F64 => match value {
            StackEntry::F64Entry(i) => Ok(StackEntry::F64Entry(i)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Global value type mismatch",
            )),
        },
    }
}

impl Global {
    pub fn new(global_type: GlobalType, value: StackEntry) -> io::Result<Self> {
        let value = check_value_type(&global_type, value)?;

        Ok(Global { global_type, value })
    }

    pub fn global_type(&self) -> &GlobalType {
        &self.global_type
    }

    pub fn is_mutable(&self) -> bool {
        self.global_type.is_mutable()
    }

    #[allow(dead_code)]
    pub fn value_type(&self) -> &ValueType {
        self.global_type.value_type()
    }

    pub fn get_value(&self) -> &StackEntry {
        &self.value
    }

    pub fn set_value(&mut self, value: StackEntry) -> io::Result<()> {
        if self.is_mutable() {
            self.value = check_value_type(self.global_type(), value)?;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Cannot mutate constant value",
            ))
        }
    }
}
