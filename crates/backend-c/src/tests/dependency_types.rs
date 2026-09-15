//! Independent header/completeness matrices for the structural dependency pass.

use super::{CFileDependencies, CTagDependency, CTypeRequirement};
use crate::ast::{
    CArrayLength, CConstness, CDeclarationKey, CFileKey, CFileRef, CFileRole, CFunctionType,
    CGeneratedOrigin, CIdentifier, CKnownConstant, CKnownObject, CObjectType, CParameterType,
    CPointerTarget, CRegistry, CReturnType, CReturnValue, CScalarType, CSynthesisReason,
};
use crate::dialect::CHeader;
use portable_codegen::RelativeOutputPath;
use std::collections::BTreeSet;

pub(super) fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}

pub(super) fn fixture() -> (CRegistry, CFileRef) {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("tests/probe.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    (registry, file)
}

fn pointer(ty: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Object(Box::new(ty)))
}

#[test]
fn every_scalar_has_an_independent_header_expectation() {
    let (_, file) = fixture();
    let rows = [
        (CScalarType::Bool, None),
        (CScalarType::PlainChar, None),
        (CScalarType::Int, None),
        (CScalarType::F64, None),
        (CScalarType::Size, Some(CHeader::Stddef)),
        (CScalarType::I8, Some(CHeader::Stdint)),
        (CScalarType::U8, Some(CHeader::Stdint)),
        (CScalarType::I16, Some(CHeader::Stdint)),
        (CScalarType::U16, Some(CHeader::Stdint)),
        (CScalarType::I32, Some(CHeader::Stdint)),
        (CScalarType::U32, Some(CHeader::Stdint)),
        (CScalarType::I64, Some(CHeader::Stdint)),
        (CScalarType::U64, Some(CHeader::Stdint)),
    ];
    assert_eq!(rows.len(), CScalarType::ALL.len());
    assert_eq!(
        rows.iter()
            .map(|(scalar, _)| *scalar)
            .collect::<BTreeSet<_>>(),
        CScalarType::ALL.into()
    );
    for (scalar, header) in rows {
        let mut dependencies = CFileDependencies::new(file.clone());
        dependencies.object_type(
            &pointer(pointer(CObjectType::scalar(scalar))),
            CTypeRequirement::Complete,
        );
        assert_eq!(
            dependencies.headers(),
            &header.into_iter().collect(),
            "{scalar:?}"
        );
        assert_eq!(dependencies.scalars(), &BTreeSet::from([scalar]));
    }
}

#[test]
fn constants_and_library_owned_objects_keep_their_header_identity() {
    use CKnownConstant as K;
    for (values, expected) in [
        (vec![K::CharBit, K::IntMin, K::IntMax], CHeader::Limits),
        (
            vec![
                K::I32Min,
                K::I32Max,
                K::U32Max,
                K::I64Min,
                K::I64Max,
                K::U64Max,
                K::SizeMax,
            ],
            CHeader::Stdint,
        ),
        (
            vec![
                K::FloatRadix,
                K::DoubleMantissaDigits,
                K::DoubleMinExponent,
                K::DoubleMaxExponent,
                K::FloatEvaluationMethod,
            ],
            CHeader::Float,
        ),
        (
            vec![
                K::EndOfFile,
                K::StandardInput,
                K::StandardOutput,
                K::StandardError,
            ],
            CHeader::Stdio,
        ),
    ] {
        for value in values {
            assert_eq!(value.header(), expected);
        }
    }
    assert_eq!(CKnownObject::File.header(), CHeader::Stdio);
    assert_eq!(CKnownObject::MaxAlign.header(), CHeader::Stddef);
    assert_eq!(CHeader::Stdint.spelling(), "stdint.h");
    assert_eq!(CHeader::Stddef.spelling(), "stddef.h");
}

#[test]
fn nested_pointers_aliases_arrays_and_prototypes_keep_exact_requirements() {
    let (mut registry, file) = fixture();
    let record = registry.declare_struct(&file, key("record")).unwrap();
    let ty = CObjectType::structure(record.clone());
    let alias = registry
        .register_typedef(&file, key("record_alias"), ty.clone())
        .unwrap();
    let tag = CTagDependency::Struct(record.clone());
    let mut dependencies = CFileDependencies::new(file.clone());
    dependencies.object_type(
        &pointer(pointer(CObjectType::typedef(alias.clone()))),
        CTypeRequirement::Complete,
    );
    assert_eq!(
        dependencies.tags().get(&tag),
        Some(&CTypeRequirement::Declaration)
    );
    assert_eq!(dependencies.aliases(), &BTreeSet::from([alias]));
    let array = CObjectType::array(ty.clone(), CArrayLength::new(2).unwrap()).unwrap();
    dependencies.object_type(&pointer(array), CTypeRequirement::Declaration);
    assert_eq!(
        dependencies.tags().get(&tag),
        Some(&CTypeRequirement::Complete)
    );
    dependencies.object_type(&pointer(ty.clone()), CTypeRequirement::Complete);
    assert_eq!(
        dependencies.tags().get(&tag),
        Some(&CTypeRequirement::Complete)
    );

    let mut prototype = CFileDependencies::new(file);
    let signature = CFunctionType::new(
        CReturnType::Value(CReturnValue::new(ty.clone()).unwrap()),
        vec![CParameterType::new(ty).unwrap()],
    );
    prototype.object_type(
        &CObjectType::pointer(CPointerTarget::Function(Box::new(signature))),
        CTypeRequirement::Complete,
    );
    assert_eq!(
        prototype.tags().get(&tag),
        Some(&CTypeRequirement::Declaration)
    );
    assert!(prototype.headers().is_empty());
}

#[test]
fn alias_parameters_are_not_replaced_by_only_their_canonical_type() {
    let (mut registry, file) = fixture();
    let alias = registry
        .register_typedef(&file, key("count"), CObjectType::scalar(CScalarType::I32))
        .unwrap();
    let signature = CFunctionType::new(
        CReturnType::Void,
        vec![CParameterType::new(CObjectType::typedef(alias.clone())).unwrap()],
    );
    let mut dependencies = CFileDependencies::new(file);
    dependencies.signature(&signature, CTypeRequirement::Declaration);
    assert_eq!(dependencies.aliases(), &BTreeSet::from([alias]));
    assert_eq!(dependencies.headers(), &BTreeSet::from([CHeader::Stdint]));
}

#[test]
fn enum_pointers_require_definitions_but_void_pointers_require_nothing() {
    let (mut registry, file) = fixture();
    let enumeration = registry.declare_enum(&file, key("choice")).unwrap();
    let mut dependencies = CFileDependencies::new(file);
    dependencies.object_type(
        &CObjectType::pointer(CPointerTarget::Void(CConstness::Const)),
        CTypeRequirement::Complete,
    );
    assert!(dependencies.tags().is_empty());
    assert!(dependencies.headers().is_empty());
    dependencies.object_type(
        &pointer(CObjectType::enumeration(enumeration.clone())),
        CTypeRequirement::Declaration,
    );
    assert_eq!(
        dependencies.tags().get(&CTagDependency::Enum(enumeration)),
        Some(&CTypeRequirement::Complete)
    );
}
