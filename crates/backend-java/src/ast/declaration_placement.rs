//! The linker names registered declarations relative to their flat source unit.

use super::{JavaDeclarationKind, JavaMember, JavaMethodDeclaration, JavaTypeDeclaration};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedOrigin, SynthesisReason, TargetAstContext};
use portable_diagnostics::DiagnosticCode;

pub(super) fn verify(
    root: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    fn visit(
        node: &JavaTypeDeclaration,
        depth: usize,
        parent_is_entry: bool,
        context: &TargetAstContext<'_, JavaDialect>,
        errors: &mut Vec<AstViolation>,
    ) {
        let registered = node.declared.and_then(|id| context.generated_type(id));
        let is_entry = registered.is_some_and(|value| {
            matches!(
                value.origin,
                GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint)
            )
        });
        if is_entry
            && (depth != 0
                || !node.type_parameters.is_empty()
                || node.name.as_str() != "Generated"
                || node.kind != JavaDeclarationKind::FinalClass)
        {
            errors.push(error(
                "Java package entry must be the top-level non-generic Generated shell",
            ));
        }
        if let Some(value) = registered {
            let portable = matches!(
                value.origin,
                GeneratedOrigin::CoreDeclaration(_)
                    | GeneratedOrigin::Synthesized(SynthesisReason::UninhabitedInterface)
            );
            if depth > 1 || (portable && (depth != 1 || !parent_is_entry)) {
                errors.push(error(
                    "Java generated nominal types must retain their flat registered lexical owner",
                ));
            }
        }
        for member in &node.members {
            match member {
                JavaMember::NestedType(child) => visit(child, depth + 1, is_entry, context, errors),
                JavaMember::Method(method) => {
                    if let JavaMethodDeclaration::Callable(id) = method.declared {
                        let portable = context.callable(id).is_some_and(|value| {
                            matches!(value.origin, GeneratedOrigin::CoreDeclaration(_))
                        });
                        if depth != 0 || (portable && !is_entry) {
                            errors.push(error(
                                "Java registered callable must remain a direct source-unit member",
                            ));
                        }
                    }
                }
                JavaMember::Field(field) => {
                    if let Some(id) = field.declared {
                        let portable = context.value(id).is_some_and(|value| {
                            matches!(value.origin, GeneratedOrigin::CoreDeclaration(_))
                        });
                        if depth != 0 || (portable && !is_entry) {
                            errors.push(error("Java registered static value must remain a direct source-unit member"));
                        }
                    }
                }
                JavaMember::EnumConstant(_)
                | JavaMember::Constructor(_)
                | JavaMember::CompileFailField(_) => {}
            }
        }
    }
    let mut errors = Vec::new();
    visit(root, 0, false, context, &mut errors);
    errors
}

fn error(message: &str) -> AstViolation {
    AstViolation::new(DiagnosticCode::InvalidStructure, message)
}
