//! Address formation reads operands but does not grant pointee initialization.
use super::contextual_reconstruction::{fixture, int, key, package};
use super::*;

#[test]
fn address_formation_requires_initialized_pointer_and_index_operands() {
    for indexed in [false, true] {
        for initialized in [false, true] {
            let (mut registry, file, function, scope) = fixture();
            let scalar = CObjectType::scalar(CScalarType::I32);
            let pointer = CObjectType::pointer(CPointerTarget::Object(Box::new(scalar.clone())));
            let operand_type = if indexed {
                CObjectType::scalar(CScalarType::Size)
            } else {
                pointer.clone()
            };
            let operand = registry
                .register_local(&scope, key("operand"), operand_type)
                .unwrap();
            let array = indexed.then(|| {
                registry
                    .register_local(
                        &scope,
                        key("storage"),
                        CObjectType::array(scalar, CArrayLength::new(1).unwrap()).unwrap(),
                    )
                    .unwrap()
            });
            let values = CExpressions::new(&registry);
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            let operand_value = values.read(values.local(operand.clone()).unwrap()).unwrap();
            let zero = if indexed {
                values
                    .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
                    .unwrap()
            } else {
                values
                    .literal(CLiteral::NullPointer(CNullPointer::new(pointer).unwrap()))
                    .unwrap()
            };
            let mut body = vec![
                ast.declare(
                    operand,
                    initialized.then(|| values.expression_initializer(zero).unwrap()),
                )
                .unwrap(),
            ];
            let place = if let Some(array) = array {
                body.push(ast.declare(array.clone(), None).unwrap());
                values
                    .index(
                        CIndexBase::Array(Box::new(values.local(array).unwrap())),
                        operand_value,
                    )
                    .unwrap()
            } else {
                values.dereference(operand_value).unwrap()
            };
            body.push(ast.discard(values.address_of(place).unwrap()).unwrap());
            assert_eq!(
                registry.check_context(&[package(&registry, file, function, scope, body)]),
                if initialized {
                    Ok(())
                } else {
                    Err(CContextError::UninitializedRead)
                }
            );
        }
    }
}

#[test]
fn an_unproved_call_cannot_initialize_a_local_merely_because_its_address_is_passed() {
    for initialized in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let scalar = CObjectType::scalar(CScalarType::I32);
        let local = registry
            .register_local(&scope, key("slot"), scalar.clone())
            .unwrap();
        let callee = registry
            .register_function(
                &file,
                key("observe"),
                CFunctionType::new(
                    CReturnType::Void,
                    vec![
                        CParameterType::new(CObjectType::pointer(CPointerTarget::Object(
                            Box::new(scalar),
                        )))
                        .unwrap(),
                    ],
                ),
            )
            .unwrap();
        let argument = registry
            .register_parameter(&callee, 0, key("pointer"), CConstness::Unqualified)
            .unwrap();
        let callee_scope = registry.register_scope(&callee, None, key("root")).unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let place = values.local(local.clone()).unwrap();
        let effect = values
            .call_effect(
                values.direct(callee.clone()).unwrap(),
                vec![values.address_of(place.clone()).unwrap()],
            )
            .unwrap();
        let body = vec![
            ast.declare(
                local,
                initialized.then(|| values.expression_initializer(int(&values)).unwrap()),
            )
            .unwrap(),
            ast.evaluate(effect).unwrap(),
            ast.discard(values.read(place).unwrap()).unwrap(),
        ];
        let mut source = package(&registry, file.clone(), function, scope, body);
        let callee_body = CStatements::new(&registry, callee.clone())
            .unwrap()
            .block(callee_scope, vec![])
            .unwrap();
        source.items.push(CFileItem::Definition(
            CDeclarations::new(&registry, file)
                .unwrap()
                .function_definition(callee, CLinkage::External, vec![argument], callee_body)
                .unwrap(),
        ));
        assert_eq!(
            registry.check_context(&[source]),
            if initialized {
                Ok(())
            } else {
                Err(CContextError::UninitializedRead)
            }
        );
    }
}
