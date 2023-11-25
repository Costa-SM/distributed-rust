mod common;

use std::path::PathBuf;
use common::{Task, KeyValue};

const REDUCE_PATH: &str = "reduce";
const RESULT_PATH: &str = "result";
const OPEN_FILE_MAX_RETRY: u8 = 3;

// Returns the name of files created after merge
pub fn merge_reduce_name(id_reduce: i32) -> String {
    format!("reduce-{id_reduce}")
}

// Returns the name of files created after map
pub fn reduce_name(id_map: i32, id_reduce: i32) -> String {
    format!("reduce-{id_map}-{id_reduce}")
}

// Store result from map operation locally.
// This will store the result from all the map calls.
pub fn store_local(task: &Task, id_map_task: i32, data: &[KeyValue]) {
    let err;
}

pub fn result_file_name(id: i32) -> String {
    let file_name = format!("result-{}", id);
    let file_path = PathBuf::from(RESULT_PATH).join(file_name);
    file_path.to_string_lossy().to_string()
}