fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("CARGO_FEATURE_GRPC").is_ok() {
        tonic_build::configure()
            .build_client(true)
            .compile_protos(&["proto/blog.proto"], &["proto"])?;

        println!("cargo:rerun-if-changed=proto/blog.proto");
    }
    Ok(())
}
