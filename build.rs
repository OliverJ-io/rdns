use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    println!("cargo:warning=OUT_DIR = {}", out_dir);
    
    tonic_build::compile_protos("proto/control.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));
}