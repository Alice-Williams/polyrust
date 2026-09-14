mod scopes;
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let input = line.unwrap().parse::<i32>().unwrap();
        println!("{}", scopes::score(input));
    }
}
