//! Platform assertions consume a checked typed query, never raw C fragments.
use super::{
    super::{
        CStdType,
        platform::{self, Object, Query},
    },
    Writer,
};
use crate::ast::CStaticAssertion;
use std::fmt::Write;

impl Writer<'_> {
    pub(super) fn platform(&self, assertion: &CStaticAssertion, text: &mut String) {
        let check = platform::classify(assertion).expect("checked platform assertion");
        match check.query {
            Query::Binary64(property) => {
                use crate::ast::CKnownConstant as K;
                let name = match property.constant() {
                    K::FloatRadix => "FLT_RADIX",
                    K::DoubleMantissaDigits => "DBL_MANT_DIG",
                    K::DoubleMinExponent => "DBL_MIN_EXP",
                    K::DoubleMaxExponent => "DBL_MAX_EXP",
                    K::DoubleHasSubnormals => "DBL_HAS_SUBNORM",
                    K::FloatEvaluationMethod => "FLT_EVAL_METHOD",
                    _ => unreachable!("closed binary64 platform properties"),
                };
                write!(
                    text,
                    "_Static_assert(({name} == {}), \"",
                    property.expected()
                )
                .unwrap();
            }
            Query::Size | Query::Alignment => {
                let query = match check.query {
                    Query::Size => "sizeof",
                    Query::Alignment => "_Alignof",
                    Query::Binary64(_) => unreachable!(),
                };
                let ty = match check.object {
                    Object::Bool => "_Bool",
                    Object::Int => "int",
                    Object::F64 => "double",
                    Object::Pointer => "void *",
                    Object::I32 => self.names.standards[&CStdType::I32].as_str(),
                    Object::I64 => self.names.standards[&CStdType::I64].as_str(),
                    Object::U32 => self.names.standards[&CStdType::U32].as_str(),
                    Object::U64 => self.names.standards[&CStdType::U64].as_str(),
                    Object::Size => self.names.standards[&CStdType::Size].as_str(),
                };
                write!(
                    text,
                    "_Static_assert(({query}({ty}) == {}UL), \"",
                    check.object.bytes()
                )
                .unwrap();
            }
        }
        // Constructor-normalized text contains no quotes, escapes, trigraphs
        // or line breaks; diagnostic presentation is not executable source.
        text.push_str(assertion.diagnostic().display_text());
        text.push_str("\");\n");
    }
}
