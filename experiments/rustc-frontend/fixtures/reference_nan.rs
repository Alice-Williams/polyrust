use nan_root_model as model;
use std::io::{self, BufRead};
fn main() {
    println!("{}", u8::from(model::positive_zero()));
    println!("{}", u8::from(model::negative_zero()));
    for line in io::stdin().lock().lines() {
        let bits = u64::from_str_radix(&line.unwrap(), 16).unwrap();
        let value = f64::from_bits(bits);
        for result in [
            model::direct(value),
            model::associated(value),
            model::local(value),
            model::imported(value),
            model::forwarded(value),
            model::shared(value),
            model::record(value),
            model::composed(value),
        ] {
            println!("{}", u8::from(result));
        }
    }
}
