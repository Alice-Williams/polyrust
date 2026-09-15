use bitwise_values_model as model;
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("line");
        let mut parts = line.split_whitespace();
        let width = parts.next().expect("width");
        let a: i64 = parts.next().expect("a").parse().expect("integer");
        let b: i64 = parts.next().expect("b").parse().expect("integer");
        match width {
            "32" => {
                let a = i32::try_from(a).expect("width");
                let b = i32::try_from(b).expect("width");
                println!(
                    "{} {} {} {} {} {} {} {}",
                    model::invert32(a, b),
                    model::and32(a, b),
                    model::or32(a, b),
                    model::xor32(a, b),
                    model::nested32(a, b),
                    model::field32(a, b),
                    model::shared32(a, b),
                    model::imported32(a, b)
                );
            }
            "64" => {
                println!(
                    "{} {} {} {} {} {} {} {}",
                    model::invert64(a, b),
                    model::and64(a, b),
                    model::or64(a, b),
                    model::xor64(a, b),
                    model::nested64(a, b),
                    model::field64(a, b),
                    model::shared64(a, b),
                    model::imported64(a, b)
                );
            }
            _ => panic!("width"),
        }
    }
}
