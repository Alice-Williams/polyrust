fn main() {
    for value in [i32::MIN, -1, 0, 1, i32::MAX] {
        for flag in [false, true] {
            println!("{}", unit_root::execute(value, flag));
            unit_root::tail(value);
            unit_root::branch(flag);
        }
    }
}
