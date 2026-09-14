pub fn tail_value(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return *x;
    }
    *y
}
#[rustfmt::skip]
pub fn explicit_value(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a); let y = Box::new(b);
    if flag { return *x; }
    return *y
}
pub fn explicit_semi(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return *x;
    }
    return *y;
}
#[rustfmt::skip]
pub fn arm_value(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a); let y = Box::new(b);
    if flag { return *x }
    *y
}
pub fn swapped(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return *y;
    }
    *x
}
pub fn moved(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let first = x;
    let second = y;
    if flag {
        return *first;
    }
    *second
}
pub fn middle(a: i32, enabled: bool, b: i32) -> i32 {
    let left = Box::new(a);
    let right = Box::new(b);
    if enabled {
        return *left;
    }
    *right
}
pub fn unused(a: i32, _unused: i32, b: i32, flag: bool) -> i32 {
    let x = Box::new(b);
    let y = Box::new(a);
    if flag {
        return *x;
    }
    *y
}
pub fn intervening(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    if flag {
        return *x;
    }
    let y = Box::new(b);
    *y
}
pub fn branch_move(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        let moved = x;
        return *moved;
    }
    *y
}
pub fn nested_condition(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        if a > 0 {
            return *x;
        }
        return *y;
    }
    *y
}
pub fn with_else(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return *x;
    } else {
        return *y;
    }
}
pub fn value_call(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return (*x).abs();
    }
    *y
}
pub fn suffix(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return *x;
    }
    return *y;
    let _extra = Box::new(a);
}
pub fn continuation_block(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return *x;
    }
    { *y }
}
pub fn negated(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if !flag {
        return *x;
    }
    *y
}
pub fn tail(a: i32) -> i32 {
    let x = Box::new(a);
    *x
}
