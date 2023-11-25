use std::sync::mpsc::{Sender, Receiver};

// KeyValue is the type used to hold elements of maps and reduces results.
pub struct KeyValue {
    key: String,
    value: String,
}

// Task is the exposed struct of the Framework that the calling code should initialize
// with the specific implementation of the operation.
pub struct Task {
    // MapReduce functions
    map: MapFunc,
    shuffle: ShuffleFunc,
    reduce: ReduceFunc,

    // Jobs
    num_reduce_jobs: i32,
    num_map_files: i32,

    // Channels for data
    input_chan: Sender<Vec<u8>>,
    output_chan: Receiver<Vec<KeyValue>>,

    // Channels for file paths
    input_file_path_chan: Sender<String>,
    output_file_path_chan: Receiver<String>,
}

type MapFunc = fn(Vec<u8>) -> Vec<KeyValue>;
type ReduceFunc = fn(Vec<KeyValue>) -> Vec<KeyValue>;
type ShuffleFunc = fn(&Task, String) -> i32;