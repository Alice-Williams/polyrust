use public_package_model as api;
use std::io::{self, BufRead};

fn main() {
    for line in io::stdin().lock().lines() {
        let value: i32 = line.unwrap().parse().unwrap();
        println!(
            "{} {} {} {} {} {} {} {}",
            api::zero(),
            api::exported(value),
            api::alias(value),
            api::api::invoke(value),
            u8::from(api::positive(value)),
            api::choose(value, true, -1, false),
            api::choose(value, false, -1, true),
            api::choose(value, false, -1, false)
        );
    }
}
