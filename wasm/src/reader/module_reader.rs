use std::io::prelude::*;

use crate::core;
use crate::reader::{ReaderUtil, TypeReader};
use anyhow::{anyhow, Result};
use std::convert::TryFrom;

fn append_to_vector<R>(target: &mut Vec<R>, mut extra: Vec<R>) {
    target.append(&mut extra);
}

#[derive(Debug)]
pub struct ModuleBuilder {
    types: Vec<core::FuncType>,
    typeidx: Vec<usize>,
    funcs: Vec<core::Func>,
    tables: Vec<core::TableType>,
    mems: Vec<core::MemType>,
    globals: Vec<core::GlobalDef>,
    elem: Vec<core::Element>,
    data: Vec<core::Data>,
    start: Option<usize>,
    imports: Vec<core::Import>,
    exports: Vec<core::Export>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        ModuleBuilder {
            types: Vec::new(),
            typeidx: Vec::new(),
            funcs: Vec::new(),
            tables: Vec::new(),
            mems: Vec::new(),
            globals: Vec::new(),
            elem: Vec::new(),
            data: Vec::new(),
            start: None,
            imports: Vec::new(),
            exports: Vec::new(),
        }
    }

    pub fn process_section<T: Read>(
        &mut self,
        section_type: core::SectionType,
        reader: &mut T,
    ) -> anyhow::Result<()> {
        match section_type {
            core::SectionType::TypeSection => Ok(append_to_vector(
                &mut self.types,
                reader.read_vec(core::FuncType::read)?,
            )),
            core::SectionType::ImportSection => Ok(append_to_vector(
                &mut self.imports,
                reader.read_vec(core::Import::read)?,
            )),
            core::SectionType::FunctionSection => Ok(append_to_vector(
                &mut self.typeidx,
                reader.read_vec(T::read_leb_usize)?,
            )),
            core::SectionType::TableSection => Ok(append_to_vector(
                &mut self.tables,
                reader.read_vec(core::TableType::read)?,
            )),
            core::SectionType::MemorySection => Ok(append_to_vector(
                &mut self.mems,
                reader.read_vec(core::MemType::read)?,
            )),
            core::SectionType::GlobalSection => Ok(append_to_vector(
                &mut self.globals,
                reader.read_vec(core::GlobalDef::read)?,
            )),
            core::SectionType::ExportSection => Ok(append_to_vector(
                &mut self.exports,
                reader.read_vec(core::Export::read)?,
            )),
            core::SectionType::StartSection => {
                self.update_start(usize::try_from(reader.read_leb_u32()?).unwrap())
            }
            core::SectionType::ElementSection => Ok(append_to_vector(
                &mut self.elem,
                reader.read_vec(core::Element::read)?,
            )),
            core::SectionType::CodeSection => Ok(append_to_vector(
                &mut self.funcs,
                reader.read_vec(core::Func::read)?,
            )),
            core::SectionType::DataSection => Ok(append_to_vector(
                &mut self.data,
                reader.read_vec(core::Data::read)?,
            )),

            _ => panic!("Cannot read unknown or custom sections"),
        }
    }

    pub fn get_next_section_type(
        current_section_type: core::SectionType,
    ) -> Option<core::SectionType> {
        match current_section_type {
            core::SectionType::TypeSection => Some(core::SectionType::ImportSection),
            core::SectionType::ImportSection => Some(core::SectionType::FunctionSection),
            core::SectionType::FunctionSection => Some(core::SectionType::TableSection),
            core::SectionType::TableSection => Some(core::SectionType::MemorySection),
            core::SectionType::MemorySection => Some(core::SectionType::GlobalSection),
            core::SectionType::GlobalSection => Some(core::SectionType::ExportSection),
            core::SectionType::ExportSection => Some(core::SectionType::StartSection),
            core::SectionType::StartSection => Some(core::SectionType::ElementSection),
            core::SectionType::ElementSection => Some(core::SectionType::CodeSection),
            core::SectionType::CodeSection => Some(core::SectionType::DataSection),
            core::SectionType::DataSection => None,

            _ => panic!("Cannot read unknown or custom sections"),
        }
    }

    pub fn make_module(self) -> Result<core::RawModule> {
        if self.typeidx.len() == 0 {
            Err(anyhow!("No functions found"))
        } else if self.typeidx.len() != self.funcs.len() {
            Err(anyhow!("TypeIdx and code tables do not match sizes"))
        } else {
            Ok(core::RawModule::new(
                self.types,
                self.typeidx,
                self.funcs,
                self.tables,
                self.mems,
                self.globals,
                self.elem,
                self.data,
                self.start,
                self.imports,
                self.exports,
            ))
        }
    }

    fn update_start(&mut self, new_start: usize) -> Result<()> {
        if let Some(_) = self.start {
            Err(anyhow!("Multiple start sections found"))
        } else {
            self.start = Some(new_start);
            Ok(())
        }
    }

    pub fn read_next_section_header<T: Read>(reader: &mut T) -> Result<core::SectionType> {
        core::SectionType::read(reader)
    }
}
