use eager_values_model as model;

fn main() {
    let functions = [
        model::and,
        model::or,
        model::xor,
        model::nested,
        model::lazy_left,
        model::lazy_right,
        model::lazy_outer,
        model::eager_outer,
        model::field,
        model::shared,
        model::imported,
        model::call_args,
        model::condition,
    ];
    for a in [false, true] {
        for b in [false, true] {
            for c in [false, true] {
                for function in functions {
                    println!("{}", u8::from(function(a, b, c)));
                }
            }
        }
    }
}
