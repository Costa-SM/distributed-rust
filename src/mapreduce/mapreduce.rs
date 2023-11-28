use std::fs;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::io;

use crate::common::{KeyValue, Task};
use crate::data::{load_local, merge_map_local, remove_contents, store_local, REDUCE_PATH};
// use crate::master::Master;
// use crate::worker::Worker;

// RunSequential will ensure that map and reduce function runs in
// a single-core linearly. The Task is passed from the calling package
// and should contains the definitions for all the required functions
// and parameters.
// Notice that this implementation will store data locally. In the distributed
// version of mapreduce it's common to store the data in the same worker that computed
// it and just pass a reference to reduce jobs so they can go grab it.
pub async fn run_sequential(task: &mut Task) {
    let mut map_counter = 0;
    let mut map_result;

    println!("Running RunSequential...");

    // Create or clear the REDUCE_PATH directory
    let _ = fs::create_dir(REDUCE_PATH);
    let _ = remove_contents(REDUCE_PATH);

    while let Some(v) = task.input_chan.recv().await {
        map_result = (task.map)(&v);
        let _ = store_local(task, map_counter, &map_result);
        map_counter += 1;
    }

    let _ = merge_map_local(task, map_counter);

    for r in 0..task.num_reduce_jobs {
        if let Ok(mut data) = load_local(r) {
            let reduced_data = (task.reduce)(&mut data);
            if let Err(_) = task.output_chan.send(reduced_data.to_vec()).await {
                println!("receiver dropped");
                return;
            }
        }
    }
    // Close the output channel
    drop(task.output_chan);
}

// RunMaster will start a master node on the map reduce operations.
// In the distributed model, a Master should serve multiple workers and distribute
// the operations to be executed in order to complete the task.
// 	- task: the Task object that contains the mapreduce operation.
//  - hostname: the tcp/ip address on which it will listen for connections.
// pub async fn run_master(task: &Task, hostname: String) {
//     let mut err: Option<io::Error> = None;
//     let master: Arc<Mutex<Master>> = Arc::new(Mutex::new(Master::new(&hostname)));

//     println!("Running Master on {}", hostname);

//     // Create a reduce directory to store intermediate reduce files.
//     let _ = fs::create_dir(REDUCE_PATH);
//     let _ = remove_contents(REDUCE_PATH);

//     {
//         let mut master = master.lock().unwrap();
//         master.task = Some(task.clone());
//         master.rpc_server = Some(rpc::Server::new());

//         if let Some(ref mut rpc_server) = master.rpc_server {
//             rpc_server.register(master.clone());

//             // Handle errors here
//             if let Err(e) = err {
//                 println!("Failed to register RPC server. Error: {:?}", e);
//                 return;
//             }
//         }

//         let listener = match TcpListener::bind(&master.address) {
//             Ok(listener) => listener,
//             Err(e) => {
//                 println!("Failed to start TCP server. Error: {:?}", e);
//                 return;
//             }
//         };

//         master.listener = Some(listener.try_clone().unwrap());
//     }

//     let master_clone = Arc::clone(&master);

//     // Start MapReduce Operation
//     tokio::spawn(async move {
//         let master = master_clone.lock().unwrap();
//         master.accept_multiple_connections();
//     });

//     let master_clone = Arc::clone(&master);
//     tokio::spawn(async move {
//         let master = master_clone.lock().unwrap();
//         master.handle_failing_workers();
//     });

//     // Schedule map operations
//     let map_operations = {
//         let master = master.lock().unwrap();
//         master.schedule(task, "Worker.RunMap", task.input_file_path_chan.clone())
//     };

//     // Merge the result of multiple map operations with the same reduceId into a single file
//     merge_map_local(task, map_operations);

//     // Schedule reduce operations
//     let reduce_file_path_chan = fan_reduce_file_path(task.num_reduce_jobs);
//     let reduce_operations = {
//         let master = master.lock().unwrap();
//         master.schedule(task, "Worker.RunReduce", reduce_file_path_chan)
//     };

//     merge_reduce_local(reduce_operations);

