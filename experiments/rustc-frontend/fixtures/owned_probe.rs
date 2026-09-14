//! Compiler-only evidence fixtures; not an admitted target program.

pub fn straight(value: i32) -> i32 {
    let original = Box::new(value);
    let moved = original;
    *moved
}

pub fn conditional(value: i32, take: bool) -> i32 {
    let original = Box::new(value);
    let selected;
    if take {
        selected = original;
    } else {
        selected = Box::new(value);
    }
    *selected
}

struct Pair {
    first: Box<i32>,
    second: Box<i32>,
}

pub fn partial(value: i32) -> i32 {
    let both = Pair {
        first: Box::new(value),
        second: Box::new(value),
    };
    let taken = both.first;
    *taken + *both.second
}

pub fn early(value: i32, take: bool) -> i32 {
    let owned = Box::new(value);
    if take {
        return *owned;
    }
    *owned
}

pub fn shadow(value: i32) -> i32 {
    let value = Box::new(value);
    {
        let value = Box::new(*value);
        *value
    }
}

mod imitation {
    struct Box(i32);

    pub fn fake(value: i32) -> i32 {
        Box(value).0
    }
}

pub fn counterfeit(value: i32) -> i32 {
    imitation::fake(value)
}
