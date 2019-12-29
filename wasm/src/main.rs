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

    core::Module::read(&mut file)
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
