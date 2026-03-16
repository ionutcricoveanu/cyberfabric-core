fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the proto files
    tonic_build::compile_protos("../proto/ml_service.proto")?;
    Ok(())
}
