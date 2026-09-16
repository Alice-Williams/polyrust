use public_constant_values_model as model;
fn main() {
    println!("{}", i32::from(model::FALSE));
    println!("{}", i32::from(model::TRUE));
    println!("{}", model::I32_MIN);
    println!("{}", model::I32_MAX);
    println!("{}", model::I64_MIN);
    println!("{}", model::I64_MAX);
    println!("{}", model::WIDE);
    println!("{}", model::NEGATIVE_WIDE);
    println!("{}", model::COMPUTED);
    println!("{}", model::FORWARD);
    println!("{}", model::WIDE_ALIAS);
    println!("{}", i32::from(model::read_false()));
    println!("{}", i32::from(model::read_true()));
    println!("{}", model::read_i32_min());
    println!("{}", model::read_i32_max());
    println!("{}", model::read_i64_min());
    println!("{}", model::read_i64_max());
    println!("{}", model::read_wide());
    println!("{}", model::read_negative_wide());
    println!("{}", model::read_computed());
    println!("{}", model::read_forward());
    println!("{}", model::read_private());
}
