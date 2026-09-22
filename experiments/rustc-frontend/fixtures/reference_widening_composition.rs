//! Native observations of the same original composition fixture.
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let value = line.unwrap().parse().unwrap();
        for result in [
            widening_composition::direct(value),
            widening_composition::literal(value),
            widening_composition::nested(value),
            widening_composition::borrowed(value),
            widening_composition::record(value),
            widening_composition::composed(value),
            widening_composition::grouped(value),
        ] {
            println!("{result}");
        }
    }
}
