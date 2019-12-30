use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io;
use std::rc::Rc;

use crate::core::{self, Callable, Global, Memory, Table};

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
    start: Option<u32>,
    imports: Vec<core::Import>,
    exports: Vec<core::Export>,
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
        start: Option<u32>,
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
pub struct Module {
    pub functions: Vec<Rc<RefCell<Callable>>>,
    pub tables: Vec<Rc<RefCell<Table>>>,
    pub memories: Vec<Rc<RefCell<Memory>>>,
    pub globals: Vec<Rc<RefCell<Global>>>,
    pub exports: HashMap<String, ExportValue>,
}

impl Module {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            exports: HashMap::new(),
        }
    }

    fn resolve_imports<Iter: Iterator<Item = core::Import>, Resolver: core::Resolver>(
        &mut self,
        imports: Iter,
        metadata: &RawModuleMetadata,
        resolver: &Resolver,
    ) -> io::Result<()> {
        for import in imports {
            match import.desc() {
                core::ImportDesc::TypeIdx(type_index) => {
                    if *type_index >= metadata.types.len() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Function import {} from module {} has invalid type index",
                                import.mod_name(),
                                import.name()
                            ),
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
            }
        }

        Ok(())
    }

    fn add_functions<Iter: Iterator<Item = (usize, core::Func)>>(
        &mut self,
        functions: Iter,
        metadata: &RawModuleMetadata,
    ) -> io::Result<()> {
        for (type_idx, func) in functions {
            if type_idx >= metadata.types.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Function has invalid type index",
                ));
            }

            self.functions
                .push(Rc::new(RefCell::new(core::WasmExprCallable::new(
                    metadata.types[type_idx].clone(),
                    func.clone(),
                ))));
        }
        Ok(())
    }

    fn add_tables<Iter: Iterator<Item = core::TableType>>(
        &mut self,
        tables: Iter,
    ) -> io::Result<()> {
        for table in tables {
            self.tables.push(Rc::new(RefCell::new(Table::new(table))));
        }

        Ok(())
    }

    fn add_memories<Iter: Iterator<Item = core::MemType>>(
        &mut self,
        memories: Iter,
    ) -> io::Result<()> {
        for memory in memories {
            self.memories
                .push(Rc::new(RefCell::new(Memory::new(memory))));
        }

        Ok(())
    }

    fn add_globals<Iter: Iterator<Item = core::GlobalDef>>(
        &mut self,
        globals: Iter,
    ) -> io::Result<()> {
        for global in globals {
            self.globals
                .push(Rc::new(RefCell::new(Global::new(global))));
        }

        Ok(())
    }

    fn collect_single_export<T>(
        idx: usize,
        items: &Vec<std::rc::Rc<T>>,
    ) -> io::Result<std::rc::Rc<T>> {
        if idx >= items.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Export has invalid index",
            ));
        }

        Ok(items[idx].clone())
    }

    fn collect_exports<Iter: Iterator<Item = core::Export>>(
        &mut self,
        exports: Iter,
    ) -> io::Result<()> {
        for core::Export { nm, d } in exports {
            match d {
                core::ExportDesc::Func(idx) => {
                    self.exports.insert(
                        nm,
                        ExportValue::Function(Self::collect_single_export(idx, &self.functions)?),
                    );
                }
                core::ExportDesc::Table(idx) => {
                    self.exports.insert(
                        nm,
                        ExportValue::Table(Self::collect_single_export(idx, &self.tables)?),
                    );
                }
                core::ExportDesc::Mem(idx) => {
                    self.exports.insert(
                        nm,
                        ExportValue::Memory(Self::collect_single_export(idx, &self.memories)?),
                    );
                }
                core::ExportDesc::Global(idx) => {
                    self.exports.insert(
                        nm,
                        ExportValue::Global(Self::collect_single_export(idx, &self.globals)?),
                    );
                }
            }
        }

        Ok(())
    }

    fn pre_execute_validate(&self) -> io::Result<()> {
        if self.tables.len() > 1 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Too many tables",
            ))
        } else if self.memories.len() > 1 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Too many memoryies",
            ))
        } else {
            Ok(())
        }
    }

    fn initialize_table_element(&self, element: core::Element) -> io::Result<()> {
        if element.table_idx() >= self.tables.len() {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Table initializer table idx out of range",
            ))
        } else {
            let table = &self.tables[element.table_idx()];
            let offset = usize::try_from(
                core::ConstantExpressionExecutor::instance()
                    .execute_constant_expression(element.expr(), self)?,
            )
            .unwrap();

            let functions = element.func_indices();
            let functions: io::Result<Vec<_>> = functions
                .into_iter()
                .map(|idx| {
                    if *idx < self.functions.len() {
                        Ok(self.functions[*idx].clone())
                    } else {
                        Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Function index out of range",
                        ))
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
    ) -> io::Result<()> {
        for element in iter {
            self.initialize_table_element(element)?;
        }

        Ok(())
    }

    fn initialize_memory_data(&self, data: core::Data) -> io::Result<()> {
        if data.mem_idx() >= self.memories.len() {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Memory initializer mem idx out of range",
            ))
        } else {
            let memory = &self.memories[data.mem_idx()];
            let offset = usize::try_from(
                core::ConstantExpressionExecutor::instance()
                    .execute_constant_expression(data.expr(), self)?,
            )
            .unwrap();

            let data = data.bytes();

            memory.borrow_mut().set_data(offset, data);

            Ok(())
        }
    }

    fn initialize_memory<Iter: Iterator<Item = core::Data>>(&self, iter: Iter) -> io::Result<()> {
        for data in iter {
            self.initialize_memory_data(data)?;
        }

        Ok(())
    }

    pub fn resolve_raw_module<Resolver: core::Resolver>(
        module: RawModule,
        resolver: &Resolver,
    ) -> io::Result<Module> {
        let mut ret_module = Self::new();
        ret_module.resolve_imports(module.imports.into_iter(), &module.metadata, resolver)?;
        ret_module.add_functions(
            module.typeidx.into_iter().zip(module.funcs.into_iter()),
            &module.metadata,
        )?;
        ret_module.add_tables(module.tables.into_iter())?;
        ret_module.add_memories(module.mems.into_iter())?;
        ret_module.add_globals(module.globals.into_iter())?;
        ret_module.collect_exports(module.exports.into_iter())?;

        // Everything prior to this point is setting up the environment so that we
        // can start executing things, so make sure that everything is sane once we're
        // at that point.
        ret_module.pre_execute_validate()?;

        // The next step is to initialize the tables and memories.
        ret_module.initialize_table_elements(module.elem.into_iter())?;
        ret_module.initialize_memory(module.data.into_iter())?;

        // Finally, if there is a start function specified then execute it.
        // TODOTODOTODO - execute the start function

        println!("{:?}", module.metadata);
        //println!("{:?}", module.typeidx);
        //println!("{:?}", module.funcs);
        //println!("{:?}", module.tables);
        //println!("{:?}", module.mems);
        //println!("{:?}", module.globals);
        //println!("{:?}", module.elem);
        //println!("{:?}", module.data);
        println!("{:?}", module.start);
        //println!("{:?}", module.imports);
        //println!("{:?}", module.exports);
        // println!("{:?}", module);

        Ok(ret_module)
    }
}
