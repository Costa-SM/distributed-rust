# Distributed Rust

Rust implementation of distributed processing algorithms, meant for comparison with other programming languages, such as Golang.

Final exam for the CSC-27 course at the Aeronautics Institute of Technology. Group members:
- Arthur Stevenson
- Eduardo Simplicio
- Mateus Cristo 
- Matheus Ramos

# MapReduce - Project

This project implements a MapReduce system in Rust, a distributed processing approach for handling large datasets. It includes both the implementation for distributed and sequential execution of MapReduce.

## Key Features

- **Distributed and Sequential Execution Modes:** The system provides the flexibility to run tasks in a distributed environment using a Master and Workers or in sequential mode without the need for Workers and Master.

- **Flexible Configuration:** Using the `clap` library, the system allows configuring various parameters, including execution mode, node type (Master or Worker), number of reduce jobs, input file, chunk size, IP address, port, master, and the ability to induce failures in Workers.

## Project Structure

The project is divided into several modules:

- **`common`:** Contains common structures and definitions, such as the `KeyValue` structure and the MapReduce task.

- **`data`:** Handles data manipulation, including data splitting into chunks, local storage and loading operations, and directory content removal.

- **`word_count`:** Example mapping, reducing, and shuffling functions for word counting.

- **`mapreduce`:** Contains the specific logic for MapReduce, both for distributed and sequential execution.

- **`master`:** Implements the Master in a distributed environment.

- **`worker`:** Implements the limited Worker in a distributed environment.

- **`main`:** The entry point of the program, argument configuration, and main logic to determine the execution mode.

## Compiling the Program

To compile the program and download all external dependencies, just run the following command:

```bash
cargo build
```

## Running the Program

### Sequential Mode

To run the program in sequential mode, use the following command:

```bash
cargo run --bin main -- --mode sequential --chunksize 51200 --reducejobs 5
```

This command executes MapReduce sequentially, specifying a chunk size of 51200 bytes and 5 reduce jobs.

### Distributed Mode

To run the program in distributed mode, start the Master, and then start one or more Workers. Use the following commands:

**Master:**

```bash
cargo run --bin master
```

**Worker:**

```bash
cargo run --bin worker
```

Adjust the parameters as needed, including the number of Workers for effective distributed execution.

### GUI

It's also possible to visualize the progress of the tasks using the GUI built in Rust.
However, since there are some problems regarding the gRPC configuration, this interface is just for mere visualization.

To open it, just execute:

```bash
cargo run --bin gui
```

## Dependencies

The project uses various libraries and dependencies, including:

- `tonic`: For gRPC communication in a distributed environment.
- `clap`: For command-line argument parsing.
- `tokio`: For asynchronous execution and runtime.

Dependencies are listed in the `Cargo.toml` file and will be downloaded automatically when compiling the project.

---