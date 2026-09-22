//! Constructors and whole-package proof reject unsupported global shapes.
use super::{
    owned_constant_fixture::{Fixture, Shape, fixture},
    project_c_package,
};
use crate::ast::*;

#[test]
fn mutable_unsigned_and_aggregate_objects_stay_outside_the_profile() {
    let scalar = CObjectType::scalar;
    let cases = [
        scalar(CScalarType::Bool),
        scalar(CScalarType::I32),
        scalar(CScalarType::I64),
        scalar(CScalarType::F64),
        scalar(CScalarType::U32)
            .with_constness(CConstness::Const)
            .unwrap(),
        CObjectType::array(
            scalar(CScalarType::I32)
                .with_constness(CConstness::Const)
                .unwrap(),
            CArrayLength::new(1).unwrap(),
        )
        .unwrap(),
    ];
    for ty in cases {
        let mut registry = CRegistry::new();
        let header = registry
            .register_file(CFileKey {
                path: portable_codegen::RelativeOutputPath::new("polyrust_rejected.h").unwrap(),
                role: CFileRole::GeneratedPublicHeader,
            })
            .unwrap();
        let source = registry
            .register_file(CFileKey {
                path: portable_codegen::RelativeOutputPath::new("rejected.c").unwrap(),
                role: CFileRole::GeneratedSource,
            })
            .unwrap();
        let object = registry
            .register_object(
                &header,
                super::owned_constant_fixture::key("rejected"),
                ty.clone(),
            )
            .unwrap();
        let declarations = CDeclarations::new(&registry, header).unwrap();
        let definitions = CDeclarations::new(&registry, source).unwrap();
        let initializer = CExpressions::new(&registry)
            .zero_initializer(ty.clone())
            .unwrap();
        let files = vec![
            declarations
                .source_file(vec![CFileItem::Declaration(
                    declarations.object_declaration(object.clone()).unwrap(),
                )])
                .unwrap(),
            definitions
                .source_file(vec![CFileItem::Definition(
                    definitions
                        .object_definition(object, CLinkage::External, initializer)
                        .unwrap(),
                )])
                .unwrap(),
        ];
        // Assert the object-shape diagnostic, not just the separate zero-initializer restriction.
        let errors = project_c_package(registry.freeze(), files).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("const bool/i32/i64")),
            "{ty:?}: {errors:?}"
        );
    }
}
fn replace(fixture: &mut Fixture, index: usize, items: Vec<CFileItem>) {
    fixture.files[index] = CDeclarations::new(
        fixture.registry.registrations(),
        fixture.files[index].identity().clone(),
    )
    .unwrap()
    .source_file(items)
    .unwrap();
}

#[derive(Clone, Copy, Debug)]
enum InventoryMutation {
    MissingDeclaration,
    DuplicateDeclaration,
    MissingDefinition,
    DuplicateDefinition,
}

#[test]
fn every_owned_object_needs_one_primary_declaration_and_one_definition() {
    for shape in [Shape::ConstantsOnly, Shape::Mixed] {
        for mutation in [
            InventoryMutation::MissingDeclaration,
            InventoryMutation::DuplicateDeclaration,
            InventoryMutation::MissingDefinition,
            InventoryMutation::DuplicateDefinition,
        ] {
            let mut fixture = fixture(shape);
            let index = match mutation {
                InventoryMutation::MissingDeclaration | InventoryMutation::DuplicateDeclaration => {
                    0
                }
                InventoryMutation::MissingDefinition | InventoryMutation::DuplicateDefinition => 1,
            };
            let mut items = fixture.files[index].items().to_vec();
            match mutation {
                InventoryMutation::MissingDeclaration | InventoryMutation::MissingDefinition => {
                    items.remove(0);
                }
                InventoryMutation::DuplicateDeclaration
                | InventoryMutation::DuplicateDefinition => items.push(items[0].clone()),
            }
            replace(&mut fixture, index, items);
            assert!(
                project_c_package(fixture.registry, fixture.files).is_err(),
                "{shape:?}: {mutation:?}"
            );
        }
    }
}

