//! File declaration/definition projections and static-object initializer checks.
use super::contextual_reconstruction::{fixture, int, key};
use super::*;

#[test]
fn file_initializer_rederives_constancy_after_a_same_typed_child_replacement() {
    for array in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let scalar = CObjectType::scalar(CScalarType::I32);
        let input = registry
            .register_object(&file, key("input"), scalar.clone())
            .unwrap();
        let ty = if array {
            CObjectType::array(scalar, CArrayLength::new(1).unwrap()).unwrap()
        } else {
            scalar
        };
        let output = registry
            .register_object(&file, key("output"), ty.clone())
            .unwrap();
        let values = CExpressions::new(&registry);
        let leaf = values.expression_initializer(int(&values)).unwrap();
        let initializer = if array {
            values.array_initializer(ty, vec![leaf]).unwrap()
        } else {
            leaf
        };
        let replacement = values.read(values.global(input.clone()).unwrap()).unwrap();
        let declarations = CDeclarations::new(&registry, file.clone()).unwrap();
        let mut source =
            super::contextual_reconstruction::package(&registry, file, function, scope, vec![]);
        source.items.push(CFileItem::Definition(
            declarations
                .object_definition(
                    input,
                    CLinkage::External,
                    values.expression_initializer(int(&values)).unwrap(),
                )
                .unwrap(),
        ));
        source.items.push(CFileItem::Definition(
            declarations
                .object_definition(output, CLinkage::External, initializer)
                .unwrap(),
        ));
        registry
            .check_context(std::slice::from_ref(&source))
            .unwrap();
        let CFileItem::Definition(definition) = source.items.last_mut().unwrap() else {
            unreachable!()
        };
        let CDefinitionKind::Object { initializer, .. } = &mut definition.kind else {
            unreachable!()
        };
        let leaf = if let CInitializerKind::Array { elements, .. } = &mut initializer.kind {
            &mut elements[0]
        } else {
            initializer
        };
        let CInitializerKind::Expression(value) = &mut leaf.kind else {
            unreachable!()
        };
        assert_eq!(value.ty(), replacement.ty());
        *value = replacement;
        assert_eq!(
            registry.check_context(&[source]),
            Err(CContextError::File(CFileError::ExpectedStaticInitializer))
        );
    }
}

#[test]
fn every_declaration_kind_rechecks_its_actual_file_and_nominal_projection() {
    let (mut registry, file, function, _) = fixture();
    let structure = registry.declare_struct(&file, key("Record")).unwrap();
    let owner = CAggregateRef::Struct(structure);
    let field = registry
        .register_member(&owner, key("field"), CObjectType::scalar(CScalarType::I32))
        .unwrap();
    registry.define_aggregate(&owner, vec![field]).unwrap();
    let enumeration = registry.declare_enum(&file, key("Choice")).unwrap();
    let zero = registry
        .register_enumerator(&enumeration, key("Zero"), 0)
        .unwrap();
    registry.define_enum(&enumeration, vec![zero]).unwrap();
    let alias = registry
        .register_typedef(&file, key("Alias"), CObjectType::scalar(CScalarType::I32))
        .unwrap();
    let object = registry
        .register_object(&file, key("global"), CObjectType::scalar(CScalarType::I32))
        .unwrap();
    let ast = CDeclarations::new(&registry, file).unwrap();
    let declarations = [
        ast.forward_tag(owner.clone()).unwrap(),
        ast.aggregate(owner).unwrap(),
        ast.enumeration(enumeration).unwrap(),
        ast.typedef(alias).unwrap(),
        ast.function_prototype(function, CLinkage::External)
            .unwrap(),
        ast.object_declaration(object).unwrap(),
    ];
    for declaration in declarations {
        let source = ast
            .source_file(vec![CFileItem::Declaration(declaration.clone())])
            .unwrap();
        registry
            .check_local_structure(std::slice::from_ref(&source))
            .unwrap();
        let mut bad = source;
        let CFileItem::Declaration(value) = &mut bad.items[0] else {
            unreachable!()
        };
        value.file = fixture().1;
        assert!(registry.check_local_structure(&[bad]).is_err());
        if let CDeclarationKind::Enum { .. } = declaration.kind() {
            let mut bad = declaration;
            let CDeclarationKind::Enum { values, .. } = &mut bad.kind else {
                unreachable!()
            };
            values.clear();
            assert_eq!(
                registry.check_local_structure(&[ast
                    .source_file(vec![CFileItem::Declaration(bad)])
                    .unwrap()]),
                Err(CContextError::StoredStructureMismatch)
            );
        }
    }
}

#[test]
fn static_object_definition_rechecks_initializer_and_extern_linkage() {
    for internal in [false, true] {
        let mut registry = CRegistry::new();
        let file = registry
            .register_file(CFileKey {
                path: portable_codegen::RelativeOutputPath::new("object.c").unwrap(),
                role: CFileRole::TestSource,
            })
            .unwrap();
        let object = registry
            .register_object(&file, key("global"), CObjectType::scalar(CScalarType::I32))
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CDeclarations::new(&registry, file).unwrap();
        let source = ast
            .source_file(vec![
                CFileItem::Declaration(ast.object_declaration(object.clone()).unwrap()),
                CFileItem::Definition(
                    ast.object_definition(
                        object,
                        if internal {
                            CLinkage::Internal
                        } else {
                            CLinkage::External
                        },
                        values.expression_initializer(int(&values)).unwrap(),
                    )
                    .unwrap(),
                ),
            ])
            .unwrap();
        assert_eq!(
            registry.check_context(std::slice::from_ref(&source)),
            if internal {
                Err(CContextError::LinkageMismatch)
            } else {
                Ok(())
            }
        );
        let mut bad = source;
        let CFileItem::Definition(value) = &mut bad.items[1] else {
            unreachable!()
        };
        let CDefinitionKind::Object { initializer, .. } = &mut value.kind else {
            unreachable!()
        };
        let CInitializerKind::Expression(value) = &mut initializer.kind else {
            unreachable!()
        };
        value.ty = CObjectType::scalar(CScalarType::F64);
        assert!(registry.check_local_structure(&[bad]).is_err());
    }
}
