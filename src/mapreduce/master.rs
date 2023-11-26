use std::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};
use registerWorker::{RegisterRequest, RegisterResponse, master::{Register, MasterServer}};

pub mod registerWorker {
    tonic::include_proto!("registerWorker");
}
  
#[derive(Debug, Default)]
pub struct Master {
    // Task performed by master
    task: Task,

    // Networking
    address: String,
    rpcServer: 

    // Worker handling
    workerMutex: Mutex,
    totalWorkers: i32,


    // Fault tolerance
}


#[tonic::async_trait]
impl RegisterWorker for Master {
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<RegisterResponse>, Status> {
        let r = request.into_inner();
        match r.
    }
} 