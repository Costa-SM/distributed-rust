use master_remoteworker::RemoteWorker;
use tokio::io::AsyncBufRead;
/* General Imports ****************************************************************************************************/
use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc::{Sender, channel};
use core::panic;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::collections::hash_map::HashMap;
use clap::{App, Arg};

mod common;
mod word_count;
mod master_remoteworker;
mod data;

/* Tonic RPC generated stubs ******************************************************************************************/
use common_rpc::register_server::{Register, RegisterServer};    // Master is the server in the register service.
use common_rpc::runner_client::RunnerClient;                    // Master is the client in the runner service.
                                                                // Client can be used without direct implementation.
use common_rpc::{RegisterArgs, RegisterReply, RunArgs, EmptyMessage};

use crate::common::Operation;                                   // Import message types.

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
    completed_operations: Arc<Mutex<usize>>,

    // Network
    address: std::net::SocketAddr,

    // Sender Channels
    idle_tx: Sender<master_remoteworker::RemoteWorker>,
    failed_tx: Sender<master_remoteworker::RemoteWorker>,
    retry_operation_tx: Sender<common::Operation>,

    // Workers handling
    workers: Arc<Mutex<HashMap<usize, master_remoteworker::RemoteWorker>>>,
    total_workers: Arc<Mutex<usize>>, // Used to generate unique ids for new workers
}

/* Auxiliary Functions ************************************************************************************************/
async fn call_map(
                id: usize, 
                args: RunArgs, 
                worker_mutex: Arc<Mutex<HashMap<usize, master_remoteworker::RemoteWorker>>>,
                idle_tx: Sender<master_remoteworker::RemoteWorker>, 
                completed_mutex: Arc<Mutex<usize>>,
                failed_tx: Sender<master_remoteworker::RemoteWorker>,
                retry_tx: Sender<common::Operation>
            ){
    // Save the operation.
    let operation = Operation{proc: {"map".to_string()}, 
                                file_path: {args.file_path.clone()}, 
                                id: {id as i32}};
    
    // Get the mutex for the workers.
    let workers = worker_mutex.lock().unwrap();
    let current_worker = (*workers).get(&id).unwrap().clone();

    // Create request for running operation.
    let mut run_client = RunnerClient::connect(current_worker.hostname.clone()).await.unwrap();
    let request = tonic::Request::new(args);
    
    // Send the request and verify whether or not operation has failed.
    let request_res = run_client.run_map(request).await;
    match request_res {
        Ok(_) => {
            // Notify via channel that the worker node is now idle.
            tokio::spawn(async move {
                idle_tx.send(current_worker.clone()).await;
            });

            let mut completed_operations = completed_mutex.lock().unwrap();
            *completed_operations += 1;
        }
        Err(error) => {
            println!("Map function on {} failed with error: {}", id, error);
            
            // Notify via channel that the operation failed.
            tokio::spawn(async move {
                failed_tx.send(current_worker.clone()).await;
                retry_tx.send(operation).await;
            });
        }
    }
}

async fn call_reduce(
                    id: usize, 
                    args: RunArgs, 
                    worker_mutex: Arc<Mutex<HashMap<usize, master_remoteworker::RemoteWorker>>>,
                    idle_tx: Sender<master_remoteworker::RemoteWorker>, 
                    completed_mutex: Arc<Mutex<usize>>,
                    failed_tx: Sender<master_remoteworker::RemoteWorker>,
                    retry_tx: Sender<common::Operation>
                ){
    // Save the operation.
    let operation = Operation{proc: {"reduce".to_string()}, 
                                file_path: {args.file_path.clone()}, 
                                id: {id as i32}};
    
    // Get the mutex for the workers.
    let workers = worker_mutex.lock().unwrap();
    let current_worker = (*workers).get(&id).unwrap().clone();

    // Create request for running operation.
    let mut run_client = RunnerClient::connect(current_worker.hostname.clone()).await.unwrap();
    let request = tonic::Request::new(args);
    
    // Send the request and verify whether or not operation has failed.
    let request_res = run_client.run_reduce(request).await;
    match request_res {
        Ok(_) => {
            // Notify via channel that the worker node is now idle.
            tokio::spawn(async move {
                idle_tx.send(current_worker.clone()).await;
            });

            let mut completed_operations = completed_mutex.lock().unwrap();
            *completed_operations += 1;
        }
        Err(error) => {
            println!("Reduce function on {} failed with error: {}", id, error);
            
            // Notify via channel that the operation failed.
            tokio::spawn(async move {
                failed_tx.send(current_worker.clone()).await;
                retry_tx.send(operation).await;
            });
        }
    }
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
        
        (*workers).insert(*worker_count, new_worker);
        *worker_count += 1;

        // Signal the idleWorker channel about the new worker.
        let idle_tx = self.idle_tx.clone();
        
        tokio::spawn(async move {
            idle_tx.send(new_worker_clone).await;
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
            completed_operations: Arc::new(Mutex::new(0)),

            // Network
            address,

            // Sender Channels
            idle_tx,
            failed_tx,
            retry_operation_tx,
        
            // Workers handling
            workers: Arc::new(Mutex::new(HashMap::new())),
            total_workers: Arc::new(Mutex::new(0)),
        };
  
        master
    }

    fn schedule_maps(&self, file_path: String) {
        println!("Scheduling map operations");

    }
}

