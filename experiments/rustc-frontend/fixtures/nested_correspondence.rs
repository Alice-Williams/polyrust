//! Bounded nested correspondence fixtures, not target heap generation.
#![allow(dead_code, unused_variables)]

struct Inner {
    first: Box<i32>,
    second: Box<i32>,
}
struct Outer {
    nested: Inner,
    spare: Box<i32>,
}
mod alternate {
    struct Inner {
        first: Box<i32>,
        second: Box<i32>,
    }
    struct Outer {
        nested: Inner,
        spare: Box<i32>,
    }
    pub fn same(a: i32, b: i32, c: i32) -> i32 {
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
}
pub fn permuted(a: i32, b: i32, c: i32) -> i32 {
    let x = Box::new(c);
    let y = Box::new(a);
    let z = Box::new(b);
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

pub fn spare(a: i32, b: i32, c: i32) -> i32 {
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
    let taken = outer.spare;
    *taken
}

pub fn all_leaves(a: i32, b: i32, c: i32) -> i32 {
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
    let one = outer.nested.first;
    let taken = outer.nested.second;
    let three = outer.spare;
    *taken
}

pub fn reverse_extractions(a: i32, b: i32, c: i32) -> i32 {
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
    let other = outer.nested.first;
    *taken
}

pub fn whole_multiple(a: i32, b: i32, c: i32) -> i32 {
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
    let one = moved.first;
    let taken = moved.second;
    *taken
}

pub fn shadowed(a: i32, b: i32, c: i32) -> i32 {
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
    let inner = outer.nested;
    let taken = inner.first;
    let inner = taken;
    *inner
}

pub fn explicit_return(a: i32, b: i32, c: i32) -> i32 {
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
    return *taken;
}

pub fn whole_unopened(a: i32, b: i32, c: i32) -> i32 {
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
    let taken = outer.spare;
    *taken
}
