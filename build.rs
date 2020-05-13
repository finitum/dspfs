use protoc_grpcio;
use walkdir;
use walkdir::DirEntry;
use std::ffi::OsStr;
use protobuf_codegen::Customize;

fn main() {
    let proto_root = "src/api";

    let dirs = walkdir::WalkDir::new(proto_root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|f: DirEntry| f.path().to_owned())
        .filter(|f| f.extension().unwrap_or(OsStr::new("")) == "proto")
        .collect::<Vec<_>>();

    println!("Proto files: {:?}", dirs);

    println!("cargo:rerun-if-changed={}", proto_root);
    if !dirs.is_empty() {
        println!("Compiling proto files.");
        protoc_grpcio::compile_grpc_protos(
            &dirs,
            &[proto_root],
            &proto_root,
            Some(Customize {
                expose_oneof: Some(true),
                expose_fields: Some(true),
                generate_accessors: Some(true),
                serde_derive: Some(true),
                ..Default::default()
            })
        ).expect("Failed to compile proto files!");
    }
}
