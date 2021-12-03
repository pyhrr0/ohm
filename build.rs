fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(&["proto/ohm/v1/ohm_api.proto"], &["proto/"])?;
    Ok(())
}
