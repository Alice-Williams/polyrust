use short_circuit_model as model;

fn main() {
    for a in [false, true] {
        for b in [false, true] {
            for c in [false, true] {
                for function in [
                    model::and,
                    model::or,
                    model::nested,
                    model::left_nested,
                    model::negated,
                    model::argument,
                    model::later_argument,
                    model::local,
                    model::shadow,
                    model::field,
                    model::shared,
                    model::comparison,
                    model::conditional,
                    model::constants,
                ] {
                    println!("{}", u8::from(function(a, b, c)));
                }
            }
        }
    }
}
