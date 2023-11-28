use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::{self, Sender, Receiver};

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

pub struct Operation {
    pub proc: String,
    pub file_path: String,
    pub id: i32,
}

// Task is the exposed struct of the Framework that the calling code should initialize
// with the specific implementation of the operation.
#[derive(Debug)]
pub struct Task {
    // MapReduce functions
    pub map: MapFunc,
    pub shuffle: ShuffleFunc,
    pub reduce: ReduceFunc,

    // Jobs
    pub num_reduce_jobs: i32,
    pub num_map_files: i32,

    // Channels for data
    pub input_chan: Receiver<Vec<u8>>,
    pub output_chan: Sender<Vec<KeyValue>>,

    // Channels for file paths
    pub input_file_path_chan: Receiver<String>,
    pub output_file_path_chan: Sender<String>,
}

impl Task {
    pub fn new_task(map: MapFunc, shuffle: ShuffleFunc, reduce: ReduceFunc) -> Task {
        let task = Task {
            // Map and reduce functions
            map,
            shuffle,
            reduce,

            // Jobs
            num_reduce_jobs: 1,
            num_map_files: 1,
            
            // Channels for data
            input_chan: mpsc::channel(1).1,
            output_chan: mpsc::channel(1).0,

            // Channels for file paths
            input_file_path_chan: mpsc::channel(1).1,
            output_file_path_chan: mpsc::channel(1).0,
        };

        return task;
    }
}

type MapFunc = fn(&Vec<u8>) -> Vec<KeyValue>;
type ReduceFunc = fn(&mut Vec<KeyValue>) -> &mut Vec<KeyValue>;
type ShuffleFunc = fn(&Task, String) -> i32;
