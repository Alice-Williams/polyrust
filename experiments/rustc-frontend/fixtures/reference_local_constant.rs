//! Independent Rust execution of every exported fixture function.
use local_constant_values_model as model;
fn main() {
    for flag in [false, true] {
        println!("{}", model::simple(flag));
        println!("{}", model::min32(flag));
        println!("{}", model::max32(flag));
        println!("{}", model::min64(flag));
        println!("{}", model::max64(flag));
        println!("{}", model::computed(flag));
        println!("{}", i32::from(model::bool_true(flag)));
        println!("{}", i32::from(model::bool_false(flag)));
        println!("{}", model::forward(flag));
        println!("{}", model::unused(flag));
        println!("{}", model::shadow(flag));
        println!("{}", model::nested(flag));
        println!("{}", model::branch(flag));
        println!("{}", model::record(flag));
        println!("{}", model::imported(flag));
    }
}
