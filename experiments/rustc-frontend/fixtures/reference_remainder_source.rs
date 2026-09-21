//! Native consumer of the actual original three Rust crates.
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut parts = line.split_whitespace();
        let left = f64::from_bits(u64::from_str_radix(parts.next().unwrap(), 16).unwrap());
        let right = f64::from_bits(u64::from_str_radix(parts.next().unwrap(), 16).unwrap());
        assert!(parts.next().is_none());
        let result = remainder_root_model::remainder(left, right);
        if result.is_nan() {
            println!("nan");
        } else {
            println!("{:016x}", result.to_bits());
        }
    }
}
