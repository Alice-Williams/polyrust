//! Execute the original crates, not a rewritten cast.
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let value = line.unwrap().parse().unwrap();
        println!("{}", widening_root_model::widen(value));
    }
}
