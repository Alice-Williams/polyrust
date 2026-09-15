//! Scalar constant reads through the compiler's evaluator, not source tokens.
const LOW32: i32 = i32::MIN;
const HIGH32: i32 = i32::MAX;
const LOW64: i64 = i64::MIN;
const HIGH64: i64 = i64::MAX;
const EXACT: i64 = (1_i64 << 53) + 1;
const NEGATIVE: i64 = -EXACT;
const COMPUTED: i32 = 7 * 9 - 1;
const FLAG: bool = COMPUTED > 0;
const DISABLED: bool = COMPUTED < 0;

mod left {
    pub(crate) const VALUE: i32 = 41;
}
mod right {
    pub(crate) const VALUE: i32 = 43;
}
use left::VALUE as RENAMED;

struct Limits;
impl Limits {
    const VALUE: i64 = 9_223_372_036_854_775_000;
}
struct Pair {
    value: i64,
    flag: bool,
}

pub fn min32(_flag: bool) -> i32 {
    LOW32
}
pub fn max32(_flag: bool) -> i32 {
    HIGH32
}
pub fn min64(_flag: bool) -> i64 {
    LOW64
}
pub fn max64(_flag: bool) -> i64 {
    HIGH64
}
pub fn exact(_flag: bool) -> i64 {
    EXACT
}
pub fn negative(_flag: bool) -> i64 {
    NEGATIVE
}
pub fn computed(_flag: bool) -> i32 {
    COMPUTED
}
pub fn primitive32(_flag: bool) -> i32 {
    i32::MIN
}
pub fn primitive64(_flag: bool) -> i64 {
    i64::MAX
}
pub fn associated(_flag: bool) -> i64 {
    Limits::VALUE
}
pub fn left(_flag: bool) -> i32 {
    left::VALUE
}
pub fn right(_flag: bool) -> i32 {
    right::VALUE
}
pub fn alias(_flag: bool) -> i32 {
    RENAMED
}
pub fn bool_true(_flag: bool) -> bool {
    FLAG
}
pub fn bool_false(_flag: bool) -> bool {
    DISABLED
}
pub fn branch(flag: bool) -> i64 {
    if flag { LOW64 } else { HIGH64 }
}
pub fn record(flag: bool) -> i64 {
    let pair = Pair {
        value: EXACT,
        flag: FLAG,
    };
    if flag == pair.flag {
        pair.value
    } else {
        NEGATIVE
    }
}
pub fn imported(flag: bool) -> i64 {
    constant_leaf::identity(EXACT, flag)
}
pub fn shared_local(_flag: bool) -> i64 {
    let value = EXACT;
    let borrowed = &value;
    *borrowed
}
