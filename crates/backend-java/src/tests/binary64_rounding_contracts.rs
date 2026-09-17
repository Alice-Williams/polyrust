//! Forged known-call metadata cannot acquire original-owner authority.
use super::*;

#[derive(Clone, Copy, Debug)]
enum Fault {
    Parameter,
    ResultSignature,
    Arity,
    Operand,
    ResultType,
    Precedence,
    Receiver,
    Nullable,
    Exceptions,
    Impure,
    Unadmitted,
}

#[test]
fn malformed_or_unadmitted_known_rounding_calls_reject() {
    for fault in [
        Fault::Parameter,
        Fault::ResultSignature,
        Fault::Arity,
        Fault::Operand,
        Fault::ResultType,
        Fault::Precedence,
        Fault::Receiver,
        Fault::Nullable,
        Fault::Exceptions,
        Fault::Impure,
        Fault::Unadmitted,
    ] {
        let mut functions = functions(&[]);
        let mut value = primitive(
            JavaKnownCallable::MathFloor,
            JavaExpr::local(double(), f::name("input")),
        );
        let JavaExprKind::Call {
            callable,
            receiver,
            arguments,
        } = &mut value.kind
        else {
            unreachable!()
        };
        let JavaCallableRef::Known {
            callable,
            signature,
        } = callable
        else {
            unreachable!()
        };
        let long = JavaType::primitive(JavaPrimitive::Long);
        match fault {
            Fault::Parameter => signature.parameters[0] = long.clone(),
            Fault::ResultSignature => signature.result = long.clone(),
            Fault::Arity => arguments.clear(),
            Fault::Operand => arguments[0] = JavaExpr::literal(long.clone(), JavaLiteral::I64(0)),
            Fault::ResultType => value.ty = long,
            Fault::Precedence => value.precedence = JavaPrecedence::Unary,
            Fault::Receiver => {
                *receiver = Some(Box::new(JavaExpr::local(double(), f::name("input"))))
            }
            Fault::Nullable => signature.nullable_result = true,
            Fault::Exceptions => signature
                .checked_exceptions
                .push(JavaKnownType::CharacterCodingException),
            Fault::Impure => signature.pure = false,
            Fault::Unadmitted => {
                *callable = JavaKnownCallable::DoubleIsNaN;
                *signature = callable.signature();
                value.ty = signature.result.clone();
                functions[0].result = value.ty.clone();
            }
        }
        functions[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(value))]);
        match verify_unresolved_package(&JavaDialect, f::package(108, functions)) {
            Err(_) => {}
            Ok(package) => {
                if let Ok(linked) = TargetLinker::new(JavaDialect).link_ast(&package)
                    && let Ok(certificate) = certify_resolved_package(&JavaDialect, linked)
                {
                    assert!(
                        JavaDependencyApi::from_certificate(certificate).is_err(),
                        "{fault:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn rounding_type_qualifiers_cannot_be_shadowed_by_parameters_or_locals() {
    for name in ["Math", "java"] {
        for parameter in [false, true] {
            let mut functions = functions(&[]);
            let value = primitive(
                JavaKnownCallable::MathFloor,
                JavaExpr::local(double(), f::name(if parameter { name } else { "input" })),
            );
            let mut statements = Vec::new();
            if parameter {
                functions[0].parameters[0].name = f::name(name);
            } else {
                statements.push(JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: double(),
                    name: f::name(name),
                    value: Some(JavaExpr::local(double(), f::name("input"))),
                });
            }
            statements.push(JavaStmt::Return(Some(value)));
            functions[0].body = JavaBlock::new(statements);
            assert!(
                verify_unresolved_package(&JavaDialect, f::package(108, functions)).is_err(),
                "{name}, parameter={parameter}"
            );
        }
    }
}
