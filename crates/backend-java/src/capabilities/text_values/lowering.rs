//! String data becomes bounded literals and non-constant balanced concatenation.

use crate::ast::literal_limits::string_chunks;
use crate::ast::{JavaExpr, JavaKnownType, JavaLiteral, JavaMemberOrigin, JavaType};
use crate::dialect::JavaKnownMethod;
use crate::lower::member_call;

pub(crate) fn text_value(value: &str) -> JavaExpr {
    assemble(&string_chunks(value))
}

fn assemble(chunks: &[&str]) -> JavaExpr {
    if let [value] = chunks {
        return JavaExpr::literal(
            JavaType::known(JavaKnownType::String),
            JavaLiteral::String((*value).to_owned()),
        );
    }
    let middle = chunks.len() / 2;
    member_call(
        assemble(&chunks[..middle]),
        JavaKnownMethod::StringConcat.name().text(),
        vec![assemble(&chunks[middle..])],
        JavaType::known(JavaKnownType::String),
        JavaMemberOrigin::Known(JavaKnownMethod::StringConcat),
    )
}
