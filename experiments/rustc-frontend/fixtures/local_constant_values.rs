//! Block-local compiler constants have no runtime storage.
struct Pair {
    value: i64,
    flag: bool,
}

pub fn simple(_flag: bool) -> i32 {
    const VALUE: i32 = 4;
    VALUE
}

pub fn min32(_flag: bool) -> i32 {
    const VALUE: i32 = i32::MIN;
    VALUE
}

pub fn max32(_flag: bool) -> i32 {
    const VALUE: i32 = i32::MAX;
    VALUE
}

pub fn min64(_flag: bool) -> i64 {
    const VALUE: i64 = i64::MIN;
    VALUE
}

pub fn max64(_flag: bool) -> i64 {
    const VALUE: i64 = i64::MAX;
    VALUE
}

pub fn computed(_flag: bool) -> i32 {
    const VALUE: i32 = 7 * 9 - 1;
    VALUE
}

pub fn bool_true(_flag: bool) -> bool {
    const VALUE: bool = 7 > 0;
    VALUE
}

pub fn bool_false(_flag: bool) -> bool {
    const VALUE: bool = 7 < 0;
    VALUE
}

pub fn forward(_flag: bool) -> i64 {
    let value = VALUE;
    const VALUE: i64 = (1_i64 << 53) + 1;
    value
}

pub fn unused(_flag: bool) -> i32 {
    #[allow(dead_code)]
    const UNUSED: i64 = i64::MIN;
    9
}

pub fn shadow(flag: bool) -> i32 {
    const VALUE: i32 = 47;
    let original = VALUE;
    if flag {
        const VALUE: i32 = 41;
        VALUE
    } else {
        const VALUE: i32 = 43;
        let _saved = original;
        VALUE
    }
}

pub fn nested(_flag: bool) -> i32 {
    const VALUE: i32 = 19;
    let saved = VALUE;
    {
        const VALUE: i32 = 17;
        let _previous = saved;
        VALUE
    }
}

pub fn branch(flag: bool) -> i64 {
    if flag {
        const VALUE: i64 = i64::MIN;
        VALUE
    } else {
        const VALUE: i64 = i64::MAX;
        VALUE
    }
}

pub fn record(flag: bool) -> i64 {
    const VALUE: i64 = (1_i64 << 53) + 1;
    const NEGATIVE: i64 = -VALUE;
    let pair = Pair { value: VALUE, flag };
    if pair.flag { pair.value } else { NEGATIVE }
}

pub fn imported(flag: bool) -> i64 {
    const VALUE: i64 = (1_i64 << 53) + 1;
    local_constant_leaf::identity(VALUE, flag)
}
