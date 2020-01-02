mod core;
mod parser;
mod reader;

use std::{
    env,
    fs::File,
    io::{self, BufReader},
};

use reader::TypeReader;

fn load_module_from_path(file: &str) -> io::Result<core::Module> {
    let file = File::open(file)?;
    let mut file = BufReader::new(file);

    let module = core::RawModule::read(&mut file)?;
    let module = core::Module::resolve_raw_module(module, core::EmptyResolver::instance())?;

    Ok(module)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("wasm [mod_name]");
    } else {
        match load_module_from_path(&args[1]) {
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

    #[test]
    fn test_load_module() -> std::result::Result<(), String> {
        if let Ok(m) = load_module_from_path("../test_app/test.wasm") {
            assert_eq!(m.exports.len(), 1);
            assert!(m.exports.contains_key("fib"));

            let exported_fn = match &m.exports["fib"] {
                core::ExportValue::Function(f) => f,
                _ => panic!("Unexpected export type"),
            };

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
        } else {
            assert!(false, "Test file failed to load");
        }
        Ok(())
    }
}
