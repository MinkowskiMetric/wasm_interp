use std::collections::HashMap;
use std::io;

use crate::core;

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
enum ExportValue {
    Function(core::RcCallable),
    Table(core::RcTable),
    Memory(core::RcMemory),
    Global(core::RcGlobal),
}

#[derive(Debug)]
pub struct Module {
    functions: Vec<core::RcCallable>,
    tables: Vec<core::RcTable>,
    memories: Vec<core::RcMemory>,
    globals: Vec<core::RcGlobal>,
    exports: HashMap<String, ExportValue>,
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
                .push(std::rc::Rc::new(core::WasmExprCallable::new(
                    metadata.types[type_idx].clone(),
                    func.clone(),
                )));
        }
        Ok(())
    }

    fn add_tables<Iter: Iterator<Item = core::TableType>>(
        &mut self,
        tables: Iter,
    ) -> io::Result<()> {
        for table in tables {
            self.tables.push(std::rc::Rc::new(core::Table::new(table)));
        }

        Ok(())
    }

    fn add_memories<Iter: Iterator<Item = core::MemType>>(
        &mut self,
        memories: Iter,
    ) -> io::Result<()> {
        for memory in memories {
            self.memories
                .push(std::rc::Rc::new(core::Memory::new(memory)));
        }

        Ok(())
    }

    fn add_globals<Iter: Iterator<Item = core::GlobalDef>>(
        &mut self,
        globals: Iter,
    ) -> io::Result<()> {
        for global in globals {
            self.globals
                .push(std::rc::Rc::new(core::Global::new(global)));
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

        Ok(ret_module)
    }
}
