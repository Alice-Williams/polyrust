pub fn score(input: i32) -> i32 {
    let outer = input;
    {
        let inner = outer;
        if inner > 10 { inner } else { 10 }
    }
}