#[test]
fn object_constructors_reject_wrong_registry_type_file_and_linkage() {
    let fixture = fixture(Shape::ConstantsOnly);
    let foreign = super::owned_constant_fixture::fixture(Shape::ConstantsOnly);
    let registry = fixture.registry.registrations();
    let expressions = CExpressions::new(registry);
    let declarations = CDeclarations::new(registry, fixture.files[0].identity().clone()).unwrap();
    let definitions = CDeclarations::new(registry, fixture.files[1].identity().clone()).unwrap();
    let initializer = expressions
        .expression_initializer(expressions.literal(CLiteral::Bool(false)).unwrap())
        .unwrap();
    assert!(
        definitions
            .object_definition(
                fixture.objects[0].clone(),
                CLinkage::External,
                initializer.clone()
            )
            .is_ok()
    );
    assert!(
        definitions
            .object_declaration(fixture.objects[0].clone())
            .is_err()
    );
    assert!(
        declarations
            .object_definition(
                fixture.objects[0].clone(),
                CLinkage::External,
                initializer.clone()
            )
            .is_err()
    );
    for linkage in [CLinkage::Internal, CLinkage::None] {
        assert!(
            definitions
                .object_definition(fixture.objects[0].clone(), linkage, initializer.clone())
                .is_err()
        );
    }
    assert!(
        definitions
            .object_definition(
                fixture.objects[2].clone(),
                CLinkage::External,
                initializer.clone()
            )
            .is_err()
    );
    assert!(
        definitions
            .object_definition(foreign.objects[0].clone(), CLinkage::External, initializer)
            .is_err()
    );
    assert!(expressions.global(foreign.objects[0].clone()).is_err());
}

#[test]
fn zero_and_computed_global_initializers_do_not_enter_the_literal_profile() {
    let mut fixture = fixture(Shape::Mixed);
    let registry = fixture.registry.registrations();
    let expressions = CExpressions::new(registry);
    let declarations = CDeclarations::new(registry, fixture.files[1].identity().clone()).unwrap();
    let literal = || {
        expressions
            .literal(CLiteral::Signed(CSignedLiteral::I32(1)))
            .unwrap()
    };
    let alternatives = [
        expressions
            .zero_initializer(fixture.objects[7].ty().clone())
            .unwrap(),
        expressions
            .expression_initializer(
                expressions
                    .binary(CBinaryOperator::Add, literal(), literal())
                    .unwrap(),
            )
            .unwrap(),
    ];
    let originals = fixture.files[1].items().to_vec();
    let replacements: Vec<_> = alternatives
        .into_iter()
        .map(|initializer| {
            declarations
                .object_definition(fixture.objects[7].clone(), CLinkage::External, initializer)
                .unwrap()
        })
        .collect();
    for definition in replacements {
        let mut items = originals.clone();
        items[7] = CFileItem::Definition(definition);
        replace(&mut fixture, 1, items);
        assert!(project_c_package(fixture.registry.clone(), fixture.files.clone()).is_err());
    }
}

#[test]
fn address_borrowing_and_assignment_of_global_constants_are_rejected() {
    let mut fixture = fixture(Shape::Mixed);
    let expressions = CExpressions::new(fixture.registry.registrations());
    let function = fixture.functions[0].clone();
    let statements = CStatements::new(fixture.registry.registrations(), function).unwrap();
    let place = expressions.global(fixture.objects[0].clone()).unwrap();
    assert!(
        statements
            .assign(
                place.clone(),
                expressions.literal(CLiteral::Bool(false)).unwrap()
            )
            .is_err()
    );
    let mut items = fixture.files[1].items().to_vec();
    let CFileItem::Definition(definition) = &items[8] else {
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
    let mut body_items = body.statements().to_vec();
    body_items.insert(
        0,
        statements
            .discard(expressions.address_of(place).unwrap())
            .unwrap(),
    );
    let body = statements.block(body.scope().clone(), body_items).unwrap();
    let definitions = CDeclarations::new(
        fixture.registry.registrations(),
        fixture.files[1].identity().clone(),
    )
    .unwrap();
    items[8] = CFileItem::Definition(
        definitions
            .function_definition(function.clone(), *linkage, parameters.clone(), body)
            .unwrap(),
    );
    replace(&mut fixture, 1, items);
    assert!(project_c_package(fixture.registry, fixture.files).is_err());
}
