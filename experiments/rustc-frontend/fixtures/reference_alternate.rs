#[path = "alternate.rs"]
mod model;

fn main() {
    use std::io::BufRead;
    for line in std::io::stdin().lock().lines() {
        let value: i32 = line.unwrap().parse().unwrap();
        println!("{}", model::score(value));
    }
}
