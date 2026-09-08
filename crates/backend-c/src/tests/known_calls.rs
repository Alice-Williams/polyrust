//! Every catalogue row has independent type, payload and rejection controls.
use super::contextual_reconstruction::{fixture, key};
use super::known_call_fixtures::rows;
use super::*;
use crate::dialect::{CKnownCall, CKnownOperands as O};

#[test]
fn catalogue_inventory_signatures_and_actual_operand_roles_are_exact() {
    let rows = rows();
    assert_eq!(
        rows.iter().map(|row| row.call).collect::<Vec<_>>(),
        CKnownCall::ALL
    );
    let (registry, _, _, _) = fixture();
    let ast = CExpressions::new(&registry);
    for row in rows {
        assert_eq!(row.call.spelling(), row.spelling);
        assert_eq!(row.call.header(), row.header);
        assert_eq!(row.call.system_library(), row.library);
        assert_eq!(row.call.form(), row.form);
        assert_eq!(row.call.signature(), row.signature());
        let callable = ast.known(row.call);
        assert_eq!(*callable.signature(), row.signature());
        assert_eq!(callable.kind(), &CCallableKind::Known(row.call));
        let arguments = row.arguments(&ast);
        let call = row.call(&ast, callable.clone(), arguments.clone()).unwrap();
        assert_eq!(call.callable(), &callable);
        assert_eq!(call.arguments(), arguments);
        use CKnownCall as K;
        let a = call.arguments();
        let expected = match row.call {
            K::Allocate => O::Allocate { bytes: &a[0] },
            K::Release => O::Release { base: &a[0] },
            K::CopyBytes => O::CopyBytes {
                destination: &a[0],
                source: &a[1],
                bytes: &a[2],
            },
            K::CompareBytes => O::CompareBytes {
                left: &a[0],
                right: &a[1],
                bytes: &a[2],
            },
            K::FloatRemainder => O::FloatRemainder {
                dividend: &a[0],
                divisor: &a[1],
            },
            K::FloatTruncate => O::FloatTruncate { value: &a[0] },
            K::IsNan => O::IsNan { value: &a[0] },
            K::SignBit => O::SignBit { value: &a[0] },
            K::WriteBytes => O::WriteBytes {
                source: &a[0],
                element_size: &a[1],
                count: &a[2],
                stream: &a[3],
            },
            K::StreamError => O::StreamError { stream: &a[0] },
        };
        assert_eq!(call.contract(), CCallContract::Known(expected));
    }
}

#[test]
fn known_calls_reject_wrong_arity_each_argument_crossed_brands_and_result_category() {
    let (registry, _, function, _) = fixture();
    let (foreign, _, _, _) = fixture();
    let ast = CExpressions::new(&registry);
    let other = CExpressions::new(&foreign);
    for row in rows() {
        let callable = ast.known(row.call);
        let arguments = row.arguments(&ast);
        for count in [arguments.len() - 1, arguments.len() + 1] {
            let mut bad = arguments.clone();
            bad.resize(count, arguments[0].clone());
            assert_eq!(
                row.call(&ast, callable.clone(), bad),
                Err(CExpressionError::ArityMismatch {
                    expected: arguments.len(),
                    actual: count
                })
            );
        }
        for index in 0..arguments.len() {
            let mut bad = arguments.clone();
            bad[index] = ast.literal(CLiteral::Bool(true)).unwrap();
            assert_eq!(
                row.call(&ast, callable.clone(), bad),
                Err(CExpressionError::TypeMismatch)
            );
            let mut bad = arguments.clone();
            bad[index] = row.arguments(&other)[index].clone();
            assert_eq!(
                row.call(&ast, callable.clone(), bad),
                Err(CExpressionError::Registry(CRegistryError::CrossRegistry))
            );
        }
        assert_eq!(
            row.call(&ast, other.known(row.call), arguments.clone()),
            Err(CExpressionError::Registry(CRegistryError::CrossRegistry))
        );
        if row.result.is_none() {
            assert_eq!(
                ast.call_value(callable, arguments),
                Err(CExpressionError::ExpectedValueCall)
            );
            let effect = other
                .call_effect(other.known(row.call), row.arguments(&other))
                .unwrap();
            assert_eq!(
                CStatements::new(&registry, function.clone())
                    .unwrap()
                    .evaluate(effect),
                Err(CStatementError::Expression(CExpressionError::Registry(
                    CRegistryError::CrossRegistry
                )))
            );
        } else {
            assert_eq!(
                ast.call_effect(callable, arguments),
                Err(CExpressionError::ExpectedEffectCall)
            );
        }
    }
}

#[test]
fn generated_matching_names_and_prototypes_never_gain_library_contracts() {
    for row in rows() {
        let (mut registry, file, _, _) = fixture();
        let function = registry
            .register_function(&file, key(row.spelling), row.signature())
            .unwrap();
        let ast = CExpressions::new(&registry);
        let direct = ast.direct(function.clone()).unwrap();
        let indirect = ast
            .indirect(
                ast.function_address(function.clone()).unwrap(),
                function.clone(),
            )
            .unwrap();
        for callable in [direct, indirect] {
            let call = row.call(&ast, callable, row.arguments(&ast)).unwrap();
            assert_eq!(
                call.contract(),
                CCallContract::Generated {
                    function: &function,
                    arguments: call.arguments()
                }
            );
        }
    }
}

#[test]
fn void_pointer_constness_and_file_identity_are_not_signature_shortcuts() {
    let (registry, _, _, _) = fixture();
    let ast = CExpressions::new(&registry);
    for row in rows() {
        let original = row.arguments(&ast);
        for (index, ty) in row.parameters.iter().enumerate() {
            let replacement = match ty.kind() {
                CObjectTypeKind::Pointer(CPointerTarget::Void(CConstness::Const)) => {
                    CPointerTarget::Void(CConstness::Unqualified)
                }
                CObjectTypeKind::Pointer(CPointerTarget::Void(CConstness::Unqualified)) => {
                    CPointerTarget::Void(CConstness::Const)
                }
                CObjectTypeKind::Pointer(CPointerTarget::Object(_)) => {
                    CPointerTarget::Void(CConstness::Unqualified)
                }
                _ => continue,
            };
            let mut arguments = original.clone();
            arguments[index] = ast
                .literal(CLiteral::NullPointer(
                    CNullPointer::new(CObjectType::pointer(replacement)).unwrap(),
                ))
                .unwrap();
            assert_eq!(
                row.call(&ast, ast.known(row.call), arguments),
                Err(CExpressionError::TypeMismatch)
            );
        }
    }
}