/* Master Main Function ***********************************************************************************************/
#[tokio::main]
async fn main() {
    /* PARSE VARIABLES FROM COMMAND LINE ******************************************************************************/
    let matches = App::new("MapReduce")
        .arg(
            Arg::with_name("reducejobs")
                .short("r")
                .long("reducejobs")
                .value_name("NUM")
                .help("Number of reduce jobs that should be run")
                .takes_value(true)
                .default_value("5"),
        )
        // Input data settings
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("File to use as input")
                .takes_value(true)
                .default_value("files/pg1342.txt"),
        )
        .arg(
            Arg::with_name("chunksize")
                .short("c")
                .long("chunksize")
                .value_name("SIZE")
                .help("Size of data chunks that should be passed to map jobs (in bytes)")
                .takes_value(true)
                .default_value("102400"), // Corresponds to 100*1024
        )
        // Network settings
        .arg(
            Arg::with_name("addr")
                .short("a")
                .long("addr")
                .value_name("ADDRESS")
                .help("IP address to listen on")
                .takes_value(true)
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("TCP port to listen on")
                .takes_value(true)
                .default_value("5000"),
        )
        .arg(
            Arg::with_name("master")
                .short("m")
                .long("master")
                .value_name("MASTER")
                .help("Master address")
                .takes_value(true)
                .default_value("127.0.0.1:5000"),
        )
        // Induced failure on Worker
        .arg(
            Arg::with_name("fail")
                .short("n")
                .long("fail")
                .value_name("NUM")
                .help("Number of operations to run before failure")
                .takes_value(true)
                .default_value("0"),
        )
        .get_matches();

    /* AUXILIARY VARIABLES ********************************************************************************************/
    let reduce_jobs = matches
        .value_of("reducejobs")
        .unwrap_or_else(|| "5")
        .parse::<i32>()
        .unwrap(); // Parse to i32

    let file = matches.value_of("file").unwrap_or_else(|| "files/pg1342.txt");
    let chunk_size = matches
        .value_of("chunksize")
        .unwrap_or_else(|| "102400")
        .parse::<usize>()
        .unwrap(); // Parse to usize

    let addr = matches.value_of("addr").unwrap_or_else(|| "127.0.0.1");
    let port = matches
        .value_of("port")
        .unwrap_or_else(|| "5000")
        .parse::<u16>()
        .unwrap(); // Parse to u16


    /* MASTER IMPLEMENTATION ******************************************************************************************/
    // Channels for idle and failed workers, as well as fault tolerance.
    let (idle_worker_tx, 
         mut idle_worker_rx) = channel::<master_remoteworker::RemoteWorker>(IDLE_WORKER_BUFFER);
    let (fail_worker_tx, 
         mut fail_worker_rx) = channel::<master_remoteworker::RemoteWorker>(FAILED_WORKER_BUFFER);
    let (retry_operation_tx, 
         mut retry_operation_rx) = channel(RETRY_OPERATION_BUFFER);

    // Number of files.
    let num_files = data::split_data(file, chunk_size);

    // Fan in and fan out channels.
    let mut fan_in = data::fan_in_data(num_files as i32);
    let (fan_out, mut wait_for_it) = data::fan_out_data();
    
    // Get the correct IP address
    let ip_addr: std::net::Ipv4Addr = match std::net::Ipv4Addr::from_str(addr) {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("Error parsing IP address: {}", e);
            return;
        }
    };
    let address: std::net::SocketAddr = std::net::SocketAddr::new(std::net::IpAddr::V4(ip_addr), port);
    
    // Create the master node.
    let master = Master::new_master(address, idle_worker_tx, fail_worker_tx, retry_operation_tx);

    // Update master with fan in channel.
    let master_task_mutex = master.task.clone();
    let mut master_task = master_task_mutex.lock().unwrap();

    // Listen to failed worker channel -- Handle Failing Workers
    let workers_clone = master.workers.clone();
    tokio::spawn(async move {
        while let Some(msg) = fail_worker_rx.recv().await {
            println!("Worker {} has failed.", msg.id);
            let mut workers_lock = workers_clone.lock().unwrap();

            println!("Removing worker {} from worker list.", msg.id);
            (*workers_lock).remove(&msg.id);
        }
    });

    // Run map.
    let mut counter = 0;
    while let Some(msg) = fan_in.recv().await {
        let operation = Operation {proc: {"map".to_string()}, file_path: {file.to_string()}, id: {counter}};
        counter += 1;

        let worker = idle_worker_rx.recv().await.unwrap();
        let args = RunArgs { id: (counter), file_path: (file.to_string()) };

        tokio::spawn(async move {
            call_map(counter as usize, args, 
                        master.workers.clone(), 
                        master.idle_tx.clone(), 
                        master.completed_operations.clone(), 
                        master.failed_tx.clone(), 
                        master.retry_operation_tx.clone());
        });
    }

    // Run merge.


    // Run reduce.

    // Start the RPC Server.
    tokio::spawn(async move {
        let server_result = Server::builder().add_service(RegisterServer::new(master))
                             .serve(address)
                             .await;
        match server_result {
            Ok(_) => {}
            Err(error) => {
                println!("Master RPC server startup failed with error: {}", error);
                std::process::exit(1);
            }
        }
    });
}