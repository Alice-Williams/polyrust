//! Test-only compiling handoff mutations. Not included in normal probe binaries.
use portable_codegen::{
    RustCanonicalInstanceFacts as Facts, RustCanonicalInstanceKey as Key,
    RustResultVariantFacts as Variant,
};

pub(super) fn facts(value: Facts) -> Facts {
    let mut ids = [
        value.core_root(),
        value.key().result_definition(),
        value.key().error_definition(),
        value.ok().variant,
        value.err().variant,
        value.ok().payload,
        value.err().payload,
    ];
    let role = std::env::var("POLYRUST_INSTANCE_FAULT").expect("test must select a role");
    let index = match role.as_str() {
        "CoreRoot" => 0,
        "Result" => 1,
        "Error" => 2,
        "Ok" => 3,
        "Err" => 4,
        "OkPayload" => 5,
        "ErrPayload" => 6,
        _ => panic!("unsupported test fault"),
    };
    // A fresh local hash keeps the description internally consistent. The
    // compiler oracle must reject the wrong binding, not just a duplicate ID.
    let crate_id = ids[index].crate_id;
    let replacement = (0..=7)
        .map(|definition_path_hash| portable_codegen::RustDeclarationId {
            crate_id,
            definition_path_hash,
        })
        .find(|candidate| !ids.contains(candidate))
        .unwrap();
    ids[index] = replacement;
    Facts::new(
        Key::i32_try_from_int_error_result(ids[1], ids[2]).unwrap(),
        ids[0],
        Variant {
            variant: ids[3],
            payload: ids[5],
        },
        Variant {
            variant: ids[4],
            payload: ids[6],
        },
    )
    .unwrap()
}
