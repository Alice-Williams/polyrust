use std::io::{self, BufRead};
use wrapping_root as model;
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("line");
        let mut parts = line.split_whitespace();
        let width = parts.next().expect("width");
        let value: i64 = parts.next().expect("value").parse().expect("integer");
        match width {
            "32" => {
                let value = i32::try_from(value).expect("width");
                println!(
                    "{} {} {} {} {} {}",
                    model::method32(value),
                    model::associated32(value),
                    model::nested32(value),
                    model::leaf32(value),
                    model::ordinary32(value),
                    model::absolute32(value)
                );
            }
            "64" => {
                println!(
                    "{} {} {} {} {} {}",
                    model::method64(value),
                    model::associated64(value),
                    model::nested64(value),
                    model::leaf64(value),
                    model::ordinary64(value),
                    model::absolute64(value)
                );
            }
            _ => panic!("width"),
        }
    }
}
