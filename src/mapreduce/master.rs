/* General Imports ****************************************************************************************************/
mod common;
mod word_count;
mod master_remoteworker;

use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};

/* Tonic RPC generated stubs ******************************************************************************************/
use common_rpc::register_server::{Register, RegisterServer};    // Master is the server in the register service.
use common_rpc::runner_client::RunnerClient;                    // Master is the client in the runner service.
                                                                // Client can be used without direct implementation.
use common_rpc::{RegisterArgs, RegisterReply, RunArgs};         // Import message types.

pub mod common_rpc {
    tonic::include_proto!("common_rpc");                        // This string must match the proto package name.
}

/* Basic Definitions **************************************************************************************************/
const IDLE_WORKER_BUFFER: usize = 100;
const FAILED_WORKER_BUFFER: usize = 100;
const RETRY_OPERATION_BUFFER: usize = 100;

#[derive(Debug)]
pub struct Master {
    // Task
    task: Arc<Mutex<common::Task>>,

    // Network
    address: std::net::SocketAddr,

    // Sender Channels
    idle_tx: Sender<master_remoteworker::RemoteWorker>,
    failed_tx: Sender<master_remoteworker::RemoteWorker>,
    retry_operation_tx: Sender<common::Operation>,

    // Workers handling
    workers: Mutex<Vec<master_remoteworker::RemoteWorker>>,
    total_workers: Mutex<usize>, // Used to generate unique ids for new workers
}

/* Master RPCs ********************************************************************************************************/
#[tonic::async_trait]
impl Register for Master {
    async fn register(
        &self,
        request: Request<RegisterArgs>,                         // Requests should have RegisterArgs type.
    ) -> Result<Response<RegisterReply>, Status> {              // Results should have RegisterReply type.
        let args = request.into_inner();                        // Unpack request since its fields are private.

        // Get the mutex for the workers.
        let mut workers = self.workers.lock().unwrap();
        let mut worker_count = self.total_workers.lock().unwrap();

        println!("Registering worker {} with hostname {}.", *worker_count, args.worker_hostname); 

        // Create the worker and push it into the worker list. Also increase the count.
        let new_worker = master_remoteworker::RemoteWorker::new_worker(*worker_count, 
                                                                                     args.worker_hostname);
        let new_worker_clone = new_worker.clone();
        (*workers).push(new_worker);
        *worker_count += 1;

        // Signal the idleWorker channel about the new worker.
        let idle_tx = self.idle_tx.clone();
        
        tokio::spawn(async move {
            idle_tx.send(new_worker_clone).await.expect("Failed to send new worker to idle channel.");
        });

        // Respond to caller with worker number and reduce jobs.
        Ok(Response::new(common_rpc::RegisterReply {
            worker_id: {*worker_count - 1} as i32,
            reduce_jobs: {1},
        }))

        // Mutex is released automatically once the variable goes out of scope
    }
}

/* Master Implementation **********************************************************************************************/
impl Master {
    // Construct a new Master struct
    fn new_master(address: std::net::SocketAddr, 
                  idle_tx: Sender<master_remoteworker::RemoteWorker>,
                  failed_tx: Sender<master_remoteworker::RemoteWorker>,
                  retry_operation_tx: Sender<common::Operation>,) 
                  -> Master {
        let master = Master {
            // Task
            task: Arc::new(Mutex::new(common::Task::new_task(word_count::map_func, word_count::shuffle_func, word_count::reduce_func))),
            
            // Network
            address,

            // Sender Channels
            idle_tx,
            failed_tx,
            retry_operation_tx,
        
            // Workers handling
            workers: Mutex::new(Vec::new()),
            total_workers: Mutex::new(0),
        };
  
        master
    }
}

/* Master Main Function ***********************************************************************************************/
#[tokio::main]
async fn main() {
    // Channels for idle and failed workers, as well as fault tolerance.
    let (idle_worker_tx, 
         mut idle_worker_rx) = channel::<master_remoteworker::RemoteWorker>(IDLE_WORKER_BUFFER);
    let (fail_worker_tx, 
         mut fail_worker_rx) = channel::<master_remoteworker::RemoteWorker>(FAILED_WORKER_BUFFER);
    let (retry_operation_tx, _) = channel(RETRY_OPERATION_BUFFER);

    // Listen to idle channel
    tokio::spawn(async move {
        while let Some(msg) = idle_worker_rx.recv().await {
            println!("New worker registered!");
            println!("ID: {}, Hostname: {}", msg.id, msg.hostname);

            std::thread::sleep(std::time::Duration::from_secs(1));

            let mut run_client = RunnerClient::connect("http://[::1]:8081").await.unwrap();
            let request = tonic::Request::new(RunArgs{
                id: {0},
                file_path: {"./src/data/alice.txt".to_string()},
            });

            run_client.run_map(request).await.expect("Failed to run map.");

            let request = tonic::Request::new(RunArgs{
                id: {0},
                file_path: {"./src/data/alice.txt".to_string()},
            });
            match run_client.run_reduce(request).await {
                Ok(_) => {
                    println!("Reduce completed successfully!");
                }
                Err(_) => {
                    std::process::exit(1);
                }
            
            }
        }
    });

    // Listen to failed worker channel
    tokio::spawn(async move {
        while let Some(_) = fail_worker_rx.recv().await {
            
        }
    });

    let address = "[::1]:8080".parse().unwrap();
    let master = Master::new_master(address, idle_worker_tx, fail_worker_tx, retry_operation_tx);

    Server::builder().add_service(RegisterServer::new(master))
    .serve(address)
    .await.expect("Failed to start RPC server.");
}