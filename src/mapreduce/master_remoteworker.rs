#[derive(Debug, Clone)]
pub enum WorkerStatus {
  Idle,
  Running,
}

#[derive(Debug, Clone)]
pub struct RemoteWorker {
  pub id: usize,
  pub hostname: String,
  pub status: WorkerStatus,
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