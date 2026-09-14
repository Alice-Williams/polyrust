#[rustfmt::skip]
pub fn root_value(a: i32) -> i32 {
    let x = Box::new(a);
    return *x
}
pub fn root_semi(a: i32) -> i32 {
    let x = Box::new(a);
    return *x;
}
#[rustfmt::skip]
pub fn nested_value(a: i32) -> i32 {
    let x = Box::new(a);
    { return *x }
}
pub fn nested_semi(a: i32) -> i32 {
    let x = Box::new(a);
    {
        return *x;
    }
}
pub fn moved(a: i32) -> i32 {
    let x = Box::new(a);
    {
        let y = x;
        {
            return *y;
        }
    }
}
pub fn outer(a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    {
        let _y = Box::new(b);
        {
            return *x;
        }
    }
}
#[rustfmt::skip]
pub fn inner(a: i32, b: i32) -> i32 {
    let _x = Box::new(a);
    { let y = Box::new(b); return *y }
}
pub fn shadow(a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    {
        let x = Box::new(b);
        return *x;
    }
}
pub fn interleaved(a: i32, b: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let x1 = x;
    let y1 = y;
    {
        let _x2 = x1;
        {
            return *y1;
        }
    }
}
pub fn tail(a: i32) -> i32 {
    let x = Box::new(a);
    *x
}
pub fn conditional(a: i32) -> i32 {
    let x = Box::new(a);
    if a > 0 {
        return *x;
    }
    *x
}
pub fn suffix(a: i32) -> i32 {
    let x = Box::new(a);
    return *x;
    let _y = Box::new(a);
}
pub fn scalar(a: i32) -> i32 {
    let _x = Box::new(a);
    return a;
}
pub fn return_block(a: i32) -> i32 {
    let x = Box::new(a);
    return { *x };
}
