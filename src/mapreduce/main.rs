mod common;
mod data;
mod mapreduce;
mod master;
mod word_count;
mod worker;

use tokio::runtime;
use clap::{App, Arg};
use std::fs;

fn main() {
    let matches = App::new("MapReduce")
        // Run mode settings
        .arg(
            Arg::with_name("mode")
                .short("o")
                .long("mode")
                .value_name("MODE")
                .help("Run mode: distributed or sequential")
                .takes_value(true)
                .default_value("distributed"),
        )
        .arg(
            Arg::with_name("type")
                .short("t")
                .long("type")
                .value_name("TYPE")
                .help("Node type: master or worker")
                .takes_value(true)
                .default_value("worker"),
        )
        .arg(
            Arg::with_name("reducejobs")
                .short("r")
                .long("reducejobs")
                .value_name("NUM")
                .help("Number of reduce jobs that should be run")
                .takes_value(true)
                .default_value("5"),
        )
        // Input data settings
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("File to use as input")
                .takes_value(true)
                .default_value("files/pg1342.txt"),
        )
        .arg(
            Arg::with_name("chunksize")
                .short("c")
                .long("chunksize")
                .value_name("SIZE")
                .help("Size of data chunks that should be passed to map jobs (in bytes)")
                .takes_value(true)
                .default_value("102400"), // Corresponds to 100*1024
        )
        // Network settings
        .arg(
            Arg::with_name("addr")
                .short("a")
                .long("addr")
                .value_name("ADDRESS")
                .help("IP address to listen on")
                .takes_value(true)
                .default_value("localhost"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("TCP port to listen on")
                .takes_value(true)
                .default_value("5000"),
        )
        .arg(
            Arg::with_name("master")
                .short("m")
                .long("master")
                .value_name("MASTER")
                .help("Master address")
                .takes_value(true)
                .default_value("localhost:5000"),
        )
        // Induced failure on Worker
        .arg(
            Arg::with_name("fail")
                .short("n")
                .long("fail")
                .value_name("NUM")
                .help("Number of operations to run before failure")
                .takes_value(true)
                .default_value("0"),
        )
        .get_matches();

    // Access the values using unwrap_or_else to provide defaults
    let mode = matches.value_of("mode").unwrap_or_else(|| "distributed");
    let node_type = matches.value_of("type").unwrap_or_else(|| "worker");
    let reduce_jobs = matches
        .value_of("reducejobs")
        .unwrap_or_else(|| "5")
        .parse::<i32>()
        .unwrap(); // Parse to i32

    let file = matches.value_of("file").unwrap_or_else(|| "files/pg1342.txt");
    let chunk_size = matches
        .value_of("chunksize")
        .unwrap_or_else(|| "102400")
        .parse::<usize>()
        .unwrap(); // Parse to usize

    let addr = matches.value_of("addr").unwrap_or_else(|| "localhost");
    let port = matches
        .value_of("port")
        .unwrap_or_else(|| "5000")
        .parse::<u16>()
        .unwrap(); // Parse to u16

    let master = matches.value_of("master").unwrap_or_else(|| "localhost:5000");

    let n_ops = matches
        .value_of("fail")
        .unwrap_or_else(|| "0")
        .parse::<i32>()
        .unwrap(); // Parse to i32

    let _ = fs::create_dir(data::MAP_PATH);
    let _ = fs::create_dir(data::RESULT_PATH);

    let mut task = common::Task::new_task(word_count::map_func, word_count::shuffle_func, word_count::reduce_func);
    task.num_reduce_jobs = reduce_jobs;    

    let rt = runtime::Runtime::new().unwrap();

    println!("Running MapReduce in {} mode.", mode);
    
    match mode {
        "sequential" => {
            rt.block_on(async {
                if let Err(err) = data::remove_contents(data::MAP_PATH) {
                    eprintln!("Error removing contents: {}", err);
                }
                if let Err(err) = data::remove_contents(data::RESULT_PATH) {
                    eprintln!("Error removing contents: {}", err);
                }

                let num_files = data::split_data(file, chunk_size);

                let fan_in = data::fan_in_data(num_files as i32);
                let (fan_out, mut wait_for_it) = data::fan_out_data();

                task.input_chan = fan_in;

                mapreduce::run_sequential(&mut task, fan_out).await;

                // Wait for the output channel to close
                wait_for_it.recv().await.unwrap();
            });
        },
        "distributed" => match node_type {
            "master" => {
                println!("Node type: {}", node_type);
                println!("Reduce jobs: {}", reduce_jobs);
                println!("Address: {}", addr);
                println!("Port: {}", port);
                println!("File: {}", file);
                println!("Chunk size: {}", chunk_size);

                if let Err(err) = data::remove_contents(data::MAP_PATH) {
                    eprintln!("Error removing contents: {}", err);
                }
                if let Err(err) = data::remove_contents(data::RESULT_PATH) {
                    eprintln!("Error removing contents: {}", err);
                }

                let hostname = format!("{}:{}", addr, port);

                let num_files = data::split_data(file, chunk_size);

                let fan_in = data::fan_in_file_path(num_files as i32);
                task.input_file_path_chan = fan_in;

                // mapreduce::run_master(&task, hostname);
            }
            "worker" => {
                println!("Node type: {}", node_type);
                println!("Address: {}", addr);
                println!("Port: {}", port);
                println!("Master: {}", master);

                if n_ops > 0 {
                    println!("Induced failure");
                    println!("After {} operations.", n_ops);
                }

                let hostname = format!("{}:{}", addr, port);
                // mapreduce::run_worker(&task, hostname, master, n_ops);
            }
            _ => println!("Invalid node type: {}", node_type),
        },
        _ => println!("Invalid mode: {}", mode),
    }


}
