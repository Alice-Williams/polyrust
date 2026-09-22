//! Java primitive type/precedence, single local receiver and callable identity.
use super::WideningInput;
use crate::java_lower::Reader;
use portable_backend_java::ast::*;
#[path = "widening_java_dataflow.rs"]
mod dataflow;
#[path = "floating_source_ast.rs"]
mod source;

pub(super) fn check<'tcx>(
    reader: &Reader<'tcx>,
    input: &WideningInput<'tcx>,
    value: &JavaExpr,
    start: usize,
) {
    input.probe(reader.tcx, reader.checked);
    let primitive = JavaPrimitive::Long;
    assert_eq!(value.ty, JavaType::primitive(primitive));
    assert_eq!(value.precedence, JavaPrecedence::Unary);
    let JavaExprKind::Cast {
        target,
        value: operand,
    } = &value.kind
    else {
        panic!("primitive signed widening")
    };
    assert_eq!(*target, value.ty);
    assert_eq!(operand.ty, JavaType::primitive(JavaPrimitive::Int));
    let JavaExprKind::Value(JavaValueRef::Local(local)) = &operand.kind else {
        panic!("pure receiver local")
    };
    if dataflow::check(reader, input.operand(), operand) {
        eprintln!("WIDENING_DETACHED\tjava");
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
    let expected: Vec<_> = source::calls(reader.checked, input.operand())
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
    eprintln!("WIDENING_AST\tjava\tI32_I64");
}
