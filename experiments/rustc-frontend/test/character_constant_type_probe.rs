//! Test-only original constant facts corruption before/after authentication.
use portable_codegen::{RustConstantValue, RustSourceTypes};

pub(crate) fn change(original: RustSourceTypes, after: bool) -> RustSourceTypes {
    let Ok(fault) = std::env::var("POLYRUST_CHARACTER_CONSTANT_FAULT") else {
        return original;
    };
    // Select the constant-only producer, not equal-valued downstream owners.
    if original.constants().len() != 10 || fault.starts_with("target_") != after {
        return original;
    }
    let mut constants = original.constants().clone();
    let id = *constants
        .iter()
        .find(|(_, value)| **value == RustConstantValue::Char('A'))
        .unwrap()
        .0;
    let mut root = original.root();
    match fault.as_str() {
        "kind" | "target_kind" => {
            constants.insert(id, RustConstantValue::I32(65));
        }
        "value" | "target_value" => {
            constants.insert(id, RustConstantValue::Char('B'));
        }
        "declaration" => {
            let value = constants.remove(&id).unwrap();
            let mut changed = id;
            changed.definition_path_hash = u64::MAX;
            assert!(constants.insert(changed, value).is_none());
        }
        "owner" => root.definition_path_hash ^= 1,
        "missing" => {
            constants.remove(&id);
        }
        "extra" => {
            let mut changed = id;
            changed.definition_path_hash = u64::MAX;
            assert!(
                constants
                    .insert(changed, RustConstantValue::Char('A'))
                    .is_none()
            );
        }
        _ => panic!("unknown constant source fault"),
    }
    let result = RustSourceTypes::new(
        root,
        original.functions().clone(),
        original.fields().clone(),
    )
    .unwrap()
    .with_constants(constants)
    .unwrap();
    assert_ne!(result, original);
    result
}
