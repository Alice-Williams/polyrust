pub fn choose(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return *x;
    } else {
        return *y;
    }
}
pub fn reversed(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return *y;
    } else {
        return *x;
    }
}
pub fn moved(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let x1 = x;
    let y1 = y;
    if flag {
        return *x1;
    } else {
        return *y1;
    }
}
pub fn middle(a: i32, flag: bool, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return *x;
    } else {
        return *y;
    }
}
pub fn renamed(other: bool, left: i32, right: i32) -> i32 {
    let first = Box::new(left);
    let second = Box::new(right);
    if other {
        return *first;
    } else {
        return *second;
    }
}
#[rustfmt::skip]
pub fn no_semicolons(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag { return *x } else { return *y }
}
pub fn unused(a: i32, _unused: i32, b: i32, flag: bool) -> i32 {
    let x = Box::new(b);
    let y = Box::new(a);
    if flag {
        return *x;
    } else {
        return *y;
    }
}
pub fn negated(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if !flag {
        return *x;
    } else {
        return *y;
    }
}
pub fn alias(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let copied = flag;
    if copied {
        return *x;
    } else {
        return *y;
    }
}
pub fn branch_move(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        let moved = x;
        return *moved;
    } else {
        return *y;
    }
}
pub fn branch_constructor(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    if flag {
        let y = Box::new(b);
        return *y;
    } else {
        return *x;
    }
}
pub fn implicit(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag { *x } else { *y }
}
pub fn early(flag: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return *x;
    }
    *y
}
pub fn two_flags(flag: bool, _other: bool, a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if flag {
        return *x;
    } else {
        return *y;
    }
}
pub fn computed(a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if a > b {
        return *x;
    } else {
        return *y;
    }
}
pub fn tail(a: i32) -> i32 {
    let x = Box::new(a);
    *x
}
