//! Handwritten pinned-Rust oracle cross-check; not a generated output library.
use std::io::{self, Read};

fn observe(value: f64) {
    if value.is_nan() {
        println!("nan");
    } else {
        println!("{:016x}", value.to_bits());
    }
}
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    for line in input.lines() {
        let pair: Vec<_> = line.split_whitespace().collect();
        assert_eq!(pair.len(), 2);
        let left = f64::from_bits(u64::from_str_radix(pair[0], 16).unwrap());
        let right = f64::from_bits(u64::from_str_radix(pair[1], 16).unwrap());
        for value in [left + right, left - right, left * right, left / right] {
            observe(value);
        }
    }
}
