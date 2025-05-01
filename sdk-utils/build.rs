use prost_wkt_build::*;
use std::{env, path::PathBuf};

fn main() {
    // Use the standard OUT_DIR for generated code
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:warning=Output directory: {}", out.display());

    let descriptor_file = out.join("descriptors.bin");

    let mut prost_build = prost_build::Config::new();
    prost_build
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .extern_path(".google.protobuf.Any", "::prost_wkt_types::Any")
        .extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
        .extern_path(".google.protobuf.Value", "::prost_wkt_types::Value")
        .file_descriptor_set_path(&descriptor_file)
        .compile_protos(&["proto/blueprint.proto"], &["proto/"])
        .unwrap();

    let descriptor_bytes = std::fs::read(descriptor_file).unwrap();
    let descriptor = FileDescriptorSet::decode(&descriptor_bytes[..]).unwrap();

    // Add serde support for well-known types
    prost_wkt_build::add_serde(out.clone(), descriptor);

    // Print the expected file path
    println!(
        "cargo:warning=Expected generated file: {}",
        out.join("blueprint.rs").display()
    );

    // Explicitly write to OUT_DIR in the build script
    println!("cargo:rerun-if-changed=proto/blueprint.proto");
}
