// RunSequential will ensure that map and reduce function runs in
// a single-core linearly. The Task is passed from the calling package
// and should contains the definitions for all the required functions
// and parameters.
// Notice that this implementation will store data locally. In the distributed
// version of mapreduce it's common to store the data in the same worker that computed
// it and just pass a reference to reduce jobs so they can go grab it.
pub fn run_sequential(task: &Task) {
  let mut map_counter = 0;
  let mut map_result: Vec<KeyValue> = Vec::new();

  println!("Running RunSequential...");

  // Create or clear the REDUCE_PATH directory
  let _ = fs::create_dir(REDUCE_PATH);
  let _ = remove_contents(REDUCE_PATH);

  while let Ok(v) = task.input_chan.1.recv() {
      map_result = task.map(v);
      store_local(task, map_counter, &map_result);
      map_counter += 1;
  }

  merge_map_local(task, map_counter);

  for r in 0..task.num_reduce_jobs {
      if let Ok(data) = load_local(r) {
          task.output_chan.0.send(task.reduce(data)).unwrap();
      }
  }

  // Close the output channel
  drop(task.output_chan.0);
  drop(task.output_chan.1);
}

// TODO: implement the rest of the functions once Master and Worker are implemented
