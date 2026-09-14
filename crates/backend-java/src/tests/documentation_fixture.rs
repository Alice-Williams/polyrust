use crate::ast::{
    JavaDocComment, JavaDocumentation, JavaDocumentationAttachment, JavaDocumentationOwner,
    JavaDocumentationStyle,
};
use portable_codegen::RustDeclarationId;

pub fn docs(depth: usize, style: JavaDocumentationStyle, attributes: &[&str]) -> JavaDocumentation {
    let id = RustDeclarationId {
        crate_id: 7,
        definition_path_hash: 8,
    };
    let mut docs = JavaDocumentation::default();
    docs.attachments.insert(
        JavaDocumentationOwner::Module(id),
        JavaDocumentationAttachment {
            indentation: depth,
            style,
            comments: attributes
                .iter()
                .map(|text| JavaDocComment::new(text))
                .collect(),
        },
    );
    match style {
        JavaDocumentationStyle::Declaration => docs.root = Some(id),
        JavaDocumentationStyle::ExplanatoryModule => docs.module_order.push(id),
    }
    docs
}
