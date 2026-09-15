use boolean_negation_model as model;
use std::io::{self, BufRead};

fn main() {
    for line in io::stdin().lock().lines() {
        let value: i32 = line.unwrap().parse().unwrap();
        let flag = value != 0;
        let results = [
            i32::from(model::invert(false)),
            i32::from(model::invert(true)),
            i32::from(model::literal_true()),
            i32::from(model::literal_false()),
            i32::from(model::nested(flag)),
            i32::from(model::comparison(value)),
            i32::from(model::call(value)),
            i32::from(model::field(flag)),
            i32::from(model::shared(flag)),
            i32::from(model::shadow(flag)),
            model::conditional(value),
            i32::from(model::nested_call(value)),
            i32::from(model::argument(value)),
        ];
        println!("{}", results.map(|value| value.to_string()).join(" "));
    }
}
