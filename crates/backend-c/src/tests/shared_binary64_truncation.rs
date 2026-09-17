//! Typed standard truncation, original dependency closure and negative contracts.
use super::*;
use crate::dialect::{CKnownCall, CSystemLibrary};
use std::collections::BTreeSet;

pub(super) fn fixture(crate_id: u64, call: crate::dialect::CKnownCall) -> f::Fixture {
    let mut fixture = f::fixture(crate_id, &[CScalarType::F64], &[], &[None]);
    let expressions = CExpressions::new(fixture.registry.registrations());
    let declarations = CDeclarations::new(
        fixture.registry.registrations(),
        fixture.files[1].identity().clone(),
    )
    .unwrap();
    let items = fixture.files[1]
        .items()
        .iter()
        .map(|item| {
            let CFileItem::Definition(definition) = item else {
                panic!("function")
            };
            let CDefinitionKind::Function {
                function,
                linkage,
                parameters,
                body,
            } = definition.kind()
            else {
                panic!("function")
            };
            let mut prefix = body.statements().to_vec();
            let returned = prefix.pop().unwrap();
            let CStatementKind::Return(Some(operand)) = returned.kind() else {
                panic!("fixture returns its input or materialized call");
            };
            let arguments = if call == crate::dialect::CKnownCall::FloatRemainder {
                vec![operand.clone(), operand.clone()]
            } else {
                vec![operand.clone()]
            };
            let value = expressions
                .call_value(expressions.known(call), arguments)
                .unwrap();
            assert_eq!(
                value.ty().kind(),
                &CObjectTypeKind::Scalar(CScalarType::F64)
            );
            let statements =
                CStatements::new(fixture.registry.registrations(), function.clone()).unwrap();
            prefix.push(statements.return_statement(Some(value)).unwrap());
            let body = statements.block(body.scope().clone(), prefix).unwrap();
            CFileItem::Definition(
                declarations
                    .function_definition(function.clone(), *linkage, parameters.clone(), body)
                    .unwrap(),
            )
        })
        .collect();
    fixture.files[1] = declarations.source_file(items).unwrap();
    fixture
}

pub(super) fn chain() -> Vec<CDependencyApi> {
    let first = api(&fixture(101, CKnownCall::FloatTruncate));
    let imports: Vec<_> = first.functions().cloned().collect();
    let second = api(&f::materialized_double_calls(102, &imports));
    let imports: Vec<_> = second.functions().cloned().collect();
    let third = api(&f::materialized_double_calls(103, &imports));
    vec![first, second, third]
}

#[test]
fn truncation_headers_and_transitive_link_libraries_are_certificate_derived() {
    for (index, owner) in chain().iter().enumerate() {
        assert_eq!(
            owner.system_libraries(),
            &BTreeSet::from([CSystemLibrary::Math])
        );
        for function in owner.functions() {
            assert_eq!(
                function.package_identity().system_libraries(),
                owner.system_libraries()
            );
        }
        let rendered = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        for file in rendered.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert_eq!(
                text.contains("#include <math.h>"),
                index == 0 && file.path().ends_with(".c")
            );
            assert!(!text.contains("runtime") && !text.contains("memcpy"));
        }
    }
    let identity = api(&f::fixture(104, &[CScalarType::F64], &[], &[None]));
    assert!(identity.system_libraries().is_empty());
    let rendered = render_certified_package(&CStructuralRenderer, identity.package()).unwrap();
    for file in rendered.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("text")
        };
        assert!(!text.contains("math.h"));
    }
}

#[test]
fn unrelated_standard_call_and_invalid_truncation_arguments_reject() {
    let remainder = fixture(105, CKnownCall::FloatRemainder);
    assert!(
        crate::dialect::project_c_package(remainder.registry.clone(), remainder.files).is_err()
    );
    let f = f::fixture(106, &[CScalarType::F64], &[], &[None]);
    let e = CExpressions::new(f.registry.registrations());
    let call = || e.known(CKnownCall::FloatTruncate);
    let zero = || {
        e.literal(CLiteral::F64(
            portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
        ))
        .unwrap()
    };
    assert!(e.call_value(call(), vec![]).is_err());
    assert!(e.call_value(call(), vec![zero(), zero()]).is_err());
    assert!(
        e.call_value(call(), vec![e.literal(CLiteral::Bool(false)).unwrap()])
            .is_err()
    );
    assert!(
        e.call_value(
            call(),
            vec![e.literal(CLiteral::Signed(CSignedLiteral::I64(0))).unwrap()]
        )
        .is_err()
    );
}

#[path = "shared_binary64_truncation_native.rs"]
mod native;

#[path = "shared_truncation_constant_import.rs"]
mod constant_import;

#[path = "shared_truncation_stack_native.rs"]
mod stack_native;

#[path = "shared_truncation_resources.rs"]
mod stack_resources;
