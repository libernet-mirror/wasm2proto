fn main() {
    prost_build::compile_protos(
        &["proto/libernet-program.proto"],
        &["proto"],
    ).unwrap();
}