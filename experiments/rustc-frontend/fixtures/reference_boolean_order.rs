mod boolean_order;

fn main() {
    use std::io::BufRead;
    for line in std::io::stdin().lock().lines() {
        let value: i32 = line.unwrap().parse().unwrap();
        println!("{}", boolean_order::score(value));
    }
}
