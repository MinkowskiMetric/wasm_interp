use std::env;

mod core;
mod parser;
mod reader;

use std::io::{self, BufReader};
use std::fs::File;

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
            Err(_) => println!("Failed to read module from {}", &args[1]),
            Ok(module) => {
                println!("Module {:?}", module);
                println!("Done");
            }
        }
    }
}
