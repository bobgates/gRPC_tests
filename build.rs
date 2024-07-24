fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/imu.proto")?;
    tonic_build::compile_protos("proto/gps.proto")?;
    Ok(())
}