use std::env;
use std::fs::File;
use std::io::Read;

mod module;
mod reader_util;
mod section;
mod type_section_data;
mod import_section_data;
mod expr;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("wasm [mod_name]");
    } else {
        match module::read_from_path(&args[1]) {
            Err(_) => println!("Failed to read module from {}", &args[1]),
            Ok(sections) => {
                for section in sections {
                    println!("Got section {:x?}", section);
                }
                println!("Done");
            }
        }
    }
}
