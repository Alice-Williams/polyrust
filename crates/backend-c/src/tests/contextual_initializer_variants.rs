//! All initializer branches are independently rebuilt from their real children.
use super::contextual_reconstruction::{fixture, int, key, package};
use super::*;

#[test]
fn initializer_variants_reject_cached_type_and_child_shape_corruption() {
    let (mut registry, file, function, scope) = fixture();
    let scalar = CObjectType::scalar(CScalarType::I32);
    let record = registry.declare_struct(&file, key("Record")).unwrap();
    let field = registry
        .register_member(
            &CAggregateRef::Struct(record.clone()),
            key("field"),
            scalar.clone(),
        )
        .unwrap();
    registry
        .define_aggregate(&CAggregateRef::Struct(record.clone()), vec![field.clone()])
        .unwrap();
    let union = registry.declare_union(&file, key("Payload")).unwrap();
    let variant = registry
        .register_member(
            &CAggregateRef::Union(union.clone()),
            key("field"),
            scalar.clone(),
        )
        .unwrap();
    registry
        .define_aggregate(&CAggregateRef::Union(union.clone()), vec![variant.clone()])
        .unwrap();
    let array = CObjectType::array(scalar.clone(), CArrayLength::new(2).unwrap()).unwrap();
    let types = [
        scalar.clone(),
        scalar.clone(),
        array.clone(),
        CObjectType::structure(record.clone()),
        CObjectType::union(union.clone()),
    ];
    let locals = types
        .into_iter()
        .enumerate()
        .map(|(i, ty)| {
            registry
                .register_local(&scope, key(&format!("local_{i}")), ty)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let values = CExpressions::new(&registry);
    let leaf = values.expression_initializer(int(&values)).unwrap();
    let initializers = [
        leaf.clone(),
        values.zero_initializer(scalar).unwrap(),
        values
            .array_initializer(array, vec![leaf.clone(), leaf.clone()])
            .unwrap(),
        values
            .struct_initializer(record, vec![(field, leaf.clone())])
            .unwrap(),
        values.union_initializer(union, variant, leaf).unwrap(),
    ];
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    for (local, initializer) in locals.into_iter().zip(initializers) {
        let source = package(
            &registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            vec![ast.declare(local, Some(initializer)).unwrap()],
        );
        registry
            .check_local_structure(std::slice::from_ref(&source))
            .unwrap();
        for corrupt_cache in [false, true] {
            let mut bad = source.clone();
            let CFileItem::Definition(definition) = &mut bad.items[0] else {
                unreachable!()
            };
            let CDefinitionKind::Function { body, .. } = &mut definition.kind else {
                unreachable!()
            };
            let CStatementKind::Declare(declaration) = &mut body.statements[0].kind else {
                unreachable!()
            };
            let initializer = declaration.initializer.as_mut().unwrap();
            if corrupt_cache {
                initializer.ty = CObjectType::scalar(CScalarType::F64);
            } else {
                match &mut initializer.kind {
                    CInitializerKind::Expression(value) => {
                        value.ty = CObjectType::scalar(CScalarType::F64)
                    }
                    CInitializerKind::Zero(ty) => *ty = CObjectType::scalar(CScalarType::F64),
                    CInitializerKind::Array { elements, .. } => {
                        elements.pop();
                    }
                    CInitializerKind::Struct { members, .. } => {
                        members.clear();
                    }
                    CInitializerKind::Union { value, .. } => {
                        value.ty = CObjectType::scalar(CScalarType::F64)
                    }
                }
            }
            assert!(registry.check_local_structure(&[bad]).is_err());
        }
    }
}
