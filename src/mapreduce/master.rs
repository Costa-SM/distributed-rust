/* General Imports ****************************************************************************************************/
use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};

use crate::common;
use crate::master_remoteworker;
use crate::word_count;

/* Tonic RPC generated stubs ******************************************************************************************/
use common_rpc::register_server::{Register, RegisterServer};    // Master is the server in the register service.
use common_rpc::runner_client::RunnerClient;                    // Master is the client in the runner service.
                                                                // Client can be used without direct implementation.
use common_rpc::{RegisterArgs, RegisterReply, RunArgs, EmptyMessage};         // Import message types.

pub mod common_rpc {
    tonic::include_proto!("common_rpc");                        // This string must match the proto package name.
}

#[derive(Debug)]
pub struct Master {
    // Task
    pub task: tokio::sync::Mutex<common::Task>,

    // Network
    pub address: std::net::SocketAddr,

    // Sender Channels
    pub idle_tx: Sender<master_remoteworker::RemoteWorker>,
    pub failed_tx: Sender<master_remoteworker::RemoteWorker>,
    pub retry_operation_tx: Sender<common::Operation>,

    // Workers handling
    pub workers: Mutex<Vec<master_remoteworker::RemoteWorker>>,
    pub total_workers: Mutex<usize>, // Used to generate unique ids for new workers
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
            let send_result = idle_tx.send(new_worker_clone).await;
            match send_result {
                Ok(_) => println!("Successfully sent worker to idle channel."),             
                Err(_) => {
                    println!("Failed to send new worker to idle channel.");
                    std::process::exit(1);
                }
            }
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
    pub fn new_master(address: std::net::SocketAddr, 
                  idle_tx: Sender<master_remoteworker::RemoteWorker>,
                  failed_tx: Sender<master_remoteworker::RemoteWorker>,
                  retry_operation_tx: Sender<common::Operation>,) 
                  -> Master {
        let master = Master {
            // Task
            task: tokio::sync::Mutex::new(common::Task::new_task(word_count::map_func, word_count::shuffle_func, word_count::reduce_func)),
            
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

    // Run the master
    pub async fn start_rpc(self, address: std::net::SocketAddr) {
   
    //    // Listen to idle channel
    //    tokio::spawn(async move {
    //        while let Some(msg) = idle_worker_rx.recv().await {
    //            println!("New worker registered!");
    //            println!("ID: {}, Hostname: {}", msg.id, msg.hostname);
   
    //            std::thread::sleep(std::time::Duration::from_secs(1));
   
    //            let mut run_client = RunnerClient::connect("http://[::1]:8081").await.unwrap();
    //            let request = tonic::Request::new(RunArgs{
    //                id: {0},
    //                file_path: {"./src/data/alice.txt".to_string()},
    //            });
   
    //            run_client.run_map(request).await;
   
    //            let request = tonic::Request::new(RunArgs{
    //                id: {0},
    //                file_path: {"./src/data/alice.txt".to_string()},
    //            });
    //            run_client.run_reduce(request).await;
    //        }
    //    });
   
    //    // Listen to failed worker channel
    //    tokio::spawn(async move {
    //        while let Some(msg) = fail_worker_rx.recv().await {
               
    //        }
    //    });
   
    //    let master = Master::new_master(address, idle_worker_tx, fail_worker_tx, retry_operation_tx);
   
       Server::builder().add_service(RegisterServer::new(self))
       .serve(address)
       .await;
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
    // fn handle_failing_workers(&self) {
    //     let receiver = Arc::clone(&self.failed_worker_chan.1); // Wrap in Arc<Mutex<...>> for cloning
    //     let workers_mutex = self.workers.clone();

    //     let sync_code = task::spawn(async move {
    //         while let Some(failed_worker) = receiver.recv().await {
    //             let mut workers = workers_mutex.lock().unwrap();
    //             workers.retain(|worker| worker.id != failed_worker.id);
    //             println!("Removing worker {} from the master list.", failed_worker.id);
    //         }
    //     });

    //     // You can choose to await the task if you want to wait for it to finish
    //     tokio::runtime::Handle::current().block_on(sync_code).unwrap();
    // }
}