//! The package composes all-syntax checking with selected constant evaluation.
use super::contextual_reconstruction::{fixture, key, package};
use super::*;

fn int(ast: &CExpressions<'_>, value: i32) -> CValue {
    ast.literal(CLiteral::Signed(CSignedLiteral::I32(value)))
        .unwrap()
}

fn assertion(
    registry: &CRegistry,
    file: CFileRef,
    function: CFunctionRef,
    scope: CScopeRef,
    condition: CValue,
) -> CSourceFile {
    let mut source = package(registry, file.clone(), function, scope, vec![]);
    source.items.push(CFileItem::StaticAssert(
        CDeclarations::new(registry, file)
            .unwrap()
            .static_assert(condition, CAssertDiagnostic::new("checked constant"))
            .unwrap(),
    ));
    source
}

#[test]
fn assertions_reject_false_unsafe_or_out_of_range_actual_constants() {
    let (registry, file, function, scope) = fixture();
    let values = CExpressions::new(&registry);
    let cases = [
        (int(&values, 0), CSafetyError::FalseAssertion),
        (
            values
                .binary(CBinaryOperator::Divide, int(&values, 1), int(&values, 0))
                .unwrap(),
            CSafetyError::DivisionByZero,
        ),
        (
            values
                .binary(
                    CBinaryOperator::Add,
                    int(&values, i32::MAX),
                    int(&values, 1),
                )
                .unwrap(),
            CSafetyError::SignedOverflow,
        ),
        (
            values
                .binary(
                    CBinaryOperator::Remainder,
                    int(&values, i32::MIN),
                    int(&values, -1),
                )
                .unwrap(),
            CSafetyError::SignedOverflow,
        ),
        (
            values
                .binary(
                    CBinaryOperator::ShiftLeft,
                    int(&values, 1),
                    int(&values, 32),
                )
                .unwrap(),
            CSafetyError::InvalidShift,
        ),
        (
            values
                .numeric_conversion(CScalarType::I8, int(&values, 128))
                .unwrap(),
            CSafetyError::IntegerRange,
        ),
    ];
    for (condition, error) in cases {
        let source = assertion(
            &registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            condition,
        );
        registry
            .check_context(std::slice::from_ref(&source))
            .unwrap();
        assert_eq!(registry.check_constants_and_layout(&[source]), Err(error));
    }
    registry
        .check_constants_and_layout(&[assertion(&registry, file, function, scope, int(&values, 1))])
        .unwrap();
}

#[test]
fn skipped_unsafe_operations_are_not_evaluated_but_selected_operations_are() {
    for selected in [false, true] {
        let (registry, file, function, scope) = fixture();
        let values = CExpressions::new(&registry);
        let unsafe_value = values
            .binary(CBinaryOperator::Divide, int(&values, 1), int(&values, 0))
            .unwrap();
        let unsafe_bool = values
            .numeric_conversion(CScalarType::Bool, unsafe_value.clone())
            .unwrap();
        let conditional = values
            .conditional(
                values.literal(CLiteral::Bool(selected)).unwrap(),
                unsafe_value,
                int(&values, 1),
            )
            .unwrap();
        let or = values
            .binary(
                CBinaryOperator::LogicalOr,
                values.literal(CLiteral::Bool(!selected)).unwrap(),
                unsafe_bool.clone(),
            )
            .unwrap();
        let and = values
            .binary(
                CBinaryOperator::LogicalAnd,
                values.literal(CLiteral::Bool(selected)).unwrap(),
                unsafe_bool,
            )
            .unwrap();
        let and = values
            .binary(CBinaryOperator::Equal, and, int(&values, 0))
            .unwrap();
        for condition in [conditional, or, and] {
            assert_eq!(
                registry.check_constants_and_layout(&[assertion(
                    &registry,
                    file.clone(),
                    function.clone(),
                    scope.clone(),
                    condition
                )]),
                if selected {
                    Err(CSafetyError::DivisionByZero)
                } else {
                    Ok(())
                }
            );
        }
    }
}

#[test]
fn unselected_and_unreachable_layout_operands_still_require_complete_types() {
    for alignment in [false, true] {
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
            let values = CExpressions::new(&registry);
            let ty = CObjectType::structure(record);
            let layout = if alignment {
                values.align_of(ty).unwrap()
            } else {
                values.size_of(ty).unwrap()
            };
            let selected = values
                .conditional(
                    values.literal(CLiteral::Bool(false)).unwrap(),
                    layout.clone(),
                    values
                        .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(1)))
                        .unwrap(),
                )
                .unwrap();
            let statements = CStatements::new(&registry, function.clone()).unwrap();
            let mut source = package(
                &registry,
                file.clone(),
                function,
                scope,
                vec![
                    statements.discard(selected).unwrap(),
                    statements.return_statement(None).unwrap(),
                    statements.discard(layout).unwrap(),
                ],
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
            registry
                .check_context(std::slice::from_ref(&source))
                .unwrap();
            assert_eq!(
                registry.check_constants_and_layout(&[source]),
                if complete {
                    Ok(())
                } else {
                    Err(CSafetyError::IncompleteLayout)
                }
            );
        }
    }
}

#[test]
fn malformed_unselected_children_fail_reconstruction_before_constant_selection() {
    let (registry, file, function, scope) = fixture();
    let values = CExpressions::new(&registry);
    let condition = values
        .conditional(
            values.literal(CLiteral::Bool(true)).unwrap(),
            int(&values, 1),
            int(&values, 2),
        )
        .unwrap();
    let mut source = assertion(&registry, file, function, scope, condition);
    registry
        .check_constants_and_layout(std::slice::from_ref(&source))
        .unwrap();
    let CFileItem::StaticAssert(assertion) = &mut source.items[1] else {
        panic!("assertion")
    };
    let CValueKind::Conditional { else_value, .. } = &mut assertion.condition.kind else {
        panic!("conditional")
    };
    else_value.ty = CObjectType::scalar(CScalarType::U64);
    assert_eq!(
        registry.check_constants_and_layout(&[source]),
        Err(CSafetyError::Context(
            CContextError::StoredStructureMismatch
        ))
    );
}

#[test]
fn nested_file_initializer_arithmetic_is_checked_beyond_constant_category() {
    for unsafe_operation in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let scalar = CObjectType::scalar(CScalarType::I32);
        let ty = CObjectType::array(scalar, CArrayLength::new(1).unwrap()).unwrap();
        let object = registry
            .register_object(&file, key("values"), ty.clone())
            .unwrap();
        let values = CExpressions::new(&registry);
        let expression = values
            .binary(
                CBinaryOperator::Divide,
                int(&values, 1),
                int(&values, if unsafe_operation { 0 } else { 1 }),
            )
            .unwrap();
        let initializer = values
            .array_initializer(ty, vec![values.expression_initializer(expression).unwrap()])
            .unwrap();
        let mut source = package(&registry, file.clone(), function, scope, vec![]);
        let declarations = CDeclarations::new(&registry, file).unwrap();
        source.items.push(CFileItem::Definition(
            declarations
                .object_definition(object, CLinkage::External, initializer)
                .unwrap(),
        ));
        registry
            .check_context(std::slice::from_ref(&source))
            .unwrap();
        assert_eq!(
            registry.check_constants_and_layout(&[source]),
            if unsafe_operation {
                Err(CSafetyError::DivisionByZero)
            } else {
                Ok(())
            }
        );
    }
}
