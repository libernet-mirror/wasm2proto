fn main() {
    prost_build::compile_protos(&["proto/libernet.proto"], &["proto"]).unwrap();
}
