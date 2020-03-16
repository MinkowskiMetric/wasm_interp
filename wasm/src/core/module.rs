use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::rc::Rc;

use crate::core::{
    self, evaluate_constant_expression, stack_entry::StackEntry, Callable, ConstantDataStore,
    DataStore, FuncType, FunctionStore, Global, Memory, Stack, Table,
};
use crate::parser::InstructionSource;
use crate::reader::{ModuleBuilder, ReaderUtil, ScopedReader, TypeReader};

fn is_data_import(import: &core::Import) -> bool {
    match import.desc() {
        core::ImportDesc::MemType(_) | core::ImportDesc::GlobalType(_) => true,

        _ => false,
    }
}

fn is_data_export(export: &core::ExportDesc) -> bool {
    match export {
        core::ExportDesc::Mem(_) | core::ExportDesc::Global(_) => true,

        _ => false,
    }
}

#[derive(Debug)]
struct RawModuleMetadata {
    types: Vec<core::FuncType>,
}

#[derive(Debug)]
pub struct RawModule {
    metadata: RawModuleMetadata,
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

impl TypeReader for core::RawModule {
    fn read<T: Read>(reader: &mut T) -> Result<Self> {
        const HEADER_LENGTH: usize = 8;
        const EXPECTED_HEADER: [u8; 8] = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

        let mut header: [u8; HEADER_LENGTH] = [0; HEADER_LENGTH];

        // Read in the header
        reader.read_exact(&mut header)?;

        if header != EXPECTED_HEADER {
            Err(anyhow!("Invalid module header"))
        } else {
            let mut current_section_type: Option<core::SectionType> =
                Some(core::SectionType::TypeSection);
            let mut module_builder = ModuleBuilder::new();

            loop {
                if let Ok(section_type) = ModuleBuilder::read_next_section_header(reader) {
                    // Read the section length
                    let section_length = usize::try_from(reader.read_leb_u32()?).unwrap();
                    // And make a scoped reader for the section
                    let mut section_reader = ScopedReader::new(reader, section_length);

                    // Always skip custom sections wherever they appear
                    if section_type == core::SectionType::CustomSection {
                        // Read the section name
                        let section_name = section_reader.read_name()?;
                        let _section_body = section_reader.read_bytes_to_end()?;

                        println!("Skipping custom section \"{}\"", section_name);
                    } else {
                        while let Some(expected_section_type) = current_section_type {
                            if expected_section_type == section_type {
                                // This is the correct section type so we process it and move on
                                module_builder
                                    .process_section(section_type, &mut section_reader)?;

                                // And the next section type is the same as this one
                                current_section_type = Some(expected_section_type);
                                break;
                            } else {
                                // The section type doesn't match, so we move on to see if it
                                // is the next valid section
                                current_section_type =
                                    ModuleBuilder::get_next_section_type(expected_section_type);
                            }
                        }

                        if current_section_type == None {
                            assert!(false, "Sections are in unexpected order");
                            return Err(anyhow!("Invalid section order"));
                        }
                    }

                    if !section_reader.is_at_end() {
                        assert!(false, "Failed to read whole section");
                        return Err(anyhow!("Failed to read whole section"));
                    }
                } else {
                    // End of file, so we can break out of the loop
                    break;
                }
            }

            module_builder.make_module()
        }
    }
}

impl RawModule {
    pub fn new(
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
    ) -> Self {
        Self {
            metadata: RawModuleMetadata { types },
            typeidx,
            funcs,
            tables,
            mems,
            globals,
            elem,
            data,
            start,
            imports,
            exports,
        }
    }
}

#[derive(Debug)]
pub enum ExportValue {
    Function(Rc<RefCell<Callable>>),
    Table(Rc<RefCell<Table>>),
    Memory(Rc<RefCell<Memory>>),
    Global(Rc<RefCell<Global>>),
}

#[derive(Debug)]
pub struct DataModule {
    pub memories: Vec<Rc<RefCell<Memory>>>,
    pub globals: Vec<Rc<RefCell<Global>>>,
}

impl DataModule {
    pub fn new() -> Self {
        Self {
            memories: Vec::new(),
            globals: Vec::new(),
        }
    }

    fn pre_execute_validate(&self) -> Result<()> {
        if self.memories.len() > 1 {
            Err(anyhow!("Too many memories"))
        } else {
            Ok(())
        }
    }

