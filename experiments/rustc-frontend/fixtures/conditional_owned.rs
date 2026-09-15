//! M-01 observation preparation; no conditional record certificate is enabled.
#![allow(dead_code, unused_variables, unused_assignments)]

struct Pair {
    first: Box<i32>,
    second: Box<i32>,
}

pub fn initialize(flag: bool, a: i32, b: i32, c: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let keep = Box::new(c);
    let record;
    if flag {
        record = Pair { first, second };
    }
    *keep
}

pub fn initialize_reversed(flag: bool, a: i32, b: i32, c: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let keep = Box::new(c);
    let record;
    if flag {
        record = Pair { second, first };
    }
    *keep
}

pub fn partial_first(flag: bool, a: i32, b: i32, c: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let keep = Box::new(c);
    let record = Pair { first, second };
    if flag {
        let taken = record.first;
    }
    *keep
}

pub fn partial_second(flag: bool, a: i32, b: i32, c: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let keep = Box::new(c);
    let record = Pair { first, second };
    if flag {
        let taken = record.second;
    }
    *keep
}

pub fn early_first(flag: bool, a: i32, b: i32, c: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let keep = Box::new(c);
    let record = Pair { first, second };
    if flag {
        let taken = record.first;
        return *taken;
    }
    let taken = record.second;
    *taken
}

pub fn early_second(flag: bool, a: i32, b: i32, c: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let keep = Box::new(c);
    let record = Pair { first, second };
    if flag {
        let taken = record.second;
        return *taken;
    }
    let taken = record.first;
    *taken
}
