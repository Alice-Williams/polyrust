//! Java primitive type/precedence, single local receiver and callable identity.
use super::TruncationInput;
use crate::java_lower::Reader;
use portable_backend_java::ast::*;
#[path = "floating_java_dataflow.rs"]
mod dataflow;
#[path = "floating_source_ast.rs"]
mod source;

pub(super) fn observe<'tcx>(
    reader: &Reader<'tcx>,
    input: &TruncationInput<'tcx>,
    value: &JavaExpr,
    start: usize,
) {
    input.probe(reader.tcx, reader.checked);
    use portable_backend_java::dialect::JavaKnownCallable;
    let JavaExprKind::Conditional { condition, .. } = &value.kind else {
        panic!("rounding selection")
    };
    let JavaExprKind::Binary {
        operator: JavaBinaryOperator::Less,
        left: operand,
        ..
    } = &condition.kind
    else {
        panic!("sign comparison")
    };
    let zero = JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Double),
        JavaLiteral::F64(portable_binary64::FiniteBinary64::from_bits(0).unwrap()),
    );
    let condition = JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Boolean),
        precedence: JavaPrecedence::Relational,
        kind: JavaExprKind::Binary {
            operator: JavaBinaryOperator::Less,
            left: operand.clone(),
            right: Box::new(zero),
        },
    };
    let call = |callable: JavaKnownCallable| JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Double),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Known {
                callable,
                signature: callable.signature(),
            },
            receiver: None,
            arguments: vec![operand.as_ref().clone()],
        },
    };
    let expected = JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Double),
        precedence: JavaPrecedence::Conditional,
        kind: JavaExprKind::Conditional {
            condition: Box::new(condition),
            when_true: Box::new(call(JavaKnownCallable::MathCeil)),
            when_false: Box::new(call(JavaKnownCallable::MathFloor)),
        },
    };
    assert_eq!(value, &expected, "complete exact typed rounding selection");
    assert_eq!(operand.ty, JavaType::primitive(JavaPrimitive::Double));
    let JavaExprKind::Value(JavaValueRef::Local(local)) = &operand.kind else {
        panic!("pure receiver local")
    };
    if dataflow::check(reader, input.receiver(), operand) {
        eprintln!("TRUNCATION_DETACHED\tjava");
    }
    let prelude = &reader.prelude[start..];
    let JavaStmt::Local {
        name,
        value: Some(_),
        ..
    } = prelude.last().unwrap()
    else {
        panic!("receiver declaration")
    };
    assert_eq!(name, local);
    let mut actual = Vec::new();
    for statement in prelude {
        let JavaStmt::Local {
            value: Some(value), ..
        } = statement
        else {
            panic!("initialized local")
        };
        if let JavaExprKind::Call {
            callable,
            receiver: None,
            ..
        } = &value.kind
        {
            actual.push(callable.clone());
        }
    }
    let expected: Vec<_> = source::calls(reader.checked, input.receiver())
        .into_iter()
        .map(|id| match id.as_local() {
            Some(id) => {
                let function = &reader.functions[&id];
                JavaCallableRef::Generated {
                    symbol: function.id,
                    signature: function.signature.clone(),
                }
            }
            None => JavaCallableRef::Dependency(reader.imported[&id].clone()),
        })
        .collect();
    assert_eq!(actual, expected, "exact receiver calls and original owners");
    eprintln!("TRUNCATION_AST\tjava\tF64");
}