//     println!("Closing Remote Workers.");
//     {
//         let master = master.lock().unwrap();
//         for worker in &master.workers {
//             if let Err(e) = worker.call_remote_worker("Worker.Done", &(), &()) {
//                 println!("Failed to close Remote Worker. Error: {:?}", e);
//             }
//         }
//     }

//     println!("Done.");
// }

// // RunWorker will run a instance of a worker. It'll initialize and then try to register with
// // master.
// // Induced failures:
// // -> nOps = number of operations to run before failure (0 = no failure)
// pub fn run_worker(task: &Task, hostname: String, master_hostname: String, n_ops: usize) {
//     let mut err: Option<io::Error> = None;
//     let mut worker = Worker::new();
//     let retry_duration = Duration::from_secs(2);

//     println!("Running Worker on {}", hostname);

//     // Create a reduce directory to store intermediate reduce files.
//     let _ = fs::create_dir(REDUCE_PATH);

//     worker.hostname = hostname;
//     worker.master_hostname = master_hostname;
//     worker.task = Some(task.clone());
//     worker.done = Some(mpsc::channel());

//     // Should induce a failure
//     if n_ops > 0 {
//         worker.task_counter = 0;
//         worker.n_ops = n_ops;
//     }

//     let rpc_server = Arc::new(Mutex::new(rpc::Server::new()));
//     {
//         let mut rpc_server = rpc_server.lock().unwrap();
//         rpc_server.register(worker.clone());

//         // Handle errors here
//         if let Err(e) = err {
//             println!("Failed to register RPC server. Error: {:?}", e);
//             return;
//         }

//         let listener = match TcpListener::bind(&worker.hostname) {
//             Ok(listener) => listener,
//             Err(e) => {
//                 println!("Starting RPC listener failed. Error: {:?}", e);
//                 return;
//             }
//         };

//         worker.listener = Some(listener.try_clone().unwrap());
//         worker.rpc_server = Some(rpc_server.clone());
//     }

//     let worker_clone = worker.clone();
//     let rpc_server_clone = rpc_server.clone();

//     // Ensure the listener is closed when the function exits
//     let _guard = OnDropGuard(|| {
//         if let Some(listener) = worker_clone.listener {
//             let _ = listener.close();
//         }
//     });

//     let retry_duration_clone = retry_duration.clone();
//     tokio::spawn(async move {
//         let mut worker = worker_clone;
//         let mut rpc_server = rpc_server_clone.lock().unwrap();

//         for _ in 0..n_ops {
//             worker.task_counter = 0;

//             // Rest of the loop body
//             rpc_server.register(worker.clone());

//             // Handle errors here
//             if let Err(e) = err {
//                 println!("Failed to register RPC server. Error: {:?}", e);
//                 return;
//             }

//             let listener = match TcpListener::bind(&worker.hostname) {
//                 Ok(listener) => listener,
//                 Err(e) => {
//                     println!("Starting RPC listener failed. Error: {:?}", e);
//                     return;
//                 }
//             };

//             worker.listener = Some(listener.try_clone().unwrap());
//             worker.rpc_server = Some(rpc_server.clone());

//             let _guard = OnDropGuard(|| {
//                 if let Some(listener) = worker.listener {
//                     let _ = listener.close();
//                 }
//             });

//             // Rest of the thread body
//         }
//     });

//     let worker_clone = Arc::new(Mutex::new(worker));
//     let retry_duration_clone = retry_duration.clone();
//     tokio::spawn(async move {
//         let mut worker = worker_clone.lock().unwrap();

//         loop {
//             if let Err(_) = worker.register() {
//                 println!(
//                     "Registration failed. Retrying in {:?} seconds...",
//                     retry_duration_clone
//                 );
//                 tokio::time::sleep(retry_duration_clone).await;
//             } else {
//                 break;
//             }
//         }

//         worker.accept_multiple_connections();
//     });

//     // Wait for the worker to finish
//     let (_, receiver) = worker.done.unwrap();
//     receiver.recv().unwrap();
// }
