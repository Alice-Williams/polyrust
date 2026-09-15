//! Coupled mapping mutations cannot override the authoritative registered sources.
use super::{
    CDialect, CFileGrammar,
    bindings::CBindings,
    constant_consumer_fixture::{Usage, fixture, producer},
    owned_constant_fixture::Shape,
    project_c_package,
};
use crate::ast::*;
use portable_codegen::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
enum Fault {
    None,
    Missing,
    Swap,
    Coupled,
}

fn change(bindings: &mut CBindings, fault: Fault) {
    let entries: Vec<_> = bindings
        .imported_values
        .iter()
        .map(|(a, b)| (a.clone(), b.clone()))
        .collect();
    let [(first, a), (second, b), ..] = entries.as_slice() else {
        panic!("two imports");
    };
    match fault {
        Fault::None => {}
        Fault::Missing => {
            bindings.imported_values.remove(first);
        }
        Fault::Swap | Fault::Coupled => {
            bindings.imported_values.insert(first.clone(), b.clone());
            bindings.imported_values.insert(second.clone(), a.clone());
        }
    }
}

#[test]
fn coordinated_value_maps_cannot_retarget_the_original_ast() {
    let owner = producer(Shape::ConstantsOnly);
    let values: Vec<_> = owner.constants().cloned().collect();
    let input = fixture(90, &values, None, Usage::Read);
    let package = project_c_package(input.registry, input.files).unwrap();
    for fault in [Fault::None, Fault::Missing, Fault::Swap, Fault::Coupled] {
        let mut builder = TargetAstBuilder::new(CDialect);
        for value in package.generated_types() {
            builder.generated_type(value.clone());
        }
        for value in package.callables() {
            builder.callable(value.clone());
        }
        for value in package.values() {
            builder.value(value.clone());
        }
        for file in package.files() {
            let mut units = file.items().to_vec();
            if !matches!(file.source_kind(), CFileGrammar::Header(_)) {
                change(&mut Arc::make_mut(&mut units[0].data).bindings, fault);
            }
            if matches!(fault, Fault::Coupled) {
                change(&mut Arc::make_mut(&mut units[0].projection).bindings, fault);
            }
            builder.file(TargetFile::new(
                file.path().clone(),
                file.role(),
                file.module().clone(),
                *file.placement(),
                units,
                file.source_kind().clone(),
                file.source().clone(),
            ));
        }
        for group in package.groups() {
            builder.group(group.clone());
        }
        assert_eq!(
            verify_unresolved_package(&CDialect, builder.build()).is_ok(),
            matches!(fault, Fault::None),
            "{fault:?}"
        );
    }
}

#[test]
fn imported_places_reject_writes_and_the_shared_profile_rejects_address_borrows() {
    let owner = producer(Shape::ConstantsOnly);
    let value = owner.constants().next().unwrap().clone();
    let mut input = fixture(90, std::slice::from_ref(&value), None, Usage::Read);
    let registry = input.registry.registrations();
    let expressions = CExpressions::new(registry);
    let place = expressions.global(input.objects[0].clone()).unwrap();
    let statements = CStatements::new(registry, input.functions[0].clone()).unwrap();
    assert!(
        statements
            .assign(
                place.clone(),
                expressions.literal(value.value().clone()).unwrap()
            )
            .is_err()
    );
    let CFileItem::Definition(definition) = &input.files[1].items()[0] else {
        panic!("definition");
    };
    let CDefinitionKind::Function { body, .. } = definition.kind() else {
        panic!("function");
    };
    let mut actions = body.statements().to_vec();
    actions.insert(
        0,
        statements
            .discard(expressions.address_of(place).unwrap())
            .unwrap(),
    );
    let body = statements.block(body.scope().clone(), actions).unwrap();
    let builder = CDeclarations::new(registry, input.files[1].identity().clone()).unwrap();
    input.files[1] = builder
        .source_file(vec![CFileItem::Definition(
            builder
                .function_definition(input.functions[0].clone(), CLinkage::External, vec![], body)
                .unwrap(),
        )])
        .unwrap();
    assert!(project_c_package(input.registry, input.files).is_err());
}
