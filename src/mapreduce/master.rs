/* General Imports ****************************************************************************************************/
use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
mod common;

/* Tonic RPC generated stubs ******************************************************************************************/
use common_rpc::register_server::{Register, RegisterServer};    // Master is the server in the register service.
use common_rpc::runner_client::RunnerClient;                    // Master is the client in the runner service.
                                                                // Client can be used without direct implementation.
use common_rpc::{RegisterArgs, RegisterReply, RunArgs, EmptyMessage};         // Import message types.

pub mod common_rpc {
    tonic::include_proto!("common_rpc");                        // This string must match the proto package name.
}

/* Basic Definitions **************************************************************************************************/
const IDLE_WORKER_BUFFER: usize = 100;

#[derive(Debug)]
pub struct Master {
    // Task
    // task: Arc<Mutex<common::Task>>,

    // Network
    address: std::net::SocketAddr,
    // TODO: need check
    // rpc_server: rpc::Server,
    // listener: TcpListener,

    // Workers handling
    // workers_mutex: Mutex<HashMap<i32, Arc<Mutex<RemoteWorker>>>>,
    total_workers: i32, // Used to generate unique ids for new workers

    // idle_worker_chan: (Sender<Arc<Mutex<RemoteWorker>>>, Receiver<Arc<Mutex<RemoteWorker>>>),
    // failed_worker_chan: (Sender<Arc<Mutex<RemoteWorker>>>, Receiver<Arc<Mutex<RemoteWorker>>>),

    // Fault Tolerance
    retry_operation_chan: (Sender<common::Operation>, Receiver<common::Operation>),
}

/* Master RPCs ********************************************************************************************************/
#[tonic::async_trait]
impl Register for Master {
    async fn register(
        &self,
        request: Request<RegisterArgs>,                         // Requests should have RegisterArgs type.
    ) -> Result<Response<RegisterReply>, Status> {              // Results should have RegisterReply type.
        
        let args = request.into_inner();                        // Unpack request since its fields are private.

        println!("Registering worker {} with hostname {}.", self.total_workers, args.worker_hostname); 
        
        //@TODO: Finish implementation of register function.
        Ok(Response::new(common_rpc::RegisterReply {
            worker_id: {self.total_workers},
            reduce_jobs: {1},
        }))
    }
}

/* Master Implementation **********************************************************************************************/
impl Master {
    // Construct a new Master struct
    fn new_master(address: std::net::SocketAddr) -> Master {
        let master = Master {
            address,
            total_workers: 0,
            // idle_worker_chan: mpsc::channel(),
            // failed_worker_chan: mpsc::channel(),
            retry_operation_chan: mpsc::channel(1),
        };
  
        master
    }
  
    // // accept_multiple_connections will handle the connections from multiple workers.
    // fn accept_multiple_connections(&self, idle_worker_receiver: mpsc::Receiver<Arc<Mutex<RemoteWorker>>>) {
    //     log::info!("Accepting connections on {}", self.listener.local_addr().unwrap());
  
    //     for stream in self.listener.incoming() {
    //         match stream {
    //             Ok(conn) => {
    //                 let task_clone = Arc::clone(&self.task);
    //                 let workers_mutex_clone = Arc::clone(&self.workers_mutex);
    //                 let idle_worker_sender_clone = self.idle_worker_chan.clone();
    //                 let failed_worker_sender_clone = self.failed_worker_chan.clone();
  
    //                 thread::spawn(move || {
    //                     self.handle_connection(conn, task_clone, workers_mutex_clone, idle_worker_sender_clone, failed_worker_sender_clone);
    //                 });
    //             }
    //             Err(e) => log::error!("Failed to accept connection. Error: {}", e),
    //         }
    //     }
  
    //     log::info!("Stopped accepting connections.");
    // }
  
    // handle_failing_workers will handle workers that fail during an operation.
    // fn handle_failing_workers(&self, failed_worker_receiver: mpsc::Receiver<Arc<Mutex<RemoteWorker>>>) {
    //     for failed_worker in failed_worker_receiver {
    //         let mut workers = self.workers_mutex.lock().unwrap();
    //         let id = failed_worker.lock().unwrap().id;
    //         workers.remove(&id);
    //         log::info!("Removing worker {} from master list.", id);
    //     }
    // }
  
    // // handle_retry_operations will handle retry operations.
    // fn handle_retry_operations(&self, retry_operation_receiver: mpsc::Receiver<Operation>) {
    //     // Implement your logic for handling retry operations here
    //     // Use retry_operation_receiver as needed
    //     // Example:
    //     // for operation in retry_operation_receiver.iter() {
    //     //     // Handle the retry operation
    //     // }
    // }
  
    // // Handle a single connection until it's done, then close it.
    // fn handle_connection(
    //     &self,
    //     conn: TcpStream,
    //     task: Arc<Task>,
    //     workers_mutex: Arc<Mutex<HashMap<i32, Arc<Mutex<RemoteWorker>>>>>,
    //     idle_worker_sender: mpsc::Sender<Arc<Mutex<RemoteWorker>>>,
    //     failed_worker_sender: mpsc::Sender<Arc<Mutex<RemoteWorker>>>,
    // ) {
    //     // Implement your logic for handling a connection here
    //     // Use task, workers_mutex, idle_worker_sender, failed_worker_sender as needed
    //     // Example:
    //     // let remote_worker = RemoteWorker::new(); // You may need to implement a new method for RemoteWorker
    //     // self.idle_worker_chan.send(remote_worker).unwrap();
    // }
}

/* Master Main Function ***********************************************************************************************/
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "[::1]:8080".parse().unwrap();
    let master = Master::new_master(address);

    Server::builder().add_service(RegisterServer::new(master))
    .serve(address)
    .await?;

    Ok(())
}