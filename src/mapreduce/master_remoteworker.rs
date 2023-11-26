enum WorkerStatus {
  Idle,
  Running,
}

struct RemoteWorker {
  id: i32,
  hostname: String,
  status: WorkerStatus,
}

// Call a RemoteWork with the procedure specified in parameters. It will also handle connecting
// to the server and closing it afterwards.
// TODO: