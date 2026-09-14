//! Same types do not establish allocation identity; parameter producers do.
pub fn one(a: i32) -> i32 {
    let x = Box::new(a);
    *x
}
pub fn two_first(a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let _y = Box::new(b);
    *x
}
pub fn two_second(a: i32, b: i32) -> i32 {
    let _x = Box::new(a);
    let y = Box::new(b);
    *y
}
pub fn interleaved(a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let x1 = x;
    let y1 = y;
    let _x2 = x1;
    *y1
}
pub fn nested(a: i32, b: i32) -> i32 {
    let outer = Box::new(a);
    {
        let inner = Box::new(b);
        {
            let _moved = outer;
            *inner
        }
    }
}
pub fn shadow(a: i32, b: i32) -> i32 {
    let _value = Box::new(a);
    {
        let _value = Box::new(b);
        *_value
    }
}
pub fn same_scope_shadow(a: i32, b: i32) -> i32 {
    let _value = Box::new(a);
    let _value = Box::new(b);
    *_value
}
pub fn unused_parameter(a: i32, _b: i32, c: i32) -> i32 {
    let _z = Box::new(c);
    let x = Box::new(a);
    *x
}
pub fn outer_drop(a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let _y = Box::new(b);
    { { *x } }
}
pub fn repeated_anchor(a: i32) -> i32 {
    let _x = Box::new(a);
    let y = Box::new(a);
    *y
}
pub fn bad_literal(_a: i32) -> i32 {
    let x = Box::new(42);
    *x
}
pub fn bad_call(a: i32) -> i32 {
    let x = Box::new(a.abs());
    *x
}
pub fn bad_scalar(a: i32) -> i32 {
    let b = a;
    let x = Box::new(b);
    *x
}
pub fn bad_branch(a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    if a > b { *x } else { *y }
}