    fn add_memories<Iter: Iterator<Item = core::MemType>>(&mut self, memories: Iter) -> Result<()> {
        for memory in memories {
            self.memories
                .push(Rc::new(RefCell::new(Memory::new(memory))));
        }

        Ok(())
    }

    fn add_globals(&mut self, globals: impl Iterator<Item = core::GlobalDef>) -> Result<()> {
        for global in globals {
            let global_type = global.global_type().clone();
            let init_expr = global.init_expr();

            let results = evaluate_constant_expression(init_expr, self, 1)?;
            let global = Global::new(global_type, results[0])?;

            self.globals.push(Rc::new(RefCell::new(global)));
        }

        Ok(())
    }

    fn evaluate_offset_expression(&self, expr: &impl InstructionSource) -> Result<usize> {
        let result = evaluate_constant_expression(expr, self, 1)?;

        match result[0] {
            StackEntry::I32Entry(i) => Ok(usize::try_from(i).unwrap()),
            _ => Err(anyhow!("Type mismatch in offset expression")),
        }
    }

    fn initialize_memory_data(&self, data: core::Data) -> Result<()> {
        if data.mem_idx() >= self.memories.len() {
            Err(anyhow!("Memory initializer mem idx out of range"))
        } else {
            let memory = &self.memories[data.mem_idx()];
            let offset = self.evaluate_offset_expression(data.expr())?;

            let data = data.bytes();

            memory.borrow_mut().set_data(offset, data)?;

            Ok(())
        }
    }

    fn initialize_memory<Iter: Iterator<Item = core::Data>>(&self, iter: Iter) -> Result<()> {
        for data in iter {
            self.initialize_memory_data(data)?;
        }

        Ok(())
    }

    fn resolve_import<Resolver: core::Resolver>(
        &mut self,
        import: core::Import,
        resolver: &Resolver,
    ) -> Result<()> {
        match import.desc() {
            core::ImportDesc::MemType(mem_type) => {
                let resolved_memory =
                    resolver.resolve_memory(import.mod_name(), import.name(), mem_type)?;
                self.memories.push(resolved_memory);
            }
            core::ImportDesc::GlobalType(global_type) => {
                let resolved_global =
                    resolver.resolve_global(import.mod_name(), import.name(), global_type)?;
                self.globals.push(resolved_global);
            }

            _ => panic!("Not a data import"),
        }

        Ok(())
    }

    fn collect_export(&self, desc: core::ExportDesc) -> Result<ExportValue> {
        match desc {
            core::ExportDesc::Mem(idx) => {
                if idx < self.memories.len() {
                    Ok(ExportValue::Memory(self.memories[idx].clone()))
                } else {
                    Err(anyhow!("Exported memory index {} out of range", idx))
                }
            }
            core::ExportDesc::Global(idx) => {
                if idx < self.globals.len() {
                    Ok(ExportValue::Global(self.globals[idx].clone()))
                } else {
                    Err(anyhow!("Exported global index {} out of range", idx))
                }
            }

            _ => panic!("Not a data export"),
        }
    }
}

impl ConstantDataStore for DataModule {
    fn get_global_value(&self, idx: usize) -> Result<StackEntry> {
        if idx < self.globals.len() {
            Ok(self.globals[idx].borrow().get_value().clone())
        } else {
            Err(anyhow!("Global index out of range"))
        }
    }
}

impl DataStore for DataModule {
    fn set_global_value(&mut self, idx: usize, value: StackEntry) -> Result<()> {
        if idx < self.globals.len() {
            self.globals[idx].borrow_mut().set_value(value)
        } else {
            Err(anyhow!("Global index out of range"))
        }
    }

    fn read_data(&self, mem_idx: usize, offset: usize, data: &mut [u8]) -> Result<()> {
        if mem_idx < self.memories.len() {
            self.memories[mem_idx].borrow().get_data(offset, data)
        } else {
            Err(anyhow!("Memory index out of range"))
        }
    }

    fn write_data(&mut self, mem_idx: usize, offset: usize, data: &[u8]) -> Result<()> {
        if mem_idx < self.memories.len() {
            self.memories[mem_idx].borrow_mut().set_data(offset, data)
        } else {
            Err(anyhow!("Memory index out of range"))
        }
    }

