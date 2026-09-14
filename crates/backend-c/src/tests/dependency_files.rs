//! Dependency discovery from expressions and authenticated complete files.

use super::{
    CFileDependencies, file_dependencies,
    tests::{fixture, key},
};
use crate::ast::{
    CAssertDiagnostic, CDeclarations, CExpressions, CFileItem, CFileKey, CFileRole, CFunctionType,
    CKnownConstant, CLiteral, CReturnType, CScalarType, CSignedLiteral,
};
use crate::dialect::{CHeader, CKnownCall, CSystemLibrary};
use portable_codegen::RelativeOutputPath;
use std::collections::BTreeSet;

#[test]
fn nested_calls_require_headers_and_link_libraries_not_just_result_types() {
    let (registry, file) = fixture();
    let ast = CExpressions::new(&registry);
    let integer = ast
        .literal(CLiteral::Signed(CSignedLiteral::Int(2)))
        .unwrap();
    let double = ast.numeric_conversion(CScalarType::F64, integer).unwrap();
    let value = ast
        .call_value(ast.known(CKnownCall::FloatTruncate), vec![double])
        .unwrap();
    let value = ast.numeric_conversion(CScalarType::Int, value).unwrap();
    let mut dependencies = CFileDependencies::new(file);
    dependencies.value(&value);
    assert_eq!(dependencies.headers(), &BTreeSet::from([CHeader::Math]));
    assert_eq!(
        dependencies.libraries(),
        &BTreeSet::from([CSystemLibrary::Math])
    );
    assert!(dependencies.functions().is_empty());
}

#[test]
fn known_constants_survive_nested_expressions_and_header_removal_is_observable() {
    let (registry, file) = fixture();
    let ast = CExpressions::new(&registry);
    let known = ast.known_constant(CKnownConstant::CharBit);
    let known = ast.numeric_conversion(CScalarType::Bool, known).unwrap();
    let declaration = CDeclarations::new(&registry, file.clone()).unwrap();
    let assertion = declaration
        .static_assert(known, CAssertDiagnostic::new("byte width"))
        .unwrap();
    let present = declaration
        .source_file(vec![CFileItem::StaticAssert(assertion)])
        .unwrap();
    let absent = declaration.source_file(vec![]).unwrap();
    let frozen = registry.freeze();
    let first = file_dependencies(&frozen, std::slice::from_ref(&present)).unwrap();
    assert_eq!(first[0].headers(), &BTreeSet::from([CHeader::Limits]));
    assert!(
        file_dependencies(&frozen, &[absent]).unwrap()[0]
            .headers()
            .is_empty()
    );
    assert_eq!(first, file_dependencies(&frozen, &[present]).unwrap());
}

#[test]
fn same_spelled_foreign_file_is_not_accepted_and_file_order_is_canonical() {
    let (mut registry, file) = fixture();
    let (foreign, foreign_file) = fixture();
    let foreign_source = CDeclarations::new(&foreign, foreign_file)
        .unwrap()
        .source_file(vec![])
        .unwrap();
    let another = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("tests/another.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let first = CDeclarations::new(&registry, file)
        .unwrap()
        .source_file(vec![])
        .unwrap();
    let second = CDeclarations::new(&registry, another)
        .unwrap()
        .source_file(vec![])
        .unwrap();
    let frozen = registry.freeze();
    assert!(file_dependencies(&frozen, &[foreign_source]).is_err());
    let ordered = file_dependencies(&frozen, &[first.clone(), second.clone()]).unwrap();
    let reversed = file_dependencies(&frozen, &[second, first]).unwrap();
    assert_eq!(ordered, reversed);
    assert_eq!(ordered[0].file().key().path.as_str(), "tests/another.c");
}

#[test]
fn function_addresses_keep_registered_function_identity() {
    let (mut registry, file) = fixture();
    let function = registry
        .register_function(
            &file,
            key("function"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let address = CExpressions::new(&registry)
        .function_address(function.clone())
        .unwrap();
    let mut dependencies = CFileDependencies::new(file);
    dependencies.value(&address);
    assert_eq!(dependencies.functions(), &BTreeSet::from([function]));
    assert!(dependencies.headers().is_empty());
}

#[test]
fn indirect_calls_use_actual_pointer_aliases_not_proof_witness_aliases() {
    use super::{CTagDependency, CTypeRequirement};
    use crate::ast::{CObjectType, CPointerTarget, CReturnValue};
    let (mut registry, file) = fixture();
    let record = registry.declare_struct(&file, key("record")).unwrap();
    let actual_alias = registry
        .register_typedef(
            &file,
            key("actual_result"),
            CObjectType::structure(record.clone()),
        )
        .unwrap();
    let proof_alias = registry
        .register_typedef(
            &file,
            key("proof_result"),
            CObjectType::structure(record.clone()),
        )
        .unwrap();
    let signature = |alias| {
        CFunctionType::new(
            CReturnType::Value(CReturnValue::new(CObjectType::typedef(alias)).unwrap()),
            vec![],
        )
    };
    let witness = registry
        .register_function(&file, key("witness"), signature(proof_alias.clone()))
        .unwrap();
    let pointer = registry
        .register_object(
            &file,
            key("callback"),
            CObjectType::pointer(CPointerTarget::Function(Box::new(signature(
                actual_alias.clone(),
            )))),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let pointer = ast.read(ast.global(pointer).unwrap()).unwrap();
    let mut pointer_only = CFileDependencies::new(file.clone());
    pointer_only.value(&pointer);
    let tag = CTagDependency::Struct(record);
    assert_eq!(
        pointer_only.tags().get(&tag),
        Some(&CTypeRequirement::Declaration)
    );
    let call = ast
        .call_value(ast.indirect(pointer, witness).unwrap(), vec![])
        .unwrap();
    let mut dependencies = CFileDependencies::new(file);
    dependencies.value(&call);
    assert_eq!(dependencies.aliases(), &BTreeSet::from([actual_alias]));
    assert!(!dependencies.aliases().contains(&proof_alias));
    assert!(dependencies.functions().is_empty());
    assert_eq!(
        dependencies.tags().get(&tag),
        Some(&CTypeRequirement::Complete)
    );
}
