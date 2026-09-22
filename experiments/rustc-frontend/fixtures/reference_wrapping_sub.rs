//! Native builtin observations; expected differences use an independent integer oracle.
use std::io::{self, BufRead};

fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut parts = line.split_whitespace();
        let width = parts.next().unwrap();
        let left = parts.next().unwrap();
        let right = parts.next().unwrap();
        assert!(parts.next().is_none());
        match width {
            "32" => {
                let left: i32 = left.parse().unwrap();
                let right: i32 = right.parse().unwrap();
                println!("{}", left.wrapping_sub(right));
            }
            "64" => {
                let left: i64 = left.parse().unwrap();
                let right: i64 = right.parse().unwrap();
                println!("{}", left.wrapping_sub(right));
            }
            _ => panic!("unexpected signed width"),
        }
    }
}
