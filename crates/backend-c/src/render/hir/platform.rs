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
        let query = match check.query {
            Query::Size => "sizeof",
            Query::Alignment => "_Alignof",
        };
        let ty = match check.object {
            Object::Bool => "_Bool",
            Object::Int => "int",
            Object::Pointer => "void *",
            Object::I32 => self.names.standards[&CStdType::I32].as_str(),
            Object::I64 => self.names.standards[&CStdType::I64].as_str(),
            Object::Size => self.names.standards[&CStdType::Size].as_str(),
        };
        write!(
            text,
            "_Static_assert(({query}({ty}) == {}UL), \"",
            check.object.bytes()
        )
        .unwrap();
        // Constructor-normalized text contains no quotes, escapes, trigraphs
        // or line breaks; diagnostic presentation is not executable source.
        text.push_str(assertion.diagnostic().display_text());
        text.push_str("\");\n");
    }
}
