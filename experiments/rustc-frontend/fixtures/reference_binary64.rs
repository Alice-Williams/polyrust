//! Handwritten oracle consumer; bit conversion is not generated library code.
use binary64_leaf as leaf;
use binary64_root_model as root;
use std::io::{self, BufRead};

fn observe(value: f64) {
    if value.is_nan() {
        println!("nan");
    } else {
        println!("{:016x}", value.to_bits());
    }
}
fn main() {
    for value in [
        leaf::positive_zero(),
        leaf::negative_zero(),
        leaf::minimum_subnormal(),
        leaf::maximum_subnormal(),
        leaf::minimum_normal(),
        leaf::maximum_finite(),
        leaf::negative_maximum(),
        leaf::next_above_one(),
        leaf::rounded_integer(),
        leaf::decimal_tenth(),
        leaf::negative_underflow(),
        root::tenth(),
    ] {
        observe(value);
    }
    let values: Vec<f64> = io::stdin()
        .lock()
        .lines()
        .map(|line| f64::from_bits(u64::from_str_radix(&line.unwrap(), 16).unwrap()))
        .collect();
    for &a in &values {
        for value in [
            root::identity(a),
            root::local(a),
            root::imported(a),
            root::shared(a),
            root::record(a),
            root::shared_record(a),
        ] {
            observe(value);
        }
    }
    for &a in &values {
        for &b in &values {
            for value in [
                root::equal(a, b),
                root::not_equal(a, b),
                root::less(a, b),
                root::less_equal(a, b),
                root::greater(a, b),
                root::greater_equal(a, b),
            ] {
                println!("{}", u8::from(value));
            }
        }
    }
}
