//! Native Rust truth; compared independently with exact integer expectations.
use constant_values_model as model;
fn main() {
    for flag in [false, true] {
        println!("{}", model::min32(flag));
        println!("{}", model::max32(flag));
        println!("{}", model::min64(flag));
        println!("{}", model::max64(flag));
        println!("{}", model::exact(flag));
        println!("{}", model::negative(flag));
        println!("{}", model::computed(flag));
        println!("{}", model::primitive32(flag));
        println!("{}", model::primitive64(flag));
        println!("{}", model::associated(flag));
        println!("{}", model::left(flag));
        println!("{}", model::right(flag));
        println!("{}", model::alias(flag));
        println!("{}", i32::from(model::bool_true(flag)));
        println!("{}", i32::from(model::bool_false(flag)));
        println!("{}", model::branch(flag));
        println!("{}", model::record(flag));
        println!("{}", model::imported(flag));
        println!("{}", model::shared_local(flag));
    }
}
