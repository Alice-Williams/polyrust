//! Observe the actual source library, with per-input trace separators.
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let bits = u64::from_str_radix(&line.unwrap(), 16).unwrap();
        println!(
            "{}",
            u8::from(negative_zero_source::is_negative_zero(f64::from_bits(bits)))
        );
        eprint!("|");
    }
}
