use std::io::{self, BufRead};

fn main() {
    for line in io::stdin().lock().lines() {
        let value: i32 = line.unwrap().parse().unwrap();
        println!("{}", direct_calls_model::score(value));
    }
}
