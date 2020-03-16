mod core;
mod parser;
mod reader;

use anyhow::{Context, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("wasm [mod_name]");
    } else {
        core::load_module_from_path(&args[1], core::EmptyResolver::instance())
            .with_context(|| format!("Failed to read module from {}", &args[1]))?;
    }

    Ok(())
}
