//! Completeness is required by actual object uses, not ordinary pointer types.

use super::contextual_reconstruction::{fixture, key, package};
use super::*;
use portable_codegen::RelativeOutputPath;

#[test]
fn allocation_storage_requires_complete_object_even_behind_a_restored_pointer() {
    for complete in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let record = registry.declare_struct(&file, key("Record")).unwrap();
        let owner = CAggregateRef::Struct(record.clone());
        if complete {
            let member = registry
                .register_member(&owner, key("value"), CObjectType::scalar(CScalarType::I32))
                .unwrap();
            registry.define_aggregate(&owner, vec![member]).unwrap();
        }
        let allocation = registry
            .register_allocation(
                &scope,
                key("allocation"),
                CObjectType::structure(record),
                CAllocatorSource::Default,
            )
            .unwrap();
        let values = CExpressions::new(&registry);
        let operand = values
            .literal(CLiteral::NullPointer(
                CNullPointer::new(CObjectType::pointer(CPointerTarget::Void(
                    CConstness::Unqualified,
                )))
                .unwrap(),
            ))
            .unwrap();
        let restored = values.allocation_restore(allocation, operand).unwrap();
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let mut source = package(
            &registry,
            file.clone(),
            function,
            scope,
            vec![ast.discard(restored).unwrap()],
        );
        let declarations = CDeclarations::new(&registry, file).unwrap();
        source.items.insert(
            0,
            CFileItem::Declaration(if complete {
                declarations.aggregate(owner).unwrap()
            } else {
                declarations.forward_tag(owner).unwrap()
            }),
        );
        let result = registry.check_context(&[source]);
        if complete {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CContextError::IncompleteObject));
        }
    }
}

#[derive(Clone, Copy)]
enum Use {
    Read,
    Index,
    AddressOfDereference,
}

#[test]
fn nested_function_prototypes_allow_incomplete_by_value_parameter_and_return_types() {
    let (mut registry, file, function, scope) = fixture();
    let record = registry.declare_struct(&file, key("Opaque")).unwrap();
    let ty = CObjectType::structure(record.clone());
    let signature = CFunctionType::new(
        CReturnType::Value(CReturnValue::new(ty.clone()).unwrap()),
        vec![CParameterType::new(ty).unwrap()],
    );
    let pointer = CObjectType::pointer(CPointerTarget::Function(Box::new(signature)));
    let alias = registry
        .register_typedef(&file, key("Callback"), pointer.clone())
        .unwrap();
    let values = CExpressions::new(&registry);
    let null = values
        .literal(CLiteral::NullPointer(CNullPointer::new(pointer).unwrap()))
        .unwrap();
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let mut source = package(
        &registry,
        file.clone(),
        function,
        scope,
        vec![statements.discard(null).unwrap()],
    );
    let declarations = CDeclarations::new(&registry, file).unwrap();
    source.items.insert(
        0,
        CFileItem::Declaration(
            declarations
                .forward_tag(CAggregateRef::Struct(record))
                .unwrap(),
        ),
    );
    source.items.insert(
        1,
        CFileItem::Declaration(declarations.typedef(alias).unwrap()),
    );
    registry.check_context(&[source]).unwrap();
}

