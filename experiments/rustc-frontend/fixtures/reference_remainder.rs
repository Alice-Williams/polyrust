//! Pinned-Rust % reference, never used to compute expected oracle values.
use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    for line in input.lines() {
        let pair: Vec<_> = line.split_whitespace().collect();
        assert_eq!(pair.len(), 2);
        let left = f64::from_bits(u64::from_str_radix(pair[0], 16).unwrap());
        let right = f64::from_bits(u64::from_str_radix(pair[1], 16).unwrap());
        let value = left % right;
        if value.is_nan() {
            println!("nan");
        } else {
            println!("{:016x}", value.to_bits());
        }
    }
}
