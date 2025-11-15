pub mod byte_stream;
pub mod header;
pub mod runtime;
pub mod compiler;
pub mod globals;
pub mod encrypter;

use std::{env, time};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        let mod_path = &args[1];
        println!("Compiling .luna mod: \"{}\" with version {} ", mod_path, globals::Globals::LUNA_VERSION);
        let header_data = header::Header::read_json(mod_path);

        let source_code = runtime::Runtime::get_source_code(mod_path);

        let compiled_data = compiler::LunaCompile::compile(&header_data, source_code);

        let final_path = format!("{}-{}-{}.luna", header_data.id, header_data.version, time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs());
        
        std::fs::write(&final_path, compiled_data.get_buffer()).unwrap();
        
        println!("Compiled successfully! Output file: {}", final_path);
    } else {
        println!("Usage: LunaCompile <mod_path>");
    }
}
