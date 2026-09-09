//! Permitted roots stay legal; wrapping the same call never preserves root status.
use super::contextual_reconstruction::{fixture, fixture_return, key, package};
use super::known_call_fixtures::rows;
use super::*;
use crate::dialect::CKnownCall;

pub(super) fn int(ast: &CExpressions<'_>) -> CValue {
    ast.literal(CLiteral::Signed(CSignedLiteral::Int(1)))
        .unwrap()
}
pub(super) fn predicate_call(ast: &CExpressions<'_>) -> CValue {
    ast.call_value(
        ast.known(CKnownCall::IsNan),
        vec![ast.numeric_conversion(CScalarType::F64, int(ast)).unwrap()],
    )
    .unwrap()
}
pub(super) fn check(registry: &CRegistry, source: CSourceFile, rejected: bool) {
    registry
        .check_constants_and_layout(std::slice::from_ref(&source))
        .unwrap();
    assert_eq!(
        registry.check_sequencing_and_control(&[source]),
        if rejected {
            Err(CSafetyError::UnsequencedCall)
        } else {
            Ok(())
        }
    );
}

#[test]
fn composed_sequencing_boundary_does_not_skip_context_or_constant_checks() {
    let (mut registry, file, function, scope) = fixture();
    let local = registry
        .register_local(
            &scope,
            key("uninitialized"),
            CObjectType::scalar(CScalarType::Int),
        )
        .unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let source = package(
        &registry,
        file,
        function,
        scope,
        vec![
            ast.declare(local.clone(), None).unwrap(),
            ast.discard(values.read(values.local(local).unwrap()).unwrap())
                .unwrap(),
        ],
    );
    assert_eq!(
        registry.check_sequencing_and_control(&[source]),
        Err(CSafetyError::Context(CContextError::UninitializedRead))
    );

    let (registry, file, function, scope) = fixture();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let mut source = package(
        &registry,
        file.clone(),
        function,
        scope,
        vec![
            ast.discard(
                values
                    .unary(CUnaryOperator::Negate, predicate_call(&values))
                    .unwrap(),
            )
            .unwrap(),
        ],
    );
    source.items.push(CFileItem::StaticAssert(
        CDeclarations::new(&registry, file)
            .unwrap()
            .static_assert(
                values
                    .literal(CLiteral::Signed(CSignedLiteral::Int(0)))
                    .unwrap(),
                CAssertDiagnostic::new("false assertion"),
            )
            .unwrap(),
    ));
    assert_eq!(
        registry.check_sequencing_and_control(&[source]),
        Err(CSafetyError::FalseAssertion)
    );
}

#[derive(Clone, Copy, Debug)]
enum Callee {
    Known,
    Direct,
    Indirect,
}
#[derive(Clone, Copy, Debug)]
enum Position {
    Initializer,
    Assignment,
    Return,
    Discard,
    Label,
}

#[test]
fn each_value_root_has_direct_indirect_and_known_positive_and_wrapped_negative_controls() {
    for callee in [Callee::Known, Callee::Direct, Callee::Indirect] {
        for position in [
            Position::Initializer,
            Position::Assignment,
            Position::Return,
            Position::Discard,
            Position::Label,
        ] {
            for wrapped in [false, true] {
                let result = CReturnType::Value(
                    CReturnValue::new(CObjectType::scalar(CScalarType::Int)).unwrap(),
                );
                let (mut registry, file, function, scope) = fixture_return(result);
                let local =
                    matches!(position, Position::Initializer | Position::Assignment).then(|| {
                        registry
                            .register_local(
                                &scope,
                                key("slot"),
                                CObjectType::scalar(CScalarType::Int),
                            )
                            .unwrap()
                    });
                let label = matches!(position, Position::Label)
                    .then(|| registry.register_cleanup_exit(&scope, key("end")).unwrap());
                let values = CExpressions::new(&registry);
                let statements = CStatements::new(&registry, function.clone()).unwrap();
                let call = match callee {
                    Callee::Known => predicate_call(&values),
                    Callee::Direct => values
                        .call_value(values.direct(function.clone()).unwrap(), vec![])
                        .unwrap(),
                    Callee::Indirect => values
                        .call_value(
                            values
                                .indirect(
                                    values.function_address(function.clone()).unwrap(),
                                    function.clone(),
                                )
                                .unwrap(),
                            vec![],
                        )
                        .unwrap(),
                };
                let value = if wrapped {
                    values.unary(CUnaryOperator::Negate, call).unwrap()
                } else {
                    call
                };
                let mut body = Vec::new();
                match position {
                    Position::Initializer => body.push(
                        statements
                            .declare(
                                local.unwrap(),
                                Some(values.expression_initializer(value).unwrap()),
                            )
                            .unwrap(),
                    ),
                    Position::Assignment => {
                        let local = local.unwrap();
                        body.push(statements.declare(local.clone(), None).unwrap());
                        body.push(
                            statements
                                .assign(values.local(local).unwrap(), value)
                                .unwrap(),
                        );
                    }
                    Position::Return => {
                        body.push(statements.return_statement(Some(value)).unwrap())
                    }
                    Position::Discard => body.push(statements.discard(value).unwrap()),
                    Position::Label => body.push(
                        statements
                            .label(label.unwrap(), statements.discard(value).unwrap())
                            .unwrap(),
                    ),
                }
                body.push(statements.return_statement(Some(int(&values))).unwrap());
                // Recursive self calls test sequencing only; callable safety remains 02D-05.
                check(
                    &registry,
                    package(&registry, file, function, scope, body),
                    wrapped,
                );
            }
        }
    }
}

#[test]
fn every_known_contract_accepts_its_call_free_standalone_root() {
    for row in rows() {
        let (registry, file, function, scope) = fixture();
        let values = CExpressions::new(&registry);
        let statements = CStatements::new(&registry, function.clone()).unwrap();
        let call = if row.result.is_some() {
            statements
                .discard(
                    values
                        .call_value(values.known(row.call), row.arguments(&values))
                        .unwrap(),
                )
                .unwrap()
        } else {
            statements
                .evaluate(
                    values
                        .call_effect(values.known(row.call), row.arguments(&values))
                        .unwrap(),
                )
                .unwrap()
        };
        check(
            &registry,
            package(&registry, file, function, scope, vec![call]),
            false,
        );
    }
}

#[test]
fn void_direct_and_indirect_effect_roots_are_sequenced() {
    for indirect in [false, true] {
        let (registry, file, function, scope) = fixture();
        let values = CExpressions::new(&registry);
        let statements = CStatements::new(&registry, function.clone()).unwrap();
        let callable = if indirect {
            values
                .indirect(
                    values.function_address(function.clone()).unwrap(),
                    function.clone(),
                )
                .unwrap()
        } else {
            values.direct(function.clone()).unwrap()
        };
        check(
            &registry,
            package(
                &registry,
                file,
                function,
                scope,
                vec![
                    statements
                        .evaluate(values.call_effect(callable, vec![]).unwrap())
                        .unwrap(),
                ],
            ),
            false,
        );
    }
}
