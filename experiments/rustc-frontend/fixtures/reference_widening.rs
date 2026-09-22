//! Native signed widening through two language/library spellings, not oracle truth.
use std::io::{self, BufRead};

fn main() {
    for line in io::stdin().lock().lines() {
        let value: i32 = line.unwrap().parse().unwrap();
        println!("{} {}", value as i64, i64::from(value));
    }
}
