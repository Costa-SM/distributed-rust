use std::sync::mpsc::{self, Sender, Receiver};

const IDLE_WORKER_BUFFER: usize = 100;

pub struct Operation {
  proc: String,
  id: i32,
  file_path: String,
}

struct Master {
  // Task
  task: Arc<Task>,

  // Network
  address: String,
  // TODO: need check
  // rpc_server: rpc::Server,
  // listener: TcpListener,

  // Workers handling
  workers_mutex: Mutex<HashMap<i32, Arc<Mutex<RemoteWorker>>>>,
  total_workers: i32, // Used to generate unique ids for new workers

  idle_worker_chan: (Sender<Arc<Mutex<RemoteWorker>>>, Receiver<Arc<Mutex<RemoteWorker>>>),
  failed_worker_chan: (Sender<Arc<Mutex<RemoteWorker>>>, Receiver<Arc<Mutex<RemoteWorker>>>),

  // Fault Tolerance
  retry_operation_chan: (Sender<Operation>, Receiver<Operation>),
}

impl Master {
  // Construct a new Master struct
  fn new_master(address: String) -> Master {
      let master = Master {
          address,
          total_workers: 0,
          idle_worker_chan: mpsc::channel(),
          failed_worker_chan: mpsc::channel(),
          retry_operation_chan: mpsc::channel(),
      };

      master
  }

  // accept_multiple_connections will handle the connections from multiple workers.
  fn accept_multiple_connections(&self, idle_worker_receiver: mpsc::Receiver<Arc<Mutex<RemoteWorker>>>) {
      log::info!("Accepting connections on {}", self.listener.local_addr().unwrap());

      for stream in self.listener.incoming() {
          match stream {
              Ok(conn) => {
                  let task_clone = Arc::clone(&self.task);
                  let workers_mutex_clone = Arc::clone(&self.workers_mutex);
                  let idle_worker_sender_clone = self.idle_worker_chan.clone();
                  let failed_worker_sender_clone = self.failed_worker_chan.clone();

                  thread::spawn(move || {
                      self.handle_connection(conn, task_clone, workers_mutex_clone, idle_worker_sender_clone, failed_worker_sender_clone);
                  });
              }
              Err(e) => log::error!("Failed to accept connection. Error: {}", e),
          }
      }

      log::info!("Stopped accepting connections.");
  }

  // handle_failing_workers will handle workers that fail during an operation.
  fn handle_failing_workers(&self, failed_worker_receiver: mpsc::Receiver<Arc<Mutex<RemoteWorker>>>) {
      for failed_worker in failed_worker_receiver {
          let mut workers = self.workers_mutex.lock().unwrap();
          let id = failed_worker.lock().unwrap().id;
          workers.remove(&id);
          log::info!("Removing worker {} from master list.", id);
      }
  }

  // handle_retry_operations will handle retry operations.
  fn handle_retry_operations(&self, retry_operation_receiver: mpsc::Receiver<Operation>) {
      // Implement your logic for handling retry operations here
      // Use retry_operation_receiver as needed
      // Example:
      // for operation in retry_operation_receiver.iter() {
      //     // Handle the retry operation
      // }
  }

  // Handle a single connection until it's done, then close it.
  fn handle_connection(
      &self,
      conn: TcpStream,
      task: Arc<Task>,
      workers_mutex: Arc<Mutex<HashMap<i32, Arc<Mutex<RemoteWorker>>>>>,
      idle_worker_sender: mpsc::Sender<Arc<Mutex<RemoteWorker>>>,
      failed_worker_sender: mpsc::Sender<Arc<Mutex<RemoteWorker>>>,
  ) {
      // Implement your logic for handling a connection here
      // Use task, workers_mutex, idle_worker_sender, failed_worker_sender as needed
      // Example:
      // let remote_worker = RemoteWorker::new(); // You may need to implement a new method for RemoteWorker
      // self.idle_worker_chan.send(remote_worker).unwrap();
  }

  // Other methods as needed...
}