//! Handwritten bit-observing consumer, not translated library code.
use floating_root_model as root;
use std::io::{self, BufRead};
fn observe(value: f64) {
    if value.is_nan() {
        println!("nan");
    } else {
        println!("{:016x}", value.to_bits());
    }
}
fn main() {
    observe(root::negative_zero());
    observe(root::restored_zero());
    for line in io::stdin().lock().lines() {
        let value = f64::from_bits(u64::from_str_radix(&line.unwrap(), 16).unwrap());
        for result in [
            root::direct(value),
            root::nested(value),
            root::local(value),
            root::imported(value),
            root::restored(value),
            root::shared(value),
            root::record(value),
            root::composed(value),
        ] {
            observe(result);
        }
    }
}
