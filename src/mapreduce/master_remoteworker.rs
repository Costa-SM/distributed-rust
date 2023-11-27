#[derive(Debug)]
pub enum WorkerStatus {
  Idle,
  Running,
}

#[derive(Debug)]
pub struct RemoteWorker {
  id: i32,
  hostname: String,
  status: WorkerStatus,
}

impl RemoteWorker {
    pub fn new_worker(id: i32, hostname: String) -> RemoteWorker {
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