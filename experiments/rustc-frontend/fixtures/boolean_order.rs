//! Every Boolean pair for each of the six comparison mappings.
fn compare_eq(left: bool, right: bool) -> bool {
    left == right
}
fn compare_ne(left: bool, right: bool) -> bool {
    left != right
}
#[expect(
    clippy::bool_comparison,
    reason = "exercise native Boolean ordering mappings"
)]
fn compare_lt(left: bool, right: bool) -> bool {
    left < right
}
fn compare_le(left: bool, right: bool) -> bool {
    left <= right
}
#[expect(
    clippy::bool_comparison,
    reason = "exercise native Boolean ordering mappings"
)]
fn compare_gt(left: bool, right: bool) -> bool {
    left > right
}
fn compare_ge(left: bool, right: bool) -> bool {
    left >= right
}
pub fn score(input: i32) -> i32 {
    if input == 0 {
        if compare_eq(false, false) { 1 } else { -1 }
    } else if input == 1 {
        if compare_eq(false, true) { 2 } else { -2 }
    } else if input == 2 {
        if compare_eq(true, false) { 3 } else { -3 }
    } else if input == 3 {
        if compare_eq(true, true) { 4 } else { -4 }
    } else if input == 4 {
        if compare_ne(false, false) { 5 } else { -5 }
    } else if input == 5 {
        if compare_ne(false, true) { 6 } else { -6 }
    } else if input == 6 {
        if compare_ne(true, false) { 7 } else { -7 }
    } else if input == 7 {
        if compare_ne(true, true) { 8 } else { -8 }
    } else if input == 8 {
        if compare_lt(false, false) { 9 } else { -9 }
    } else if input == 9 {
        if compare_lt(false, true) { 10 } else { -10 }
    } else if input == 10 {
        if compare_lt(true, false) { 11 } else { -11 }
    } else if input == 11 {
        if compare_lt(true, true) { 12 } else { -12 }
    } else if input == 12 {
        if compare_le(false, false) { 13 } else { -13 }
    } else if input == 13 {
        if compare_le(false, true) { 14 } else { -14 }
    } else if input == 14 {
        if compare_le(true, false) { 15 } else { -15 }
    } else if input == 15 {
        if compare_le(true, true) { 16 } else { -16 }
    } else if input == 16 {
        if compare_gt(false, false) { 17 } else { -17 }
    } else if input == 17 {
        if compare_gt(false, true) { 18 } else { -18 }
    } else if input == 18 {
        if compare_gt(true, false) { 19 } else { -19 }
    } else if input == 19 {
        if compare_gt(true, true) { 20 } else { -20 }
    } else if input == 20 {
        if compare_ge(false, false) { 21 } else { -21 }
    } else if input == 21 {
        if compare_ge(false, true) { 22 } else { -22 }
    } else if input == 22 {
        if compare_ge(true, false) { 23 } else { -23 }
    } else if input == 23 {
        if compare_ge(true, true) { 24 } else { -24 }
    } else {
        0
    }
}
