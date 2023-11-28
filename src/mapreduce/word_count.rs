use crate::common::{Task, KeyValue};

use std::collections::hash_map::{HashMap, DefaultHasher};
use std::hash::{Hash, Hasher};
use std::num::Wrapping;

// KeyValue is the type used to hold elements of maps and reduces results.


// This function receives a string obtained from the split files.
pub fn map_func(buffer: &Vec<u8>) -> Vec<KeyValue> {
    // This considers UTF-8 encoding.
    let words = match std::str::from_utf8(buffer) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    // Separate words by whitespace.
    let split = words.split_ascii_whitespace();
    
    // Create a vector to store the words.
    let mut result: Vec<KeyValue> = Vec::new();

    for word in split {
        result.push(KeyValue{key: word.to_string(), value: "1".to_string()})
    }

    return result
}

pub fn reduce_func(inputs: &mut Vec<KeyValue>) -> &mut Vec<KeyValue> {
    // Hash map for counting number of times each word appears. It would be 
    // best to use the hash map from the beginning, but then the map and reduce 
    // functions would be done at the same time.
    let mut element_count = HashMap::new();
    for element in inputs.clone() {
        *element_count.entry(element.key).or_insert(0) += 1;
    }

    // Clear vector.
    inputs.clear();

    // Update values in the vector
    for (element, count) in &element_count {
        inputs.push(KeyValue { key: (element.to_string()), value: (*count).to_string() });
    }

    return inputs;
}

pub fn shuffle_func(task: &Task, key: String) -> i32 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash_value = hasher.finish();

    let num_reduce_jobs = task.num_reduce_jobs as u64;
    let reduce_job = Wrapping(hash_value) % Wrapping(num_reduce_jobs);

    reduce_job.0 as i32
}
