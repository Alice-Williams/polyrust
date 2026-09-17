//! Handwritten consumer of the actual three Rust crates, not a target renderer.
use std::io::{self, BufRead};

fn observe(value: f64) {
    if value.is_nan() {
        println!("nan");
    } else {
        println!("{:016x}", value.to_bits());
    }
}

fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut parts = line.split_whitespace();
        let left = f64::from_bits(u64::from_str_radix(parts.next().unwrap(), 16).unwrap());
        let right = f64::from_bits(u64::from_str_radix(parts.next().unwrap(), 16).unwrap());
        assert!(parts.next().is_none());
        let functions = [
            arithmetic_root_model::add,
            arithmetic_root_model::subtract,
            arithmetic_root_model::multiply,
            arithmetic_root_model::divide,
            arithmetic_root_model::grouped,
            arithmetic_root_model::separate,
            arithmetic_root_model::nested_division,
        ];
        for function in functions {
            observe(function(left, right));
        }
    }
}