    fn get_memory_size(&self, mem_idx: usize) -> Result<usize> {
        if mem_idx < self.memories.len() {
            Ok(self.memories[mem_idx].borrow().current_size())
        } else {
            Err(anyhow!("Memory index out of range"))
        }
    }

    fn grow_memory_by(&mut self, mem_idx: usize, grow_by: usize) -> Result<()> {
        if mem_idx < self.memories.len() {
            self.memories[mem_idx].borrow_mut().grow_by(grow_by)
        } else {
            Err(anyhow!("Memory index out of range"))
        }
    }
}

#[derive(Debug)]
pub struct FunctionModule {
    pub functions: Vec<Rc<RefCell<Callable>>>,
    pub tables: Vec<Rc<RefCell<Table>>>,
    func_types: Vec<FuncType>,
}

impl FunctionModule {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
            tables: Vec::new(),
            func_types: Vec::new(),
        }
    }

    fn pre_execute_validate(&self) -> Result<()> {
        if self.tables.len() > 1 {
            Err(anyhow!("Too many tables"))
        } else {
            Ok(())
        }
    }

    fn add_functions<Iter: Iterator<Item = (usize, core::Func)>>(
        &mut self,
        functions: Iter,
        metadata: &RawModuleMetadata,
    ) -> Result<()> {
        for (type_idx, func) in functions {
            if type_idx >= metadata.types.len() {
                return Err(anyhow!("Function has invalid type index"));
            }

            self.functions
                .push(Rc::new(RefCell::new(core::WasmExprCallable::new(
                    metadata.types[type_idx].clone(),
                    func.clone(),
                ))));
        }
        Ok(())
    }

    fn add_tables<Iter: Iterator<Item = core::TableType>>(&mut self, tables: Iter) -> Result<()> {
        for table in tables {
            self.tables.push(Rc::new(RefCell::new(Table::new(table))));
        }

        Ok(())
    }

    fn add_func_types(&mut self, func_types: Vec<FuncType>) -> Result<()> {
        self.func_types = func_types;
        Ok(())
    }

    fn initialize_table_element(
        &self,
        element: core::Element,
        data_module: &DataModule,
    ) -> Result<()> {
        if element.table_idx() >= self.tables.len() {
            Err(anyhow!("Table initializer table idx out of range"))
        } else {
            let table = &self.tables[element.table_idx()];
            let offset = data_module.evaluate_offset_expression(element.expr())?;

            let functions = element.func_indices();
            let functions: Result<Vec<_>> = functions
                .into_iter()
                .map(|idx| {
                    if *idx < self.functions.len() {
                        Ok(self.functions[*idx].clone())
                    } else {
                        Err(anyhow!("Function index out of range"))
                    }
                })
                .collect();
            let functions = functions?;

            table.borrow_mut().set_entries(offset, &functions);

            Ok(())
        }
    }

    fn initialize_table_elements<Iter: Iterator<Item = core::Element>>(
        &self,
        iter: Iter,
        data_module: &DataModule,
    ) -> Result<()> {
        for element in iter {
            self.initialize_table_element(element, data_module)?;
        }

        Ok(())
    }

    fn resolve_import<Resolver: core::Resolver>(
        &mut self,
        import: core::Import,
        metadata: &RawModuleMetadata,
        resolver: &Resolver,
    ) -> Result<()> {
        match import.desc() {
            core::ImportDesc::TypeIdx(type_index) => {
                if *type_index >= metadata.types.len() {
                    return Err(anyhow!(
                        "Function import {} from module {} has invalid type index",
                        import.mod_name(),
                        import.name()
                    ));
                }

                let resolved_function = resolver.resolve_function(
                    import.mod_name(),
                    import.name(),
                    &metadata.types[*type_index],
                )?;
                self.functions.push(resolved_function);
            }
            core::ImportDesc::TableType(table_type) => {
                let resolved_table =
                    resolver.resolve_table(import.mod_name(), import.name(), table_type)?;
                self.tables.push(resolved_table);
            }

            _ => panic!("Not a function import"),
        }

        Ok(())
    }

    fn collect_export(&self, desc: core::ExportDesc) -> Result<ExportValue> {
        match desc {
            core::ExportDesc::Func(idx) => {
                if idx < self.functions.len() {
                    Ok(ExportValue::Function(self.functions[idx].clone()))
                } else {
                    Err(anyhow!("Exported function index {} out of range", idx))
                }
            }
            core::ExportDesc::Table(idx) => {
                if idx < self.tables.len() {
                    Ok(ExportValue::Table(self.tables[idx].clone()))
                } else {
                    Err(anyhow!("Exported table index {} out of range", idx))
                }
            }

            _ => panic!("Not a function export"),
        }
    }
}

