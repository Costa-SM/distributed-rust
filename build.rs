fn main () -> Result<(), Box<dyn std::error::Error>> {
    // Voting example server/client
    tonic_build::compile_protos("src/voting/voting.proto")?;

    // MapReduce RPC
    tonic_build::compile_protos("src/mapreduce/mapreduce.proto")?;
    Ok(())
}
