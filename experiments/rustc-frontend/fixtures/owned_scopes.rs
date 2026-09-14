//! Ownership scope differs from the read scope in empty tail wrappers.
pub fn root(value: i32) -> i32 {
    let owner = Box::new(value);
    *owner
}
pub fn inward(value: i32) -> i32 {
    let outer = Box::new(value);
    {
        let inner = outer;
        *inner
    }
}
pub fn read_wrapper(value: i32) -> i32 {
    let owner = Box::new(value);
    { *owner }
}
pub fn inner_construction(value: i32) -> i32 {
    {
        let owner = Box::new(value);
        *owner
    }
}
pub fn many(value: i32) -> i32 {
    let outer = Box::new(value);
    {
        let middle = outer;
        {
            let inner = middle;
            { *inner }
        }
    }
}
pub fn shadow(value: i32) -> i32 {
    let value = Box::new(value);
    {
        let value = value;
        { *value }
    }
}
pub fn empty_wrappers(value: i32) -> i32 {
    {
        {
            let owner = Box::new(value);
            { *owner }
        }
    }
}
pub fn bad_sibling(value: i32) -> i32 {
    let owner = Box::new(value);
    {
        let _borrow = &owner;
    }
    *owner
}
pub fn bad_escape(value: i32) -> i32 {
    let owner = { Box::new(value) };
    *owner
}
pub fn bad_branch(value: i32) -> i32 {
    let owner = Box::new(value);
    { if value > 0 { *owner } else { *owner } }
}
pub fn bad_early(value: i32) -> i32 {
    let owner = Box::new(value);
    {
        return *owner;
    }
}
pub fn bad_multiple(value: i32) -> i32 {
    let _first = Box::new(value);
    {
        let second = Box::new(value);
        *second
    }
}
