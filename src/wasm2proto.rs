use prost::Message;
use std::env;
use std::fs::read;

pub mod libernet_wasm {
    include!(concat!(env!("OUT_DIR"), "/libernet.wasm.rs"));
}

mod helpers;
mod operators;
mod program_module;
mod sections;

use program_module::{from_wasm, render_wasm};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!(
            "Usage: {} <input_wasm_file> <output_proto_file> <output_wasm_file>",
            args[0]
        );
        std::process::exit(1);
    }
    let input_wasm_file = &args[1];
    let output_proto_file = &args[2];
    let output_wasm_file = &args[3];

    let in_bytes = read(input_wasm_file).expect("Failed to read wasm file");
    let program_module = from_wasm(&in_bytes).expect("Failed to parse wasm file");
    let proto_bytes = program_module.encode_to_vec();
    let out_bytes = render_wasm(program_module).expect("Failed to render wasm file");

    std::fs::write(output_proto_file, &proto_bytes).expect("Failed to write proto file");
    std::fs::write(output_wasm_file, &out_bytes).expect("Failed to write wasm file");

    println!(
        "in: {}, out: {}, proto: {}",
        in_bytes.len(),
        out_bytes.len(),
        proto_bytes.len()
    );

    let features = wasmparser::WasmFeatures::default()
        | wasmparser::WasmFeatures::EXCEPTIONS
        | wasmparser::WasmFeatures::LEGACY_EXCEPTIONS
        | wasmparser::WasmFeatures::BULK_MEMORY
        | wasmparser::WasmFeatures::SIGN_EXTENSION
        | wasmparser::WasmFeatures::SATURATING_FLOAT_TO_INT;
    let mut validator = wasmparser::Validator::new_with_features(features);
    let parser = wasmparser::Parser::new(0);
    parser.parse_all(&out_bytes).for_each(|payload| {
        if let Ok(payload) = payload
            && let Err(e) = validator.payload(&payload)
        {
            eprintln!("WASM validation error: {:?}", e);
            std::process::exit(1);
        }
    });
}
