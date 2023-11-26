mod common;

use std::fs;
use std::path::{Path, PathBuf};
use serde_json;
use std::thread::sleep;
use std::time::Duration;
use common::{Task, KeyValue};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Error};

const REDUCE_PATH: &str = "reduce";
const RESULT_PATH: &str = "result";
const OPEN_FILE_MAX_RETRY: u8 = 3;

// Returns the name of files created after merge
// NOTE: TESTED
pub fn merge_reduce_name(id_reduce: i32) -> String {
    format!("reduce-{id_reduce}")
}

// Returns the name of files created after map
// NOTE: TESTED
pub fn reduce_name(id_map: i32, id_reduce: i32) -> String {
    format!("reduce-{id_map}-{id_reduce}")
}

// Store result from map operation locally.
// This will store the result from all the map calls.
pub fn store_local(task: &Task, id_map_task: i32, data: Vec<KeyValue>) {
    for r in 0..task.num_reduce_jobs {
        let file_path = Path::new(REDUCE_PATH).join(reduce_name(id_map_task, r));
        let file = fs::File::create(&file_path).expect("Error creating file");
        let mut file_encoder = serde_json::to_writer(file, &data).expect("Error encoding JSON");
        file_encoder.flush().expect("Error flushing buffer");
    }
}

// Merge the result from all the map operations by reduce job id.
pub fn merge_map_local(task: &Task, map_counter: i32) {
    for r in 0..task.num_reduce_jobs {
        let merge_file_path = Path::new(REDUCE_PATH).join(merge_reduce_name(r));
        let merge_file = fs::File::create(&merge_file_path).expect("Error creating file");
        let mut merge_file_encoder =
            serde_json::to_writer(merge_file, &()).expect("Error encoding JSON");

        for m in 0..map_counter {
            for i in 0..OPEN_FILE_MAX_RETRY {
                let file_path = Path::new(REDUCE_PATH).join(reduce_name(m, r));
                if let Ok(file) = fs::File::open(&file_path) {
                    // Read from the file
                    break;
                }
                eprintln!(
                    "({}/{}) Failed to open file {}. Retrying in 1 second...",
                    i + 1,
                    OPEN_FILE_MAX_RETRY,
                    file_path.display()
                );
                sleep(Duration::from_secs(1));
            }
        }
    }
}

// Merge the result from all the map operations by reduce job id.
pub fn merge_reduce_local(reduce_counter: i32) -> Result<(), Error> {
    let merge_file_path = PathBuf::from(RESULT_PATH).join("result-final.txt");
    let merge_file = OpenOptions::new().create(true).write(true).truncate(true).open(&merge_file_path)?;

    let mut merge_file_encoder = BufWriter::new(serde_json::to_writer(merge_file, &[])?);

    for r in 0..reduce_counter {
        let mut file: Option<File> = None;

        for i in 0..OPEN_FILE_MAX_RETRY {
            match File::open(result_file_name(r)) {
                Ok(f) => {
                    file = Some(f);
                    break;
                }
                Err(_) => {
                    eprintln!(
                        "({}/{}) Failed to open file {}. Retrying in 1 second...",
                        i + 1,
                        OPEN_FILE_MAX_RETRY,
                        result_file_name(r)
                    );
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }

        if let Some(mut file) = file {
            let file_reader = BufReader::new(&mut file);

            for result in serde_json::Deserializer::from_reader(file_reader).into_iter::<KeyValue>() {
                let kv = result?;
                merge_file_encoder.write_all(serde_json::to_string(&kv)?.as_bytes())?;
                merge_file_encoder.write_all(b"\n")?;
            }

            file.sync_all()?;
        }
    }

    Ok(())
}

// Load data for reduce jobs.
pub fn load_local(id_reduce: i32) -> Result<Vec<KeyValue>, Error> {
    let file_path = PathBuf::from(REDUCE_PATH).join(merge_reduce_name(id_reduce));

    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    let mut data = Vec::new();
    for result in serde_json::Deserializer::from_reader(reader).into_iter::<KeyValue>() {
        let kv = result?;
        data.push(kv);
    }

    Ok(data)
}

// Remove all the files in a directory
// NOTE: TESTED
pub fn remove_contents(dir: &str) -> Result<(), std::io::Error> {
    let d = fs::read_dir(dir)?;

    for entry in d {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }

    Ok(())
}

// FanIn is a pattern that will return a channel in which the goroutines generated here will keep
// writing until the loop is done.
// This is used to generate the name of all the reduce files.
pub fn fan_reduce_file_path(num_reduce_jobs: i32) -> (Sender<String>, Receiver<String>) {
    let (output_tx, output_rx) = mpsc::channel();

    thread::spawn(move || {
        for i in 0..num_reduce_jobs {
            let file_path = PathBuf::from(REDUCE_PATH).join(merge_reduce_name(i));

            if let Some(file_path_str) = file_path.to_str() {
                if output_tx.send(file_path_str.to_string()).is_err() {
                    break;
                }
            }
        }
        // Close the channel when the loop is done
        drop(output_tx); 
        drop(output_rx);
    });

    (output_tx, output_rx)
}

// Support function to generate the name of result files.
// NOTE: TESTED
pub fn result_file_name(id: i32) -> String {
    let file_name = format!("result-{}", id);
    let file_path = PathBuf::from(RESULT_PATH).join(file_name);
    file_path.to_string_lossy().to_string()
}