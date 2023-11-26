fn main () -> Result<(), Box<dyn std::error::Error>> {
    // Voting example server/client
    tonic_build::compile_protos("src/voting/voting.proto")?;
    tonic_build::compile_protos("src/proto/common_rpc.proto")?;
    Ok(())

    // Voting example client
}