impl FunctionStore for FunctionModule {
    fn execute_function(
        &self,
        idx: usize,
        stack: &mut Stack,
        data_store: &mut impl DataStore,
    ) -> Result<()> {
        if idx < self.functions.len() {
            let callable = self.functions[idx].borrow();
            callable.call(stack, self, data_store)
        } else {
            Err(anyhow!("Callable index out of range"))
        }
    }

    fn execute_indirect_function(
        &self,
        func_type_idx: usize,
        table_idx: usize,
        elem_idx: usize,
        stack: &mut Stack,
        data_store: &mut impl DataStore,
    ) -> Result<()> {
        if func_type_idx >= self.func_types.len() {
            Err(anyhow!("FuncType index out of range"))
        } else if table_idx >= self.tables.len() {
            Err(anyhow!("Table index out of range"))
        } else {
            let table = self.tables[table_idx].borrow();
            let callable = table.get_entry(elem_idx)?;
            let callable = callable.borrow();

            if *callable.func_type() != self.func_types[func_type_idx] {
                Err(anyhow!("Indirect function call type does not match"))
            } else {
                callable.call(stack, self, data_store)
            }
        }
    }
}

fn resolve_imports<Iter: Iterator<Item = core::Import>, Resolver: core::Resolver>(
    function_module: &mut FunctionModule,
    data_module: &mut DataModule,
    imports: Iter,
    metadata: &RawModuleMetadata,
    resolver: &Resolver,
) -> Result<()> {
    for import in imports {
        if is_data_import(&import) {
            data_module.resolve_import(import, resolver)?;
        } else {
            function_module.resolve_import(import, metadata, resolver)?;
        }
    }

    Ok(())
}

fn collect_exports<Iter: Iterator<Item = core::Export>>(
    function_module: &FunctionModule,
    data_module: &DataModule,
    exports: Iter,
) -> Result<HashMap<String, ExportValue>> {
    let mut ret = HashMap::new();

    for core::Export { nm, d } in exports {
        if is_data_export(&d) {
            ret.insert(nm, data_module.collect_export(d)?);
        } else {
            ret.insert(nm, function_module.collect_export(d)?);
        }
    }

    Ok(ret)
}

type LoadedModule = (FunctionModule, DataModule, HashMap<String, ExportValue>);

pub fn resolve_raw_module<Resolver: core::Resolver>(
    module: RawModule,
    resolver: &Resolver,
) -> Result<LoadedModule> {
    let mut data_module = DataModule::new();
    let mut function_module = FunctionModule::new();

    resolve_imports(
        &mut function_module,
        &mut data_module,
        module.imports.into_iter(),
        &module.metadata,
        resolver,
    )?;
    function_module.add_functions(
        module.typeidx.into_iter().zip(module.funcs.into_iter()),
        &module.metadata,
    )?;
    function_module.add_tables(module.tables.into_iter())?;
    data_module.add_memories(module.mems.into_iter())?;
    data_module.add_globals(module.globals.into_iter())?;
    let exports = collect_exports(&function_module, &data_module, module.exports.into_iter())?;
    function_module.add_func_types(module.metadata.types)?;

    // Everything prior to this point is setting up the environment so that we
    // can start executing things, so make sure that everything is sane once we're
    // at that point.
    data_module.pre_execute_validate()?;
    function_module.pre_execute_validate()?;

    // The next step is to initialize the tables and memories.
    function_module.initialize_table_elements(module.elem.into_iter(), &data_module)?;
    data_module.initialize_memory(module.data.into_iter())?;

    // Finally, if there is a start function specified then execute it.
    if let Some(start) = module.start {
        let mut stack = Stack::new();
        function_module.execute_function(start, &mut stack, &mut data_module)?;
    }

    Ok((function_module, data_module, exports))
}

pub fn load_module_from_path(file: &str, resolver: &impl core::Resolver) -> Result<LoadedModule> {
    let mut buf = BufReader::new(File::open(file)?);
    let raw_module = core::RawModule::read(&mut buf)?;
    resolve_raw_module(raw_module, resolver)
}
