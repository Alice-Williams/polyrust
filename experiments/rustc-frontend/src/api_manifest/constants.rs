//! Bidirectional owned constant metadata, reconstructed from certified objects.
use super::*;
use portable_backend_c::dialect::c_defined_constants;

pub(crate) type ExpectedConstants = BTreeMap<RustDeclarationId, (CObjectRef, CScalarConstantValue)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Constant {
    pub reference: CObjectRef,
    pub name: CIdentifier,
    pub value: CScalarConstantValue,
}

pub(super) fn collect(
    package: &RenderReadyPackage<CDialect>,
    exports: &Arc<RustCrateExports>,
    header: &CFileRef,
    implementation: &CFileRef,
    expected: &ExpectedConstants,
    public: &BTreeSet<RustDeclarationId>,
) -> Result<BTreeMap<RustDeclarationId, Constant>, String> {
    let mut constants = BTreeMap::new();
    for definition in c_defined_constants(package) {
        let reference = definition.object();
        let CGeneratedOrigin::RustSource(origin) = &reference.key().origin else {
            return Err("API constant lacks compiler provenance".into());
        };
        let id = origin.declaration;
        let value = *definition.value();
        if origin.node != RustSourceNode::Declaration
            || id.crate_id != exports.root.crate_id
            || &origin.crate_exports != exports
            || !origin.externally_reachable
            || !public.contains(&id)
            || reference.file() != header
            || definition.implementation() != implementation
            || definition.linkage() != CLinkage::External
            || expected.get(&id) != Some(&(reference.clone(), value))
        {
            return Err("API manifest compiler/constant/file/value mapping disagrees".into());
        }
        scalar(&value)?;
        if constants
            .insert(
                id,
                Constant {
                    reference: reference.clone(),
                    name: definition.name().clone(),
                    value,
                },
            )
            .is_some()
        {
            return Err("duplicate API constant identity".into());
        }
    }
    if constants.len() != expected.len() {
        return Err("API manifest misses a source or target constant".into());
    }
    Ok(constants)
}

/// Integers use decimal strings; binary64 uses fixed-width hexadecimal bits.
/// Neither encoding loses bits in consumers whose JSON numbers are doubles.
pub(super) fn scalar(value: &CScalarConstantValue) -> Result<(&'static str, String), String> {
    match value {
        CScalarConstantValue::Bool(value) => Ok(("bool", value.to_string())),
        CScalarConstantValue::I32(value) => Ok(("i32", serialization::quote(&value.to_string()))),
        CScalarConstantValue::I64(value) => Ok(("i64", serialization::quote(&value.to_string()))),
        CScalarConstantValue::F64(value) => Ok((
            "f64",
            serialization::quote(&format!("0x{:016x}", value.to_bits())),
        )),
        CScalarConstantValue::Infinity(sign) => Ok((
            "f64",
            serialization::quote(match sign {
                portable_binary64::Binary64Sign::Positive => "0x7ff0000000000000",
                portable_binary64::Binary64Sign::Negative => "0xfff0000000000000",
            }),
        )),
    }
}

pub(super) fn certified_scalar(
    value: &portable_backend_c::ast::CScalarConstantValue,
) -> Result<(&'static str, String), String> {
    scalar(value)
}
