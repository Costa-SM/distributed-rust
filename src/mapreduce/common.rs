use std::sync::mpsc::{Sender, Receiver, channel};
use serde::{Serialize, Deserialize};

// KeyValue is the type used to hold elements of maps and reduces results.
#[derive(Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

// Task is the exposed struct of the Framework that the calling code should initialize
// with the specific implementation of the operation.
#[derive(Debug)]
pub struct Task {
    // // MapReduce functions
    // map: MapFunc,
    // pub shuffle: ShuffleFunc,
    // reduce: ReduceFunc,

    // Jobs
    pub num_reduce_jobs: i32,
    num_map_files: i32,

    // Channels for data
    input_chan: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    output_chan: (Sender<Vec<KeyValue>>, Receiver<Vec<KeyValue>>),

    // Channels for file paths
    input_file_path_chan: (Sender<String>, Receiver<String>),
    output_file_path_chan: (Sender<String>, Receiver<String>),
}

impl Task {
    pub fn new_task() -> Task {
        let task = Task {
            num_reduce_jobs: 0,
            num_map_files: 0,
            input_chan: channel(),
            output_chan: channel(),

            input_file_path_chan: channel(),
            output_file_path_chan: channel(),
        };

        return task;
    }
}

pub struct Operation {
    proc: String,
    file_path: String,
    id: i32,
}

type MapFunc = fn(Vec<u8>) -> Vec<KeyValue>;
type ReduceFunc = fn(Vec<KeyValue>) -> Vec<KeyValue>;
type ShuffleFunc = fn(&Task, String) -> i32;
