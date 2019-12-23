use std::env;

mod core;
mod parser;
mod reader;

mod module;

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
