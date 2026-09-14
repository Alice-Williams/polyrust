use std::io::{self, BufRead};

fn main() {
    // The differential corpus includes 0: reversing nested dependency calls
    // must change its observable result, not merely its call tree.
    assert_ne!(left::score(right::score(0)), right::score(left::score(0)));
    assert_ne!(root::score(0), root::score(1));
    assert_ne!(root::choose(true, 2, 0), root::choose(false, 2, 0));
    for line in io::stdin().lock().lines() {
        let value: i32 = line.unwrap().parse().unwrap();
        println!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            leaf::score(value),
            leaf::alias(value),
            left::score(value),
            right::score(value),
            root::score(value),
            root::alias(value),
            root::alternate(value),
            u8::from(leaf::flip(false)),
            u8::from(leaf::flip(true)),
            u8::from(right::flip(false)),
            u8::from(right::flip(true)),
            u8::from(root::flip(false)),
            u8::from(root::flip(true)),
            root::zero(),
            root::choose(true, value, 0),
            root::choose(false, value, 0)
        );
    }
}
