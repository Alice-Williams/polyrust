use i64_values_model as model;

fn main() {
    let values = [
        i64::MIN,
        i64::MIN + 1,
        -9_007_199_254_740_993,
        -9_007_199_254_740_992,
        -4_294_967_296,
        -2_147_483_649,
        -2_147_483_648,
        -2_147_483_647,
        -1,
        0,
        1,
        2_147_483_646,
        2_147_483_647,
        2_147_483_648,
        4_294_967_295,
        4_294_967_296,
        9_007_199_254_740_991,
        9_007_199_254_740_992,
        9_007_199_254_740_993,
        i64::MAX - 1,
        i64::MAX,
    ];
    for a in values {
        for b in values {
            for flag in [false, true] {
                for function in [
                    model::identity,
                    model::choose,
                    model::minimum,
                    model::maximum,
                    model::positive,
                    model::negative,
                    model::local,
                    model::record,
                    model::shared,
                    model::shared_record,
                    model::imported,
                    model::mixed,
                ] {
                    println!("{}", function(a, b, flag));
                }
                for function in [
                    model::equal,
                    model::not_equal,
                    model::less,
                    model::less_equal,
                    model::greater,
                    model::greater_equal,
                    model::lazy,
                ] {
                    println!("{}", u8::from(function(a, b, flag)));
                }
                println!("{}", model::exported_identity(a, b, flag));
            }
        }
    }
}
