//! Fixed delimiters and prefixes around already encoded, owner-bound text.
use crate::ast::{JavaDocumentation, JavaDocumentationOwner, JavaDocumentationStyle};
use std::fmt::Write;

#[cfg(test)]
#[path = "../tests/documentation_presentation.rs"]
mod tests;

pub(super) fn render(docs: &JavaDocumentation, owner: JavaDocumentationOwner) -> String {
    let Some(attachment) = docs.get(owner) else {
        return String::new();
    };
    let indent = " ".repeat(attachment.indentation());
    let mut output = String::new();
    match attachment.style() {
        JavaDocumentationStyle::Declaration => {
            writeln!(output, "{indent}/**").unwrap();
        }
        JavaDocumentationStyle::ExplanatoryModule => {
            let JavaDocumentationOwner::Module(id) = owner else {
                unreachable!("certified explanatory comment has a module owner");
            };
            writeln!(
                output,
                "{indent}/* Rust module {:016x}:{:016x}",
                id.crate_id, id.definition_path_hash
            )
            .unwrap();
        }
    }
    for line in attachment
        .comments()
        .iter()
        .flat_map(|comment| comment.text().split('\n'))
    {
        writeln!(output, "{indent} * {line}").unwrap();
    }
    writeln!(output, "{indent} */").unwrap();
    output
}

pub(super) fn modules(docs: &JavaDocumentation) -> String {
    let mut output = String::new();
    for module in docs.modules() {
        output.push_str(&render(docs, JavaDocumentationOwner::Module(*module)));
    }
    if let Some(root) = docs.root() {
        output.push_str(&render(docs, JavaDocumentationOwner::Module(root)));
    }
    output
}
