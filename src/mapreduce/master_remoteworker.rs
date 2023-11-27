#[derive(Debug, Clone)]
pub enum WorkerStatus {
  Idle,
  Running,
}

#[derive(Debug, Clone)]
pub struct RemoteWorker {
  id: usize,
  hostname: String,
  status: WorkerStatus,
}

impl RemoteWorker {
    pub fn new_worker(id: usize, hostname: String) -> RemoteWorker {
        let worker = RemoteWorker {
            id,
            hostname,
            status: WorkerStatus::Idle,
        };

        return worker;
    }
}

// Call a RemoteWork with the procedure specified in parameters. It will also handle connecting
// to the server and closing it afterwards.
// TODO: