#![allow(dead_code, unused_variables)]

struct Inner {
    first: Box<i32>,
    second: Box<i32>,
}
struct Outer {
    nested: Inner,
    spare: Box<i32>,
}
pub fn first(a: i32, b: i32, c: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let z = Box::new(c);
    let inner = Inner {
        first: x,
        second: y,
    };
    let outer = Outer {
        nested: inner,
        spare: z,
    };
    let taken = outer.nested.first;
    *taken
}
pub fn second(a: i32, b: i32, c: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let z = Box::new(c);
    let inner = Inner {
        first: x,
        second: y,
    };
    let outer = Outer {
        nested: inner,
        spare: z,
    };
    let taken = outer.nested.second;
    *taken
}
pub fn reversed(a: i32, b: i32, c: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let z = Box::new(c);
    let inner = Inner {
        second: y,
        first: x,
    };
    let outer = Outer {
        spare: z,
        nested: inner,
    };
    let taken = outer.nested.first;
    *taken
}
pub fn multiple(a: i32, b: i32, c: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let z = Box::new(c);
    let inner = Inner {
        first: x,
        second: y,
    };
    let outer = Outer {
        nested: inner,
        spare: z,
    };
    let first = outer.nested.first;
    let second = outer.nested.second;
    *first
}
pub fn whole_inner(a: i32, b: i32, c: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let z = Box::new(c);
    let inner = Inner {
        first: x,
        second: y,
    };
    let outer = Outer {
        nested: inner,
        spare: z,
    };
    let moved = outer.nested;
    let taken = moved.first;
    *taken
}