#[test]
fn expression_only_pointer_array_types_require_complete_elements_in_all_syntax() {
    for complete in [false, true] {
        for array in [false, true] {
            for skipped in [false, true] {
                let (mut registry, file, function, scope) = fixture();
                let record = registry.declare_struct(&file, key("Opaque")).unwrap();
                let owner = CAggregateRef::Struct(record.clone());
                if complete {
                    let member = registry
                        .register_member(
                            &owner,
                            key("field"),
                            CObjectType::scalar(CScalarType::I32),
                        )
                        .unwrap();
                    registry.define_aggregate(&owner, vec![member]).unwrap();
                }
                let element = CObjectType::structure(record);
                let target = if array {
                    CObjectType::array(element, CArrayLength::new(1).unwrap()).unwrap()
                } else {
                    element
                };
                let pointer = CObjectType::pointer(CPointerTarget::Object(Box::new(target)));
                let values = CExpressions::new(&registry);
                let null = values
                    .literal(CLiteral::NullPointer(CNullPointer::new(pointer).unwrap()))
                    .unwrap();
                let statements = CStatements::new(&registry, function.clone()).unwrap();
                let mut body = Vec::new();
                if skipped {
                    body.push(statements.return_statement(None).unwrap());
                }
                body.push(statements.discard(null).unwrap());
                let mut source = package(&registry, file.clone(), function, scope, body);
                let declarations = CDeclarations::new(&registry, file).unwrap();
                source.items.insert(
                    0,
                    CFileItem::Declaration(if complete {
                        declarations.aggregate(owner).unwrap()
                    } else {
                        declarations.forward_tag(owner).unwrap()
                    }),
                );
                assert_eq!(
                    registry.check_context(&[source]),
                    if array && !complete {
                        Err(CContextError::IncompleteObject)
                    } else {
                        Ok(())
                    },
                    "complete={complete}, array={array}, skipped={skipped}"
                );
            }
        }
    }
}

#[test]
fn actual_incomplete_reads_and_pointer_indices_fail_but_address_cancellation_is_legal() {
    for complete in [false, true] {
        for usage in [Use::Read, Use::Index, Use::AddressOfDereference] {
            let mut registry = CRegistry::new();
            let file = registry
                .register_file(CFileKey {
                    role: CFileRole::TestSource,
                    path: RelativeOutputPath::new("tests/use.c").unwrap(),
                })
                .unwrap();
            let record = registry.declare_struct(&file, key("Opaque")).unwrap();
            let owner = CAggregateRef::Struct(record.clone());
            if complete {
                let member = registry
                    .register_member(&owner, key("value"), CObjectType::scalar(CScalarType::I32))
                    .unwrap();
                registry.define_aggregate(&owner, vec![member]).unwrap();
            }
            let pointer_type = CObjectType::pointer(CPointerTarget::Object(Box::new(
                CObjectType::structure(record),
            )));
            let function = registry
                .register_function(
                    &file,
                    key("use_pointer"),
                    CFunctionType::new(
                        CReturnType::Void,
                        vec![CParameterType::new(pointer_type).unwrap()],
                    ),
                )
                .unwrap();
            let parameter = registry
                .register_parameter(&function, 0, key("pointer"), CConstness::Unqualified)
                .unwrap();
            let scope = registry
                .register_scope(&function, None, key("body"))
                .unwrap();
            let values = CExpressions::new(&registry);
            let pointer = values
                .read(values.parameter(parameter.clone()).unwrap())
                .unwrap();
            let value = match usage {
                Use::Read => values.read(values.dereference(pointer).unwrap()).unwrap(),
                Use::Index => values
                    .address_of(
                        values
                            .index(
                                CIndexBase::Pointer(Box::new(pointer)),
                                values
                                    .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
                                    .unwrap(),
                            )
                            .unwrap(),
                    )
                    .unwrap(),
                Use::AddressOfDereference => values
                    .address_of(values.dereference(pointer).unwrap())
                    .unwrap(),
            };
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            let body = ast.block(scope, vec![ast.discard(value).unwrap()]).unwrap();
            let declarations = CDeclarations::new(&registry, file).unwrap();
            let declaration = if complete {
                declarations.aggregate(owner).unwrap()
            } else {
                declarations.forward_tag(owner).unwrap()
            };
            let definition = declarations
                .function_definition(function, CLinkage::External, vec![parameter], body)
                .unwrap();
            let source = declarations
                .source_file(vec![
                    CFileItem::Declaration(declaration),
                    CFileItem::Definition(definition),
                ])
                .unwrap();
            let result = registry.check_context(&[source]);
            if complete || matches!(usage, Use::AddressOfDereference) {
                result.unwrap();
            } else {
                assert_eq!(result, Err(CContextError::IncompleteObject));
            }
        }
    }
}
