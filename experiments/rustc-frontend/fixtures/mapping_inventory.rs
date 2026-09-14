pub struct Ticket {
    value: i32,
    flag: bool,
}

pub fn score(input: i32) -> i32 {
    let original = Ticket {
        flag: input == 0,
        value: input,
    };
    let moved = original;
    let one = &moved;
    let two = &one;
    #[expect(
        clippy::explicit_auto_deref,
        reason = "exercise explicit HIR dereferences separately from the implicit field adjustment below"
    )]
    let local = (*(*two)).value;
    let expected = true;
    let impossible = false;
    if two.flag == expected {
        -2147483648
    } else if two.flag != impossible {
        1
    } else if local < -10 {
        -7
    } else if local <= 0 {
        0
    } else if local > 10 {
        10
    } else if local >= 1 {
        {
            let shadow = local;
            if shadow == 1 { 1 } else { shadow }
        }
    } else {
        1
    }
}
