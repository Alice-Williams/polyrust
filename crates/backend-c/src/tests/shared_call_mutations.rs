//! Direct calls cannot weaken declaration, signature or admission boundaries.
use super::{
    CDialect,
    call_fixture::{fixture, scalar_fixture},
    project_c_package,
};
use crate::ast::*;
use portable_codegen::{TargetLinker, certify_resolved_package, verify_unresolved_package};

#[test]
fn call_requires_an_earlier_prototype_and_a_matching_definition() {
    for mutation in 0..4 {
        let (registry, source) = fixture(&[vec![1], vec![]], 0, true);
        assert!(project_c_package(registry.clone(), vec![source.clone()]).is_ok());
        let declarations =
            CDeclarations::new(registry.registrations(), source.identity().clone()).unwrap();
        let mut items = source.items().to_vec();
        match mutation {
            0 => {
                items.remove(1);
            }
            1 => {
                let prototype = items.remove(1);
                items.insert(2, prototype);
            }
            2 => {
                items.pop();
            }
            3 => {
                let CFileItem::Declaration(prototype) = &items[1] else {
                    panic!()
                };
                let CDeclarationKind::FunctionPrototype { function, .. } = prototype.kind() else {
                    panic!()
                };
                items[1] = CFileItem::Declaration(
                    declarations
                        .function_prototype(function.clone(), CLinkage::External)
                        .unwrap(),
                );
            }
            _ => unreachable!(),
        }
        let source = declarations.source_file(items).unwrap();
        assert!(
            project_c_package(registry, vec![source]).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn exact_callable_signature_and_registry_identity_are_required() {
    let (registry, source) = fixture(&[vec![1], vec![]], 0, true);
    let CFileItem::Declaration(prototype) = &source.items()[1] else {
        panic!()
    };
    let CDeclarationKind::FunctionPrototype { function, .. } = prototype.kind() else {
        panic!()
    };
    let expressions = CExpressions::new(registry.registrations());
    let target = expressions.direct(function.clone()).unwrap();
    let integer = expressions
        .literal(CLiteral::Signed(CSignedLiteral::I32(42)))
        .unwrap();
    assert!(
        expressions
            .call_value(target.clone(), vec![integer.clone()])
            .is_ok()
    );
    assert!(expressions.call_value(target.clone(), vec![]).is_err());
    assert!(
        expressions
            .call_value(target.clone(), vec![integer.clone(), integer])
            .is_err()
    );
    let boolean = expressions.literal(CLiteral::Bool(true)).unwrap();
    assert!(expressions.call_value(target, vec![boolean]).is_err());
    let (other, _) = fixture(&[vec![1], vec![]], 0, true);
    assert!(
        CExpressions::new(other.registrations())
            .direct(function.clone())
            .is_err()
    );
}

#[test]
fn function_definition_cannot_reuse_another_functions_body_or_parameters() {
    let (registry, source) = fixture(&[vec![1], vec![]], 0, true);
    let definition = |index| {
        let CFileItem::Definition(definition) = &source.items()[index] else {
            panic!()
        };
        let CDefinitionKind::Function {
            function,
            parameters,
            body,
            ..
        } = definition.kind()
        else {
            panic!()
        };
        (function, parameters, body)
    };
    let (root, parameters, body) = definition(2);
    let (_, other_parameters, other_body) = definition(3);
    let declarations =
        CDeclarations::new(registry.registrations(), source.identity().clone()).unwrap();
    assert!(
        declarations
            .function_definition(
                root.clone(),
                CLinkage::External,
                parameters.clone(),
                (**body).clone()
            )
            .is_ok()
    );
    assert!(
        declarations
            .function_definition(
                root.clone(),
                CLinkage::External,
                other_parameters.clone(),
                (**body).clone()
            )
            .is_err()
    );
    assert!(
        declarations
            .function_definition(
                root.clone(),
                CLinkage::External,
                parameters.clone(),
                (**other_body).clone()
            )
            .is_err()
    );
}

#[test]
fn call_arity_resource_limit_is_not_an_ast_constructor_cap() {
    for kind in [CScalarType::I32, CScalarType::Int, CScalarType::Bool] {
        for arity in [0, 3, 127, 128] {
            let (registry, source) = scalar_fixture(&[vec![1], vec![]], 0, true, &[1, arity], kind);
            let package = project_c_package(registry, vec![source]).unwrap();
            let checked = verify_unresolved_package(&CDialect, package).unwrap();
            let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
            let result = certify_resolved_package(&CDialect, linked);
            assert_eq!(result.is_ok(), arity <= 127, "{kind:?} arity {arity}");
            if let Err(errors) = result {
                assert!(
                    errors.iter().all(|error| error.code
                        == portable_diagnostics::DiagnosticCode::TargetResourceLimit)
                );
            }
        }
    }
}
