//! Java AST: file verification.

use super::file_model::JavaFileItem;
use super::privileged_literals::{
    verify_privileged_literals_in_declaration, verify_privileged_literals_in_member,
};
use super::statement_context::verify_member_type_context;
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedSymbolId, TargetAstContext, TargetFileItemNode};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

impl TargetFileItemNode<JavaDialect> for JavaFileItem {
    fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        match self {
            Self::Type {
                declared,
                declaration,
            } => {
                let mut violations = declaration.verify(context, true);
                violations.extend(verify_privileged_literals_in_declaration(declaration, None));
                for symbol in declared {
                    let present = match symbol {
                        GeneratedSymbolId::Type(id) => context.generated_type(*id).is_some(),
                        GeneratedSymbolId::Callable(id) => context.callable(*id).is_some(),
                        GeneratedSymbolId::InterfaceMethod(id) => {
                            context.interface_method(*id).is_some()
                        }
                        GeneratedSymbolId::Value(id) => context.value(*id).is_some(),
                    };
                    if !present {
                        violations.push(AstViolation::new(
                            DiagnosticCode::UnresolvedReference,
                            "file item declares an unknown generated symbol",
                        ));
                    }
                }
                violations
            }
            Self::RuntimeMembers { helper, members } => {
                let variables = BTreeSet::new();
                let mut violations = members
                    .iter()
                    .flat_map(|value| {
                        let mut violations = value.verify(context);
                        violations.extend(verify_member_type_context(value, &variables, context));
                        violations
                    })
                    .collect::<Vec<_>>();
                for member in members {
                    violations.extend(verify_privileged_literals_in_member(
                        member,
                        Some(*helper),
                        None,
                    ));
                }
                violations
            }
        }
    }
}
