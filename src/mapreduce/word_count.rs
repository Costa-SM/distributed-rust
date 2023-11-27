use serde::{Serialize, Deserialize};
use crate::common::Task;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::Wrapping;

// KeyValue is the type used to hold elements of maps and reduces results.
#[derive(Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: i32
}

// This function receives a string obtained from the split files.
pub fn map_func(buffer: &[u8]) -> Vec<KeyValue> {
    // This considers UTF-8 encoding.
    let words = match std::str::from_utf8(buffer) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    // Make all characters lowercase, and remove the punctuation. Then, separate words by whitespace.
    let words = words.to_ascii_lowercase().replace(&['(', ')', ',', '\"', '.', ';', ':', '\''][..], "");
    let split = words.split_ascii_whitespace();
    
    // Create a vector to store the words.
    let mut result: Vec<KeyValue> = Vec::new();

    for word in split {
        result.push(KeyValue{key: word.to_string(), value: 1})
    }

    return result
}

pub fn reduce_func(inputs: &mut Vec<KeyValue>) -> &mut Vec<KeyValue> {
    // Find repeated words and their indices, to remove them from the key/value pairs.
    // Use the KeyValue struct as an indicator of the word and the position.
    let mut removed_words: Vec<KeyValue> = Vec::new();

    for (iter, item) in inputs.iter().enumerate() {
        let indices = inputs
                                        .iter()
                                        .enumerate()
                                        .filter(|(i, k)| k.key == item.key && i > &iter)
                                        .map(|(index, _)| index)
                                        .collect::<Vec<_>>();
        
        for index in indices {
            removed_words.push(KeyValue {key: item.key.clone(), value: index as i32});
        }
    }

    // Sort the indices so that when they are removed from the vector, they are not shifted around.
    removed_words.sort_unstable_by(|a, b| b.value.cmp(&a.value));

    // Remove repeated words from the vector.
    for word in removed_words {
        let index = inputs.iter().position(|w| w.key == word.key).unwrap();

        inputs[index].value += 1;
        inputs.remove(word.value as usize);
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

/* Main function for testing the Word Count algorithm. */
fn main() {
    let test_string: &str = "hello, this is a TeSt tEst. stRIng; with soME: repeated. words words";
    let mut dict = map_func(test_string.as_bytes());

    let reduced_dict = reduce_func(&mut dict);
    
    for item in reduced_dict.clone() {
        println!("Word/Value: {} / {}", item.key, item.value);
    }
}
