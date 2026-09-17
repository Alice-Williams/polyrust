//! Java primitive type/precedence, single local receiver and callable identity.
use super::AbsoluteInput;
use crate::java_lower::Reader;
use portable_backend_java::ast::*;
#[path = "floating_java_dataflow.rs"]
mod dataflow;
#[path = "floating_source_ast.rs"]
mod source;

pub(super) fn observe<'tcx>(
    reader: &Reader<'tcx>,
    input: &AbsoluteInput<'tcx>,
    value: &JavaExpr,
    start: usize,
) {
    input.probe(reader.tcx, reader.checked);
    let JavaExprKind::Conditional { condition, .. } = &value.kind else {
        panic!("outer zero conditional")
    };
    let JavaExprKind::Binary {
        operator: JavaBinaryOperator::Equal,
        left: operand,
        ..
    } = &condition.kind
    else {
        panic!("zero equality")
    };
    let zero = JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Double),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Literal(JavaLiteral::F64(
            portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
        )),
    };
    let compare = |operator, precedence| JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Boolean),
        precedence,
        kind: JavaExprKind::Binary {
            operator,
            left: operand.clone(),
            right: Box::new(zero.clone()),
        },
    };
    let select = |condition, when_true, when_false| JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Double),
        precedence: JavaPrecedence::Conditional,
        kind: JavaExprKind::Conditional {
            condition: Box::new(condition),
            when_true: Box::new(when_true),
            when_false: Box::new(when_false),
        },
    };
    let negative = JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Double),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Unary {
            operator: JavaUnaryOperator::Negate,
            operand: operand.clone(),
        },
    };
    let magnitude = select(
        compare(JavaBinaryOperator::Less, JavaPrecedence::Relational),
        negative,
        operand.as_ref().clone(),
    );
    let expected = select(
        compare(JavaBinaryOperator::Equal, JavaPrecedence::Equality),
        zero,
        magnitude,
    );
    assert_eq!(
        value, &expected,
        "complete typed zero/sign selection and original operand"
    );
    assert_eq!(operand.ty, JavaType::primitive(JavaPrimitive::Double));
    let JavaExprKind::Value(JavaValueRef::Local(local)) = &operand.kind else {
        panic!("pure receiver local")
    };
    if dataflow::check(reader, input.receiver(), operand) {
        eprintln!("ABSOLUTE_DETACHED\tjava");
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
    eprintln!("ABSOLUTE_AST\tjava\tF64");
}
