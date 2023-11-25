// common_rpc.rs defined all the parameters used in RPC between
// master and workers
pub struct RegisterArgs {
    worker_hostname: String,
}

pub struct RegisterReply {
    worker_id: i32,
    reduce_jobs: i32,
}

pub struct RunArgs {
    id: i32,
    file_path: String,
}