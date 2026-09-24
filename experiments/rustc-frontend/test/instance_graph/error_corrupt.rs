//! Compiling test-only mutations keep metadata consistent but original IDs wrong.
use portable_codegen::{
    RustCanonicalErrorKindFacts as Facts, RustDeclarationId as Id, RustIntegerErrorKind as Kind,
    RustIntegerErrorVariants as Variants,
};

pub(super) fn facts(original: Facts) -> Facts {
    let ids = [
        original.wrapper_field(),
        original.kind_definition(),
        original.variant(Kind::Empty),
        original.variant(Kind::InvalidDigit),
        original.variant(Kind::PosOverflow),
        original.variant(Kind::NegOverflow),
        original.variant(Kind::Zero),
        original.variant(Kind::NotAPowerOfTwo),
    ];
    let role = std::env::var("POLYRUST_ERROR_STATE_FAULT").expect("test selects role");
    let index = match role.as_str() {
        "WrapperField" => 0,
        "Kind" => 1,
        "Empty" => 2,
        "InvalidDigit" => 3,
        "PosOverflow" => 4,
        "NegOverflow" => 5,
        "Zero" => 6,
        "NotAPowerOfTwo" => 7,
        _ => panic!("unsupported test fault"),
    };
    for definition_path_hash in 0..=15 {
        let mut changed = ids;
        changed[index] = Id {
            crate_id: ids[index].crate_id,
            definition_path_hash,
        };
        let replacement = Facts::new(
            original.instance(),
            changed[0],
            changed[1],
            Variants {
                empty: changed[2],
                invalid_digit: changed[3],
                positive_overflow: changed[4],
                negative_overflow: changed[5],
                zero: changed[6],
                not_a_power_of_two: changed[7],
            },
        );
        if let Ok(replacement) = replacement
            && replacement != original
        {
            return replacement;
        }
    }
    unreachable!("sixteen candidates exceed fifteen reserved roles")
}
