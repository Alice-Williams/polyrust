use same_spelling_model as api;
use std::io::{self, BufRead};

fn main() {
    for line in io::stdin().lock().lines() {
        let value: i32 = line.unwrap().parse().unwrap();
        println!("{} {}", api::first_value(value), api::second_value(value));
    }
}
