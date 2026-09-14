//! Typed trees must satisfy the declared warning-clean native profile too.
use super::{project_c_package, record_fixture, tests::fixture};
use crate::ast::*;

#[test]
fn missing_or_late_prototypes_and_late_records_are_rejected() {
    for late in [false, true] {
        let (registry, source) = fixture(CScalarType::I32);
        let declarations =
            CDeclarations::new(registry.registrations(), source.identity().clone()).unwrap();
        let mut items = source.items().to_vec();
        if late {
            items.swap(0, 1);
        } else {
            items.remove(0);
        }
        let source = declarations.source_file(items).unwrap();
        let errors = project_c_package(registry, vec![source]).unwrap_err();
        assert!(errors[0].message.contains("prototype before"));
    }
    let (registry, source) = record_fixture::fixture();
    let declarations =
        CDeclarations::new(registry.registrations(), source.identity().clone()).unwrap();
    let mut items = source.items().to_vec();
    let index = items.iter().position(|item| matches!(item, CFileItem::Declaration(decl) if matches!(decl.kind(), CDeclarationKind::Aggregate { .. }))).unwrap();
    let record = items.remove(index);
    items.push(record);
    let source = declarations.source_file(items).unwrap();
    let errors = project_c_package(registry, vec![source]).unwrap_err();
    assert!(errors[0].message.contains("record declarations before"));
}

#[test]
fn unused_parameter_requires_explicit_typed_discard() {
    for consume in [false, true] {
        let (registry, source) = fixture(CScalarType::I32);
        let CFileItem::Definition(definition) = &source.items()[1] else {
            panic!()
        };
        let CDefinitionKind::Function {
            function,
            linkage,
            parameters,
            body,
        } = definition.kind()
        else {
            panic!()
        };
        let ast = CStatements::new(registry.registrations(), function.clone()).unwrap();
        let expressions = CExpressions::new(registry.registrations());
        let mut statements = vec![];
        if consume {
            statements.push(
                ast.discard(
                    expressions
                        .read(expressions.parameter(parameters[0].clone()).unwrap())
                        .unwrap(),
                )
                .unwrap(),
            );
        }
        statements.push(
            ast.return_statement(Some(
                expressions
                    .literal(CLiteral::Signed(CSignedLiteral::I32(42)))
                    .unwrap(),
            ))
            .unwrap(),
        );
        let body = ast.block(body.scope().clone(), statements).unwrap();
        let declarations =
            CDeclarations::new(registry.registrations(), source.identity().clone()).unwrap();
        let definition = declarations
            .function_definition(function.clone(), *linkage, parameters.clone(), body)
            .unwrap();
        let source = declarations
            .source_file(vec![
                source.items()[0].clone(),
                CFileItem::Definition(definition),
            ])
            .unwrap();
        let result = project_c_package(registry, vec![source]);
        assert_eq!(result.is_ok(), consume);
        if let Err(errors) = result {
            assert!(errors[0].message.contains("unused bindings"));
        }
    }
}

#[test]
fn unused_local_requires_explicit_typed_discard() {
    for consume in [false, true] {
        let (registry, source) = super::storage_fixture::fixture(1, 0);
        let CFileItem::Definition(definition) = &source.items()[1] else {
            panic!()
        };
        let CDefinitionKind::Function {
            function,
            linkage,
            parameters,
            body,
        } = definition.kind()
        else {
            panic!()
        };
        let CStatementKind::Declare(local) = body.statements()[0].kind() else {
            panic!()
        };
        let ast = CStatements::new(registry.registrations(), function.clone()).unwrap();
        let expressions = CExpressions::new(registry.registrations());
        let mut statements = vec![body.statements()[0].clone()];
        if consume {
            statements.push(
                ast.discard(
                    expressions
                        .address_of(expressions.local(local.local().clone()).unwrap())
                        .unwrap(),
                )
                .unwrap(),
            );
        }
        statements.push(
            ast.return_statement(Some(
                expressions
                    .read(expressions.parameter(parameters[0].clone()).unwrap())
                    .unwrap(),
            ))
            .unwrap(),
        );
        let body = ast.block(body.scope().clone(), statements).unwrap();
        let declarations =
            CDeclarations::new(registry.registrations(), source.identity().clone()).unwrap();
        let definition = declarations
            .function_definition(function.clone(), *linkage, parameters.clone(), body)
            .unwrap();
        let source = declarations
            .source_file(vec![
                source.items()[0].clone(),
                CFileItem::Definition(definition),
            ])
            .unwrap();
        let result = project_c_package(registry, vec![source]);
        assert_eq!(result.is_ok(), consume);
        if let Err(errors) = result {
            assert!(errors[0].message.contains("unused bindings"));
        }
    }
}
