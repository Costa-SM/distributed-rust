use std::sync::mpsc::{Sender, Receiver, channel};
use crate::word_count;

// Task is the exposed struct of the Framework that the calling code should initialize
// with the specific implementation of the operation.
#[derive(Debug)]
pub struct Task {
    // MapReduce functions
    map: MapFunc,
    // pub shuffle: ShuffleFunc,
    reduce: ReduceFunc,

    // Jobs
    pub num_reduce_jobs: i32,
    num_map_files: i32,

    // Channels for data
    input_chan: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    output_chan: (Sender<Vec<word_count::KeyValue>>, Receiver<Vec<word_count::KeyValue>>),

    // Channels for file paths
    input_file_path_chan: (Sender<String>, Receiver<String>),
    output_file_path_chan: (Sender<String>, Receiver<String>),
}

impl Task {
    pub fn new_task(map: MapFunc, reduce: ReduceFunc) -> Task {
        let task = Task {
            // Map and reduce functions
            map,
            reduce,

            // Jobs
            num_reduce_jobs: 0,
            num_map_files: 0,
            
            // Channels for data
            input_chan: channel(),
            output_chan: channel(),

            // Channels for file paths
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

type MapFunc = fn(&[u8]) -> Vec<word_count::KeyValue>;
type ReduceFunc = fn(&mut Vec<word_count::KeyValue>) -> &mut Vec<word_count::KeyValue>;
type ShuffleFunc = fn(&Task, String) -> i32;
