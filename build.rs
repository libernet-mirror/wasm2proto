fn main() {
    prost_build::compile_protos(&["proto/libernet_wasm.proto"], &["proto"]).unwrap();
}
