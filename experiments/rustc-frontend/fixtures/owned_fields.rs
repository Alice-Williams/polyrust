//! Straight-line partial-record ownership; target heap output remains disabled.
pub fn tail(value: i32) -> i32 {
    let owner = Box::new(value);
    *owner
}
pub struct Record {
    first: Box<i32>,
    second: Box<i32>,
    third: Box<i32>,
}
pub fn first(a: i32, b: i32, c: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let z = Box::new(c);
    let record = Record {
        first: x,
        second: y,
        third: z,
    };
    let taken = record.first;
    *taken
}
pub fn reversed(a: i32, b: i32, c: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let z = Box::new(c);
    let record = Record {
        third: z,
        first: x,
        second: y,
    };
    let taken = record.second;
    *taken
}
pub fn multiple(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let extra = Box::new(d);
    let x = Box::new(a);
    let moved = x;
    let y = Box::new(b);
    let z = Box::new(c);
    let record = Record {
        second: y,
        third: z,
        first: moved,
    };
    let first = record.first;
    let third = record.third;
    let final_owner = first;
    let _extra = extra;
    *final_owner
}
pub fn all(a: i32, b: i32, c: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let z = Box::new(c);
    let record = Record {
        first: x,
        second: y,
        third: z,
    };
    let first = record.first;
    let second = record.second;
    let third = record.third;
    return *third;
}

pub fn last(a: i32, b: i32, c: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(b);
    let z = Box::new(c);
    let record = Record {
        first: x,
        second: y,
        third: z,
    };
    let taken = record.third;
    *taken
}
pub fn shadow(a: i32, b: i32, c: i32) -> i32 {
    let a = Box::new(a);
    let b = Box::new(b);
    let c = Box::new(c);
    let a = a;
    let record = Record {
        third: c,
        second: b,
        first: a,
    };
    let record = record.second;
    let record = record;
    *record
}
pub struct Single {
    renamed: Box<i32>,
}
pub fn single(a: i32) -> i32 {
    let value = Box::new(a);
    let record = Single { renamed: value };
    let taken = record.renamed;
    *taken
}
pub mod left {
    pub struct Record {
        renamed: Box<i32>,
    }
    pub fn same(a: i32) -> i32 {
        let value = Box::new(a);
        let record = Record { renamed: value };
        let taken = record.renamed;
        *taken
    }
}
pub mod right {
    pub struct Record {
        renamed: Box<i32>,
    }
    pub fn same(a: i32) -> i32 {
        let value = Box::new(a);
        let record = Record { renamed: value };
        let taken = record.renamed;
        *taken
    }
}
pub fn repeated(a: i32) -> i32 {
    let x = Box::new(a);
    let y = Box::new(a);
    let z = Box::new(a);
    let record = Record {
        first: x,
        second: y,
        third: z,
    };
    let taken = record.first;
    *taken
}
pub fn inline(a: i32) -> i32 {
    let record = Single {
        renamed: Box::new(a),
    };
    let taken = record.renamed;
    *taken
}
pub fn after(a: i32, b: i32) -> i32 {
    let value = Box::new(a);
    let record = Single { renamed: value };
    let extra = Box::new(b);
    let taken = record.renamed;
    *taken
}
pub fn whole(a: i32) -> i32 {
    let value = Box::new(a);
    let record = Single { renamed: value };
    let moved = record;
    let taken = moved.renamed;
    *taken
}
pub fn remaining(a: i32) -> i32 {
    let value = Box::new(a);
    let record = Single { renamed: value };
    *record.renamed
}
pub fn nested_scope(a: i32) -> i32 {
    let value = Box::new(a);
    let record = Single { renamed: value };
    {
        let taken = record.renamed;
        *taken
    }
}
pub fn borrowed(a: i32) -> i32 {
    let value = Box::new(a);
    let record = Single { renamed: value };
    let taken = &record.renamed;
    **taken
}
pub fn conditional(a: i32, flag: bool) -> i32 {
    let value = Box::new(a);
    let record = if flag {
        Single { renamed: value }
    } else {
        Single { renamed: value }
    };
    let taken = record.renamed;
    *taken
}
pub fn updated(a: i32) -> i32 {
    let value = Box::new(a);
    let record = Single { renamed: value };
    let record = Single { ..record };
    let taken = record.renamed;
    *taken
}
pub fn assigned(a: i32) -> i32 {
    let value = Box::new(a);
    let record = Single { renamed: value };
    let taken;
    taken = record.renamed;
    *taken
}
pub struct Generic<T> {
    value: T,
}
pub fn generic(a: i32) -> i32 {
    let value = Box::new(a);
    let record = Generic { value };
    let taken = record.value;
    *taken
}
pub struct Custom {
    value: Box<i32>,
}
impl Drop for Custom {
    fn drop(&mut self) {}
}
pub fn custom(a: i32) -> i32 {
    let value = Box::new(a);
    let record = Custom { value };
    *record.value
}
pub struct Nested {
    value: Single,
}
pub fn nested_record(a: i32) -> i32 {
    let value = Box::new(a);
    let record = Nested {
        value: Single { renamed: value },
    };
    let taken = record.value.renamed;
    *taken
}
pub fn tuple(a: i32) -> i32 {
    let value = Box::new(a);
    let record = (value,);
    let taken = record.0;
    *taken
}
