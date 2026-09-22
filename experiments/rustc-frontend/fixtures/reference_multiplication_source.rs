//! Execute the original source crates; tracing instruments only original leaves.
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let values: Vec<_> = line.split_whitespace().collect();
        assert_eq!(values.len(), 3);
        match values[0] {
            "32" => println!(
                "{}",
                multiplication_root_model::multiplication32(
                    values[1].parse().unwrap(),
                    values[2].parse().unwrap()
                )
            ),
            "64" => println!(
                "{}",
                multiplication_root_model::multiplication64(
                    values[1].parse().unwrap(),
                    values[2].parse().unwrap()
                )
            ),
            _ => panic!("width"),
        }
    }
}
