use eframe::WindowBuilderHook;
/* General Imports ****************************************************************************************************/
use tonic::{transport::Server, Request, Response, Status};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
mod common;
mod word_count;
mod data;

/* Tonic RPC generated stubs ******************************************************************************************/
use common_rpc::register_client::RegisterClient;              // Worker is the client in the register service.
                                                                // Client can be used without direct implementation.
use common_rpc::runner_server::{Runner, RunnerServer};          // Worker is the server in the runner service.
use common_rpc::{RegisterArgs, RegisterReply, RunArgs, EmptyMessage};   // Import message types.

pub mod common_rpc {
    tonic::include_proto!("common_rpc");                        // This string must match the proto name.
}

/* Basic Definitions **************************************************************************************************/
#[derive(Debug)]
pub struct Worker {
    id: i32,

    // Network
    hostname: String,
    master_hostname: String,

    // Operation
    task: common::Task,
    done: bool,
}

/* Worker RPCs ********************************************************************************************************/
#[tonic::async_trait]
impl Runner for Worker {
    async fn run_map(
        &self,
        request: Request<RunArgs>,
    ) -> Result<Response<EmptyMessage>, Status> {
        let args = request.into_inner();
        println!("Running map ID: {}, Path: {}", args.id, args.file_path.clone());

        // Open, read file and perform map.
        let read_string = std::fs::read_to_string(args.file_path.clone()).unwrap();
        let map_result = (self.task.map)(read_string.as_bytes());

        // Store the result locally.
        data::store_local(&self.task, self.id, map_result.clone());

        for pair in map_result {
            println!("{}: {}", pair.key, pair.value);
        }

        println!("Finished map ID: {}, Path: {}", args.id, args.file_path.clone());
        Ok(Response::new(common_rpc::EmptyMessage {
        }))
    }

    async fn run_reduce(
        &self,
        request: Request<RunArgs>,
    ) -> Result<Response<EmptyMessage>, Status> {

        //@TODO: Finish implementation.
        Ok(Response::new(common_rpc::EmptyMessage {
        }))
    }

    async fn done(
        &self,
        request: Request<EmptyMessage>,
    ) -> Result<Response<EmptyMessage>, Status> {
        //@TODO: Finish implementation.
        Ok(Response::new(common_rpc::EmptyMessage {
        }))
    }
}

/* Worker Implementation **********************************************************************************************/
impl Worker {
    fn new_worker(id: i32, hostname: String, master_hostname: String) -> Worker {
        let worker = Worker {
            id,
            hostname,
            master_hostname,
            task: common::Task::new_task(word_count::map_func, word_count::shuffle_func, word_count::reduce_func),
            done: false,
        };
    
        return worker;
    }
}

/* Worker Main Function ***********************************************************************************************/
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reg_client = RegisterClient::connect("http://[::1]:8080").await?;
    let worker = Worker::new_worker(0, "HostA".to_string(), "MasterA".to_string());

    println!("\nRegistering with master...");
    let request = tonic::Request::new(RegisterArgs {
        worker_hostname: worker.hostname,
    });
    let response = reg_client.register(request).await?.into_inner();
    println!("Registered with ID {} and ReduceJobs {}.", response.worker_id, response.reduce_jobs);

    let address = "[::1]:8081".parse().unwrap();
    let worker = Worker::new_worker(0, "HostA".to_string(), "MasterA".to_string());
    
    Server::builder().add_service(RunnerServer::new(worker))
    .serve(address)
    .await;

    Ok(())
}