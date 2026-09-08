//! Complete-object dependencies must traverse declared aliases, not names.

use super::contextual_reconstruction::{fixture, key, package};
use super::*;

#[test]
fn aliases_cannot_hide_mutual_by_value_cycles_but_recursive_pointers_are_legal() {
    for pointer in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let a = registry.declare_struct(&file, key("A")).unwrap();
        let b = registry.declare_struct(&file, key("B")).unwrap();
        let mut aliases = Vec::new();
        for record in [&a, &b] {
            let ty = CObjectType::structure(record.clone());
            let ty = if pointer {
                CObjectType::pointer(CPointerTarget::Object(Box::new(ty)))
            } else {
                ty
            };
            aliases.push(
                registry
                    .register_typedef(
                        &file,
                        key(if record == &a { "AliasA" } else { "AliasB" }),
                        ty,
                    )
                    .unwrap(),
            );
        }
        for (record, alias) in [(&a, &aliases[1]), (&b, &aliases[0])] {
            let owner = CAggregateRef::Struct(record.clone());
            let member = registry
                .register_member(&owner, key("next"), CObjectType::typedef(alias.clone()))
                .unwrap();
            registry.define_aggregate(&owner, vec![member]).unwrap();
        }
        let mut source = package(&registry, file.clone(), function, scope, vec![]);
        let ast = CDeclarations::new(&registry, file).unwrap();
        for alias in aliases {
            source
                .items
                .push(CFileItem::Declaration(ast.typedef(alias).unwrap()));
        }
        for record in [a, b] {
            source.items.push(CFileItem::Declaration(
                ast.aggregate(CAggregateRef::Struct(record)).unwrap(),
            ));
        }
        assert_eq!(
            registry.check_context(&[source]),
            if pointer {
                Ok(())
            } else {
                Err(CContextError::RecursiveObject)
            }
        );
    }
}

#[test]
fn alias_hidden_incomplete_storage_is_rejected_independently_of_layout_cycles() {
    for complete in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let record = registry.declare_struct(&file, key("Opaque")).unwrap();
        let owner = CAggregateRef::Struct(record.clone());
        if complete {
            let member = registry
                .register_member(&owner, key("field"), CObjectType::scalar(CScalarType::I32))
                .unwrap();
            registry.define_aggregate(&owner, vec![member]).unwrap();
        }
        let alias = registry
            .register_typedef(&file, key("Alias"), CObjectType::structure(record))
            .unwrap();
        let ty = CObjectType::typedef(alias.clone());
        let local = registry
            .register_local(&scope, key("storage"), ty.clone())
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let mut source = package(
            &registry,
            file.clone(),
            function,
            scope,
            vec![
                ast.declare(local, Some(values.zero_initializer(ty).unwrap()))
                    .unwrap(),
            ],
        );
        let declarations = CDeclarations::new(&registry, file).unwrap();
        source
            .items
            .push(CFileItem::Declaration(declarations.typedef(alias).unwrap()));
        source.items.push(CFileItem::Declaration(if complete {
            declarations.aggregate(owner).unwrap()
        } else {
            declarations.forward_tag(owner).unwrap()
        }));
        assert_eq!(
            registry.check_context(&[source]),
            if complete {
                Ok(())
            } else {
                Err(CContextError::IncompleteObject)
            }
        );
    }
}
