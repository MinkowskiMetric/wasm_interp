mod core;
mod parser;
mod reader;

use std::env;

#[cfg(test)]
use anyhow::{Result, anyhow};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("wasm [mod_name]");
    } else {
        match core::Module::load_module_from_path(&args[1], core::EmptyResolver::instance()) {
            Err(e) => println!("Failed to read module from {} - {}", &args[1], e),
            Ok(module) => {
                println!("Module {:?}", module);
                println!("Done");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::{cell::RefCell, rc::Rc};

    use crate::core::{
        Callable, FuncType, Global, GlobalType, MemType, Memory, MutableType, Table, TableType,
        ValueType,
    };

    struct TestResolver {
        global_zero: Rc<RefCell<Global>>,
    }

    impl TestResolver {
        pub fn new() -> Self {
            let global_zero_type = GlobalType::new(ValueType::I32, MutableType::Const);
            let global_zero = Global::new(global_zero_type, 0u32.into()).unwrap();
            let global_zero = Rc::new(RefCell::new(global_zero));

            TestResolver { global_zero }
        }
    }

    impl core::Resolver for TestResolver {
        fn resolve_function(
            &self,
            mod_name: &str,
            name: &str,
            _func_type: &FuncType,
        ) -> Result<Rc<RefCell<Callable>>> {
            Err(anyhow!("Imported function {}:{} not found", mod_name, name))
        }
        fn resolve_table(
            &self,
            mod_name: &str,
            name: &str,
            _table_type: &TableType,
        ) -> Result<Rc<RefCell<Table>>> {
            Err(anyhow!("Imported table {}:{} not found", mod_name, name))
        }
        fn resolve_memory(
            &self,
            mod_name: &str,
            name: &str,
            _mem_type: &MemType,
        ) -> Result<Rc<RefCell<Memory>>> {
            Err(anyhow!("Imported memory {}:{} not found", mod_name, name))
        }
        fn resolve_global(
            &self,
            mod_name: &str,
            name: &str,
            global_type: &GlobalType,
        ) -> Result<Rc<RefCell<Global>>> {
            if mod_name == "test" && name == "zero" {
                if global_type.clone() == self.global_zero.borrow().global_type().clone() {
                    Ok(self.global_zero.clone())
                } else {
                    Err(anyhow!("Global import {}:{} type mismatch", mod_name, name))
                }
            } else {
                Err(anyhow!("Imported global {}:{} not found", mod_name, name))
            }
        }
    }

    #[test]
    fn test_load_module() -> std::result::Result<(), String> {
        let resolver = TestResolver::new();

        match core::Module::load_module_from_path("../test_app/test.wasm", &resolver) {
            Ok(m) => {
                assert_eq!(m.exports.len(), 4);
                assert!(m.exports.contains_key("fib"));
                assert!(m.exports.contains_key("fib7"));
                assert!(m.exports.contains_key("zero"));
                assert!(m.exports.contains_key("one"));

                let exported_fn = match &m.exports["fib"] {
                    core::ExportValue::Function(f) => f,
                    _ => panic!("Unexpected export type"),
                };

                let exported_value_fib7 = match &m.exports["fib7"] {
                    core::ExportValue::Global(g) => g,
                    _ => panic!("Unexpected global export type"),
                };

                assert_eq!(
                    exported_value_fib7.borrow().global_type().clone(),
                    GlobalType::new(ValueType::I32, MutableType::Var)
                );
                assert_eq!(
                    exported_value_fib7.borrow().get_value().clone(),
                    7u32.into()
                );

                let exported_value_zero = match &m.exports["zero"] {
                    core::ExportValue::Global(g) => g,
                    _ => panic!("Unexpected global export type"),
                };

                assert_eq!(
                    exported_value_zero.borrow().global_type().clone(),
                    GlobalType::new(ValueType::I32, MutableType::Const)
                );
                assert_eq!(
                    exported_value_zero.borrow().get_value().clone(),
                    0u32.into()
                );

                let exported_value_one = match &m.exports["one"] {
                    core::ExportValue::Global(g) => g,
                    _ => panic!("Unexpected global export type"),
                };

                assert_eq!(
                    exported_value_one.borrow().global_type().clone(),
                    GlobalType::new(ValueType::I32, MutableType::Const)
                );
                assert_eq!(exported_value_one.borrow().get_value().clone(), 1u32.into());

                assert_eq!(m.memories.len(), 1);
                let memory = m.memories[0].borrow();

                assert_eq!(memory.min_size(), 2);
                assert_eq!(memory.max_size(), None);
                assert_eq!(memory.current_size(), 2);
                assert_eq!(memory[0], 't' as u8);
                assert_eq!(memory[1], 'e' as u8);
                assert_eq!(memory[2], 's' as u8);
                assert_eq!(memory[3], 't' as u8);

                let mut buf: [u8; 4] = [0; 4];
                memory.get_data(0, &mut buf);
                assert_eq!(buf, ['t' as u8, 'e' as u8, 's' as u8, 't' as u8]);

                assert_eq!(memory[65534], 's' as u8);
                assert_eq!(memory[65535], 'p' as u8);
                assert_eq!(memory[65536], 'a' as u8);
                assert_eq!(memory[65537], 'n' as u8);

                memory.get_data(65534, &mut buf);
                assert_eq!(buf, ['s' as u8, 'p' as u8, 'a' as u8, 'n' as u8]);

                assert_eq!(m.tables.len(), 1);
                let table = m.tables[0].borrow();

                assert_eq!(table.min_size(), 2);
                assert_eq!(table.max_size(), None);
                assert_eq!(table.current_size(), 2);
                assert!(table[0].is_some());
                assert!(std::rc::Rc::ptr_eq(
                    &table[0].as_ref().unwrap(),
                    &exported_fn
                ));
                assert!(table[1].is_none());
            }
            Err(e) => {
                assert!(false, format!("Test file failed to load: {}", e));
            }
        }
        Ok(())
    }
}
