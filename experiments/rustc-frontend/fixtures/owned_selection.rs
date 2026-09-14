pub fn select(flag: bool, a: i32, b: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let selected = if flag { first } else { second };
    *selected
}

pub fn tail(value: i32) -> i32 {
    let owned = Box::new(value);
    *owned
}

pub fn moved(flag: bool, a: i32, b: i32, c: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let _unused = Box::new(c);
    let first = x;
    let second = y;
    let selected = if flag { second } else { first };
    return *selected;
}

pub fn reordered(a: i32, b: i32, flag: bool) -> i32 {
    let first = Box::new(b);
    let second = Box::new(a);
    let selected = if flag { first } else { second };
    *selected
}
#[rustfmt::skip]
pub fn explicit(flag: bool, a: i32, b: i32) -> i32 {
    let first = Box::new(a); let second = Box::new(b);
    let selected = if flag { first } else { second };
    return *selected
}
pub fn middle(a: i32, take: bool, b: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let selected = if take { second } else { first };
    *selected
}
pub fn extra_before(flag: bool, a: i32, b: i32, c: i32) -> i32 {
    let _other = Box::new(c);
    let first = Box::new(a);
    let second = Box::new(b);
    let selected = if flag { first } else { second };
    *selected
}
pub fn extra_after(flag: bool, a: i32, b: i32, c: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let _other = Box::new(c);
    let selected = if flag { first } else { second };
    *selected
}
pub fn shadow(flag: bool, a: i32, b: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let first = first;
    let second = second;
    let selected = if flag { first } else { second };
    *selected
}

pub fn same(flag: bool, a: i32) -> i32 {
    let first = Box::new(a);
    let selected = if flag { first } else { first };
    *selected
}
pub fn branch_statement(flag: bool, a: i32, b: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let selected = if flag {
        let moved = first;
        moved
    } else {
        second
    };
    *selected
}
pub fn allocation(flag: bool, a: i32, b: i32) -> i32 {
    let first = Box::new(a);
    let selected = if flag { first } else { Box::new(b) };
    *selected
}
pub fn after_move(flag: bool, a: i32, b: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let selected = if flag { first } else { second };
    let last = selected;
    *last
}
pub fn negated(flag: bool, a: i32, b: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let selected = if !flag { first } else { second };
    *selected
}
pub fn nested_exit(flag: bool, a: i32, b: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let selected = if flag { first } else { second };
    { *selected }
}
pub fn wrong_read(flag: bool, a: i32, b: i32, c: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let other = Box::new(c);
    let _selected = if flag { first } else { second };
    *other
}
pub fn assignment(flag: bool, a: i32, b: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let selected;
    if flag {
        selected = first;
    } else {
        selected = second;
    }
    *selected
}
pub fn two_bools(flag: bool, _other: bool, a: i32, b: i32) -> i32 {
    let first = Box::new(a);
    let second = Box::new(b);
    let selected = if flag { first } else { second };
    *selected
}
pub fn constant(flag: bool, b: i32) -> i32 {
    let first = Box::new(1);
    let second = Box::new(b);
    let selected = if flag { first } else { second };
    *selected
}
