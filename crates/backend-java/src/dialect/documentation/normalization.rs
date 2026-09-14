use crate::ast::{
    JavaDocComment, JavaDocumentation, JavaDocumentationAttachment, JavaDocumentationOwner,
    JavaDocumentationStyle,
};
use portable_codegen::AstViolation;
use portable_diagnostics::DiagnosticCode;

#[derive(Default)]
pub(super) struct Projection {
    pub documentation: JavaDocumentation,
    raw_bytes: usize,
    attributes: usize,
}

impl Projection {
    pub fn attach(
        &mut self,
        owner: JavaDocumentationOwner,
        indentation: usize,
        style: JavaDocumentationStyle,
        attributes: &[String],
    ) -> Result<(), AstViolation> {
        if attributes.is_empty() {
            return Ok(());
        }
        self.attributes = self.attributes.saturating_add(attributes.len());
        self.raw_bytes = attributes.iter().fold(self.raw_bytes, |total, text| {
            total.saturating_add(text.len())
        });
        if self.attributes > 100_000 || self.raw_bytes > 16 * 1024 * 1024 {
            return Err(AstViolation::new(
                DiagnosticCode::TargetResourceLimit,
                "Java documentation normalization input limit exceeded",
            ));
        }
        let attachment = JavaDocumentationAttachment {
            indentation,
            style,
            comments: attributes
                .iter()
                .map(|text| JavaDocComment::new(text))
                .collect(),
        };
        if self
            .documentation
            .attachments
            .insert(owner, attachment)
            .is_some()
        {
            return Err(super::error("duplicate primary Java documentation owner"));
        }
        Ok(())
    }
}
