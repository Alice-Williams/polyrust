fn main() {
    println!("{}", i32::from(root::read_false()));
    println!("{}", i32::from(root::read_true()));
    println!("{}", root::read_i32_min());
    println!("{}", root::read_i32_max());
    println!("{}", root::read_i64_min());
    println!("{}", root::read_i64_max());
    println!("{}", root::read_wide());
    println!("{}", root::read_negative_wide());
    println!("{}", root::read_computed());
    println!("{}", root::read_forward());
    println!("{}", root::read_private());
    println!("{}", root::read_local());
    println!("{}", root::read_left());
    println!("{}", root::read_right());
}
