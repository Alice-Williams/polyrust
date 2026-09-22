//! Corrupt both compiler expectations and retained descriptive metadata.
use super::*;
pub(crate) fn check(
    package: &RenderReadyPackage<CDialect>,
    manifest: &ApiManifest,
    functions: &BTreeMap<RustDeclarationId, CFunctionRef>,
    expected: &constants::ExpectedConstants,
) {
    let reconstruct = |expected| {
        ApiManifest::with_constants(
            package,
            manifest.exports.clone(),
            functions,
            &BTreeMap::new(),
            expected,
        )
    };
    let verify = |candidate: &ApiManifest| {
        candidate.verify_constants(
            package,
            manifest.exports.clone(),
            functions,
            &BTreeMap::new(),
            expected,
        )
    };
    assert!(verify(manifest).is_ok());
    let (&id, (object, value)) = expected.first_key_value().unwrap();
    let mut absent = expected.clone();
    absent.remove(&id);
    assert!(reconstruct(&absent).is_err());
    let mut changed = expected.clone();
    let wrong_value = match value {
        CScalarConstantValue::Bool(v) => CScalarConstantValue::Bool(!v),
        CScalarConstantValue::I32(v) => CScalarConstantValue::I32(v.wrapping_add(1)),
        CScalarConstantValue::I64(v) => CScalarConstantValue::I64(v.wrapping_add(1)),
        CScalarConstantValue::F64(v) => CScalarConstantValue::F64(
            portable_binary64::FiniteBinary64::from_bits(v.to_bits() ^ (1_u64 << 63)).unwrap(),
        ),
        CScalarConstantValue::Infinity(sign) => CScalarConstantValue::Infinity(match sign {
            portable_binary64::Binary64Sign::Positive => portable_binary64::Binary64Sign::Negative,
            portable_binary64::Binary64Sign::Negative => portable_binary64::Binary64Sign::Positive,
        }),
    };
    changed.insert(id, (object.clone(), wrong_value));
    assert!(reconstruct(&changed).is_err());
    let mut altered = manifest.clone();
    altered.constants.get_mut(&id).unwrap().value = wrong_value;
    assert!(verify(&altered).is_err());
    let mut altered = manifest.clone();
    altered.constants.get_mut(&id).unwrap().name = CIdentifier::new("wrong_name").unwrap();
    assert!(verify(&altered).is_err());
    let mut altered = manifest.clone();
    altered.constants.remove(&id);
    assert!(verify(&altered).is_err());
    let mut altered = manifest.clone();
    Arc::make_mut(&mut altered.exports)
        .modules
        .get_mut(&manifest.exports.root)
        .unwrap()
        .retain(|_, value| *value != RustExportTarget::Declaration(id));
    assert!(verify(&altered).is_err());
    let mut changed = expected.clone();
    let mut foreign = id;
    foreign.crate_id ^= 1;
    let moved = changed.remove(&id).unwrap();
    changed.insert(foreign, moved);
    assert!(reconstruct(&changed).is_err());
    let bundled = manifest.bundle_json().unwrap();
    let version = if manifest.has_binary64_metadata() {
        8
    } else {
        4
    };
    assert!(bundled.contains(&format!("\"schema_version\":{version}")));
    assert!(bundled.len() <= manifest.bundle_bound().unwrap());
    println!("PUBLIC_CONSTANT_MANIFEST_MUTATIONS\t7");
}
