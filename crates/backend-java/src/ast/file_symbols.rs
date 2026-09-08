//! Java AST: file symbols.

use super::file_model::JavaFileItem;
use super::{JavaMember, JavaMethodDeclaration, JavaTypeDeclaration};
use crate::dialect::JavaDialect;
use portable_codegen::{
    AstViolation, GeneratedSymbolId, GeneratedTypeId, TargetAstContext, TargetSymbolRef,
};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(super) fn verify_declaration_inventory(
    declared: &[GeneratedSymbolId],
    declaration: &JavaTypeDeclaration,
) -> Vec<AstViolation> {
    fn visit(declaration: &JavaTypeDeclaration, actual: &mut Vec<GeneratedSymbolId>) {
        if let Some(id) = declaration.declared {
            actual.push(GeneratedSymbolId::Type(id));
        }
        for member in &declaration.members {
            match member {
                JavaMember::NestedType(nested) => visit(nested, actual),
                JavaMember::Field(field) => {
                    if let Some(id) = field.declared {
                        actual.push(GeneratedSymbolId::Value(id));
                    }
                }
                JavaMember::EnumConstant(value) => {
                    actual.push(GeneratedSymbolId::Value(value.declared));
                }
                JavaMember::Method(method) => match method.declared {
                    JavaMethodDeclaration::Callable(id) => {
                        actual.push(GeneratedSymbolId::Callable(id))
                    }
                    JavaMethodDeclaration::Interface(id) => {
                        actual.push(GeneratedSymbolId::InterfaceMethod(id))
                    }
                    JavaMethodDeclaration::Structural
                    | JavaMethodDeclaration::Implementation { .. }
                    | JavaMethodDeclaration::UninhabitedImplementation(_) => {}
                },
                JavaMember::Constructor(_) | JavaMember::CompileFailField(_) => {}
            }
        }
    }
    let mut actual = Vec::new();
    visit(declaration, &mut actual);
    let actual_set = actual.iter().copied().collect::<BTreeSet<_>>();
    let claimed_set = declared.iter().copied().collect::<BTreeSet<_>>();
    if actual.len() == actual_set.len()
        && declared.len() == claimed_set.len()
        && actual_set == claimed_set
    {
        vec![]
    } else {
        vec![AstViolation::new(
            DiagnosticCode::UnresolvedReference,
            "Java file declaration inventory must match actual AST declarations exactly once",
        )]
    }
}

pub(super) fn verify_unique_type_declarations(
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    fn visit(
        declaration: &JavaTypeDeclaration,
        seen: &mut BTreeSet<GeneratedTypeId>,
        violations: &mut Vec<AstViolation>,
    ) {
        if let Some(id) = declaration.declared
            && !seen.insert(id)
        {
            violations.push(AstViolation::new(
                DiagnosticCode::DuplicateDeclaration,
                "Java generated type identity has more than one AST declaration",
            ));
        }
        for member in &declaration.members {
            if let JavaMember::NestedType(nested) = member {
                visit(nested, seen, violations);
            }
        }
    }
    let mut seen = BTreeSet::new();
    let mut top_level_names = BTreeSet::new();
    let mut violations = super::binary_names::verify(context);
    for file in context.files() {
        for item in file.items() {
            if let JavaFileItem::Type { declaration, .. } = item {
                if !top_level_names.insert((file.module(), &declaration.name)) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::DuplicateDeclaration,
                        "Java package contains more than one top-level type with the same name",
                    ));
                }
                visit(declaration, &mut seen, &mut violations);
            }
        }
    }
    violations
}

impl JavaFileItem {
    pub fn declared_symbols(&self) -> Vec<GeneratedSymbolId> {
        match self {
            Self::Type { declared, .. } => declared.clone(),
            Self::RuntimeMembers { .. } => vec![],
        }
    }

    pub fn symbols(&self) -> Vec<TargetSymbolRef<JavaDialect>> {
        let mut symbols = BTreeSet::new();
        match self {
            Self::Type { declaration, .. } => declaration.symbols(&mut symbols),
            Self::RuntimeMembers { helper, members } => {
                for member in members {
                    member.symbols(&mut symbols);
                }
                symbols.remove(&TargetSymbolRef::RuntimeHelper(*helper));
            }
        }
        symbols.into_iter().collect()
    }
}
