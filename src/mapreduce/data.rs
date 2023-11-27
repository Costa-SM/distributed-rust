use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path;
use std::thread::sleep;
use std::time::Duration;

use serde_json;
use tokio::sync::mpsc::{self, Sender, Receiver};

use crate::common;

pub const MAP_PATH: &str = "map";
pub const REDUCE_PATH: &str = "reduce";
pub const RESULT_PATH: &str = "result";
pub const OPEN_FILE_MAX_RETRY: u8 = 3;

const MAP_BUFFER_SIZE: usize = 10;
const REDUCE_BUFFER_SIZE: usize = 10;

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
// NOTE: TESTED
pub fn store_local(task: &common::Task, id_map_task: i32, data: &Vec<common::KeyValue>) -> io::Result<()> {
    for r in 0..task.num_reduce_jobs {
        let file_path = path::Path::new(REDUCE_PATH).join(reduce_name(id_map_task, r));
        let mut file = File::create(&file_path).expect("Error creating file");
        
        for kv in data {
            if (task.shuffle)(task, kv.key.clone()) == r {
                let json = serde_json::to_string(&kv)?;
                file.write_all(json.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }
    }

    Ok(())
}

// Merge the result from all the map operations by reduce job id.
// NOTE: TESTED
pub fn merge_map_local(task: &common::Task, map_counter: i32) -> io::Result<()> {
    for r in 0..task.num_reduce_jobs {
        let merged_file_path = path::Path::new(REDUCE_PATH).join(merge_reduce_name(r));
        let mut merged_file = File::create(merged_file_path)?;

        for m in 0..map_counter {
            // Use max number of retries to open the file
            let file_path = path::Path::new(REDUCE_PATH).join(reduce_name(m, r));
            for i in 0..OPEN_FILE_MAX_RETRY {
                if let Ok(_f) = File::open(&file_path) {
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

            let mut file = File::open(&file_path)?;
            // Read the contents of the file and write to the destination file
            io::copy(&mut file, &mut merged_file)?;
        }
    }

    Ok(())
}

// Merge the result from all the map operations by reduce job id.
// NOTE: TESTED
pub fn merge_reduce_local(reduce_counter: i32) -> io::Result<()> {
    let merged_file_path = path::Path::new(RESULT_PATH).join("result-final.txt");
    let mut merged_file = File::create(merged_file_path)?;

    for r in 0..reduce_counter {
        // Use max number of retries to open the file
        let file_path = result_file_name(r);
        for i in 0..OPEN_FILE_MAX_RETRY {
            if let Ok(_f) = File::open(&file_path) {
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

        let mut file = File::open(&file_path)?;
        // Read the contents of the file and write to the destination file
        io::copy(&mut file, &mut merged_file)?;
    }

    Ok(())
}

// Load data for reduce jobs.
// NOTE: TESTED
pub fn load_local(id_reduce: i32) -> io::Result<Vec<common::KeyValue>> {
    let file_path = path::Path::new(REDUCE_PATH).join(merge_reduce_name(id_reduce));

    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    let mut data = Vec::new();
    for result in serde_json::Deserializer::from_reader(reader).into_iter::<common::KeyValue>() {
        let kv = result?;
        data.push(kv);
    }

    Ok(data)
}

// Remove all the files in a directory
// NOTE: TESTED
pub fn remove_contents(dir: &str) -> io::Result<()> {
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
pub fn fan_reduce_file_path(num_reduce_jobs: i32) -> Receiver<String> {
    let (output_tx, output_rx) = mpsc::channel(1);

    tokio::spawn(async move {
        for i in 0..num_reduce_jobs {
            let file_path = path::Path::new(REDUCE_PATH).join(merge_reduce_name(i));

            if let Some(file_path_str) = file_path.to_str() {
                if let Err(_) = output_tx.send(file_path_str.to_string()).await {
                    break;
                }
            }
        }

        drop(output_tx);
    });

    output_rx
}

// Reads input file and split it into files smaller than chunkSize.
fn split_data(filename: &str, chunk_size: usize) -> io::Result<usize> {
    // Read the content of the input file
    let mut input_file = File::open(filename)?;
    let mut data = Vec::new();
    input_file.read_to_end(&mut data)?;

    // Calculate the number of chunks
    let num_map_files = (data.len() + chunk_size - 1) / chunk_size;

    // Create and write chunks to separate files
    for i in 0..num_map_files {
        let start = i * chunk_size;
        let end = (i + 1) * chunk_size;
        let chunk = &data[start..end];

        // Create a new file for each chunk
        let output_filename = format!("{}_chunk_{}.txt", filename, i + 1);
        let mut output_file = File::create(output_filename)?;
        output_file.write_all(chunk)?;
    }

    Ok(num_map_files)
}

// Support function to generate the name of map files.
// NOTE: TESTED
pub fn map_file_name(id: i32) -> path::PathBuf {
    let file_name = format!("map-{id}");
    let file_path = path::Path::new(MAP_PATH).join(file_name);
    file_path
}

// Support function to generate the name of result files.
// NOTE: TESTED
pub fn result_file_name(id: i32) -> path::PathBuf {
    let file_name = format!("result-{}", id);
    let file_path = path::Path::new(RESULT_PATH).join(file_name);
    file_path
}

// fanInFilePath will run a goroutine that returns the path of files created during
// splitData. These paths will be sent to remote workers so they can access the data
// and run map operations on it.
pub fn fan_in_file_path(num_files: i32) -> Receiver<String> {
    let (output_tx, output_rx) = mpsc::channel(1);

    tokio::spawn(async move {
        for i in 0..num_files {
            let file_path = map_file_name(i);

            if let Some(file_path_str) = file_path.to_str() {
                if let Err(_) = output_tx.send(file_path_str.to_string()).await {
                    break;
                }
            }
        }

        drop(output_tx);
    });

    output_rx
}

// fanInData will run a goroutine that reads files crated by splitData and share them with
// the mapreduce framework through the one-way channel. It'll buffer data up to
// MAP_BUFFER_SIZE (files smaller than chunkSize) and resume loading them
// after they are read on the other side of the channle (in the mapreduce package)
pub fn fan_in_data(num_files: i32) -> Receiver<Vec<u8>> {
    let (output_tx, output_rx) = mpsc::channel(MAP_BUFFER_SIZE);

    tokio::spawn(async move {
        for i in 0..num_files {
            let file_path = map_file_name(i);

            if let Ok(file) = File::open(&file_path) {
                let mut reader = BufReader::new(file);
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer).unwrap();

                if let Err(_) = output_tx.send(buffer).await {
                    break;
                }
            }
        }

        drop(output_tx);
    });

    output_rx
}

// fanOutData will run a goroutine that receive data on the one-way channel and will
// proceed to store it in their final destination. The data will come out after the
// reduce phase of the mapreduce model.
pub fn fun_out_data() -> (Sender<Vec<u8>>, Receiver<bool>) {
    let (output_tx, output_rx) = mpsc::channel(REDUCE_BUFFER_SIZE);
    let (done_tx, done_rx) = mpsc::channel(REDUCE_BUFFER_SIZE);

    tokio::spawn(async move {
        while let Some(data) = output_rx.recv().await {
            let file_path = result_file_name(0);
            let mut file = File::create(&file_path).expect("Error creating file");

            file.write_all(&data).expect("Error writing to file");
        }

        drop(output_tx);
    });

    (output_tx, done_rx)
}