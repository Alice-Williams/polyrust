//! The shared graph must be an exact authenticated projection, not a parallel AST.
use super::*;
use crate::ast::*;
use portable_codegen::{
    RelativeOutputPath, TargetAstBuilder, TargetLinker, TargetTypeRef, certify_resolved_package,
    verify_unresolved_package,
};

pub(super) fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}

pub(super) fn fixture(scalar: CScalarType) -> (CFrozenRegistry, CSourceFile) {
    qualified_fixture(scalar, CConstness::Unqualified)
}

pub(super) fn qualified_fixture(
    scalar: CScalarType,
    constness: CConstness,
) -> (CFrozenRegistry, CSourceFile) {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("tests/projected.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let ty = CObjectType::scalar(scalar);
    let function = registry
        .register_function(
            &file,
            key("identity"),
            CFunctionType::new(
                CReturnType::Value(CReturnValue::new(ty.clone()).unwrap()),
                vec![CParameterType::new(ty).unwrap()],
            ),
        )
        .unwrap();
    let parameter = registry
        .register_parameter(&function, 0, key("input"), constness)
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("body"))
        .unwrap();
    let expressions = CExpressions::new(&registry);
    let value = expressions
        .read(expressions.parameter(parameter.clone()).unwrap())
        .unwrap();
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let body = statements
        .block(
            scope,
            vec![statements.return_statement(Some(value)).unwrap()],
        )
        .unwrap();
    let declarations = CDeclarations::new(&registry, file).unwrap();
    let prototype = declarations
        .function_prototype(function.clone(), CLinkage::External)
        .unwrap();
    let definition = declarations
        .function_definition(function, CLinkage::External, vec![parameter], body)
        .unwrap();
    let source = declarations
        .source_file(vec![
            CFileItem::Declaration(prototype),
            CFileItem::Definition(definition),
        ])
        .unwrap();
    (registry.freeze(), source)
}

#[derive(Clone, Copy, Debug)]
enum Mutation {
    None,
    MissingCallable,
    ExtraCallable,
    Signature,
    ValueType,
    Origin,
    MissingGroup,
    FilePath,
}

fn rebuild(package: &TargetAstPackage<CDialect>, mutation: Mutation) -> TargetAstPackage<CDialect> {
    let mut builder = TargetAstBuilder::new(CDialect);
    for ty in package.generated_types() {
        builder.generated_type(ty.clone());
    }
    for callable in package.callables() {
        if matches!(mutation, Mutation::MissingCallable) {
            continue;
        }
        let mut callable = callable.clone();
        if matches!(mutation, Mutation::Signature) {
            callable.signature.parameters.clear();
        }
        if matches!(mutation, Mutation::Origin) {
            callable.origin = portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::EvaluationTemporary,
            );
        }
        builder.callable(callable.clone());
        if matches!(mutation, Mutation::ExtraCallable) {
            builder.callable(callable);
        }
    }
    for value in package.values() {
        let mut value = value.clone();
        if matches!(mutation, Mutation::ValueType) {
            value.ty =
                TargetTypeRef::Primitive(crate::dialect::CPrimitiveType::Scalar(CScalarType::Bool));
        }
        builder.value(value);
    }
    for file in package.files() {
        let file = if matches!(mutation, Mutation::FilePath) {
            portable_codegen::TargetFile::new(
                RelativeOutputPath::new("tests/forged.c").unwrap(),
                file.role(),
                file.module().clone(),
                *file.placement(),
                file.items().to_vec(),
                file.source_kind().clone(),
                file.source().clone(),
            )
        } else {
            file.clone()
        };
        builder.file(file);
    }
    if !matches!(mutation, Mutation::MissingGroup) {
        for group in package.groups() {
            builder.group(group.clone());
        }
    }
    builder.build()
}

#[test]
fn exact_registry_projection_links_with_type_derived_imports() {
    // Both typedef headers are required by the exact typed platform queries,
    // even when the function body itself only uses _Bool.
    for (scalar, includes) in [(CScalarType::I32, 2), (CScalarType::Bool, 2)] {
        let (registry, source) = fixture(scalar);
        let package = project_c_package(registry, vec![source]).unwrap();
        assert_eq!(package.callables().len(), 1);
        assert_eq!(package.values().len(), 1);
        for _ in 0..3 {
            let verified =
                verify_unresolved_package(&CDialect, rebuild(&package, Mutation::None)).unwrap();
            let linked = TargetLinker::new(CDialect).link_ast(&verified).unwrap();
            verify_linked_package(&linked).unwrap();
            assert_eq!(linked.files().first().unwrap().imports().len(), includes);
            assert!(certify_resolved_package(&CDialect, linked).is_ok());
        }
    }
}

#[test]
fn missing_or_forged_shared_inventory_cannot_be_verified() {
    let (registry, source) = fixture(CScalarType::I32);
    let package = project_c_package(registry, vec![source]).unwrap();
    for mutation in [
        Mutation::MissingCallable,
        Mutation::ExtraCallable,
        Mutation::Signature,
        Mutation::ValueType,
        Mutation::Origin,
        Mutation::MissingGroup,
        Mutation::FilePath,
    ] {
        assert!(
            verify_unresolved_package(&CDialect, rebuild(&package, mutation)).is_err(),
            "{mutation:?}"
        );
    }
    assert!(verify_unresolved_package(&CDialect, TargetAstBuilder::new(CDialect).build()).is_err());
}

#[test]
fn copied_names_do_not_authenticate_a_foreign_registry() {
    let (registry, _) = fixture(CScalarType::I32);
    let (_, source) = fixture(CScalarType::I32);
    assert!(project_c_package(registry, vec![source]).is_err());
}
