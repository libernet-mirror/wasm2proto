fn main() {
    prost_build::compile_protos(&["proto/program.proto"], &["proto"]).unwrap();
}
