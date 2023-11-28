/* General Imports ****************************************************************************************************/
use tonic::{transport::Server, Request, Response, Status};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};

use crate::common;
use crate::data;
use crate::word_count;

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
        let read_string = std::fs::read_to_string(args.file_path.clone());
        match read_string {
            Ok(read_string) => {
                let map_result = (self.task.map)(&read_string.as_bytes().to_vec());

                // Store the result locally.
                let local_store = data::store_local(&self.task, args.id, &map_result);

                match local_store {
                    Ok(_) => {
                        println!("Finished map ID: {}, Path: {}", args.id, args.file_path.clone());
                        
                        Ok(Response::new(common_rpc::EmptyMessage {
                        }))
                    }

                    Err(error) => {
                        println!("Map result storage failure with error: {}", error);
                        println!("Dumping map result...");
                        
                        for pair in map_result {
                            println!("{}: {}", pair.key, pair.value);
                        }
                        
                        std::process::exit(1);
                    }
                }
            }

            Err(error) => {
                println!("Error reading input file: {}", error);
                std::process::exit(1);
            }
        }
    }

    async fn run_reduce(
        &self,
        request: Request<RunArgs>,
    ) -> Result<Response<EmptyMessage>, Status> {
        let args = request.into_inner();
        println!("Running reduce ID: {}, Path: {}", args.id, args.file_path.clone());

        // Try to load map result from local storage.
        let mut file_opening = data::load_local(args.id);
        match file_opening {
            Ok(ref mut map_result) => {
                let reduce_result = (self.task.reduce)(map_result);

                for pair in reduce_result {
                    println!("{}: {}", pair.key, pair.value);
                }
            }

            Err(error) => {
                println!("Error reading map file: {}", error);
                std::process::exit(1);
            }
        }
        
        
        


        Ok(Response::new(common_rpc::EmptyMessage {
        }))
    }

    async fn done(
        &self,
        request: Request<EmptyMessage>,
    ) -> Result<Response<EmptyMessage>, Status> {
        
        println!("Worker is done.");

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

    async fn register(&mut self) {
        println!("\nRegistering with master...");
        let mut reg_client = RegisterClient::connect(self.master_hostname.clone()).await;

        match reg_client {
            Ok(ref mut reg_client) => {
                let request = tonic::Request::new(RegisterArgs {
                    worker_hostname: self.hostname.clone(),
                });
        
                let response = reg_client.register(request).await;
                match response {
                    Ok(response) => {
                        let args = response.into_inner();
                        println!("Registered with ID {} and ReduceJobs {}.", args.worker_id, args.reduce_jobs);
        
                        self.id = args.worker_id;
                        self.task.num_reduce_jobs = args.reduce_jobs;
                    }
        
                    Err(_) => {
                        panic!("Registration with master failed !")
                    }
                }
            }

            Err(_) => {
                panic!("Connection with master has been refused !\nAddress used was: {}", self.master_hostname);
            }
        }
    }
}

/* Worker Main Function ***********************************************************************************************/
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut worker = Worker::new_worker(0, "HostA".to_string(), "http://[::1]:8080".to_string());
    
    // Worker must wait for registration to complete before doing anything else.
    worker.register().await;

    // Since registration is complete, start up the RPC server, and perform other tasks.
    let address = "[::1]:8081".parse().unwrap();
    let worker = Worker::new_worker(0, "HostA".to_string(), "MasterA".to_string());
    
    Server::builder().add_service(RunnerServer::new(worker))
    .serve(address)
    .await;

    Ok(())
}