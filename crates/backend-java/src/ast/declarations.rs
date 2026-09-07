//! Java AST: declarations.

use super::declaration_grammar::verify_declaration_kind_grammar;
use super::declaration_model::{
    JavaDeclarationKind, JavaHeritage, JavaMember, JavaTypeDeclaration, JavaVisibility,
    declaration_owner_type,
};
use super::final_assignment::declaration_blank_instance_finals;
use super::identifiers::JavaIdentifier;
use super::initializers::{
    verify_declaration_type_context, verify_field_initializer_declaration_order,
};
use super::interface_conformance::{verify_interface_conformance, verify_method_registration};
use super::modifiers::{JavaModifierSite, verify_modifiers_for};
use super::object_members::{
    java_record_component_name_is_reserved, verify_inherited_object_method, verify_record_accessor,
};
use super::sealed_permits::verify_sealed_permits;
use super::type_context::erased_java_type;
use super::types::{JavaKnownType, JavaType, JavaTypeName, JavaTypeUse};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedSymbolId, TargetAstContext, TargetSymbolRef};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

impl JavaTypeDeclaration {
    pub fn contains_compile_fail_member(&self) -> bool {
        self.members.iter().any(|member| match member {
            JavaMember::CompileFailField(_) => true,
            JavaMember::NestedType(value) => value.contains_compile_fail_member(),
            JavaMember::Field(_)
            | JavaMember::EnumConstant(_)
            | JavaMember::Method(_)
            | JavaMember::Constructor(_) => false,
        })
    }

    pub fn symbols(&self, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
        if let Some(value) = self.declared {
            symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Type(value)));
        }
        for component in &self.record_components {
            component.ty.symbols(symbols);
        }
        if let JavaHeritage::Interfaces(values) = &self.heritage {
            for value in values {
                value.symbols(symbols);
            }
        }
        for value in &self.permits {
            value.symbols(symbols);
        }
        for member in &self.members {
            member.symbols(symbols);
        }
    }

    pub fn verify(
        &self,
        context: &TargetAstContext<'_, JavaDialect>,
        top_level: bool,
    ) -> Vec<AstViolation> {
        self.verify_with_enclosing(context, top_level, &[])
    }

    fn verify_with_enclosing(
        &self,
        context: &TargetAstContext<'_, JavaDialect>,
        top_level: bool,
        enclosing_names: &[JavaIdentifier],
    ) -> Vec<AstViolation> {
        let mut violations =
            verify_modifiers_for(&self.modifiers, JavaModifierSite::Type { top_level });
        if enclosing_names.contains(&self.name) {
            violations.push(AstViolation::new(
                DiagnosticCode::DuplicateDeclaration,
                "Java nested type cannot reuse any enclosing type name",
            ));
        }
        let owner = declaration_owner_type(self);
        let distinct_type_parameters = self.type_parameters.iter().collect::<BTreeSet<_>>();
        if distinct_type_parameters.len() != self.type_parameters.len() {
            violations.push(AstViolation::new(
                DiagnosticCode::DuplicateDeclaration,
                "Java type parameter is declared more than once",
            ));
        }
        violations.extend(verify_declaration_type_context(self, context));
        if !top_level && self.visibility == JavaVisibility::Package {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "nested types require explicit public or private visibility",
            ));
        }
        if top_level && self.visibility == JavaVisibility::Private {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "top-level Java types cannot be private",
            ));
        }
        if self.kind != JavaDeclarationKind::Record && !self.record_components.is_empty() {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "only records may have record components",
            ));
        }
        if self.kind != JavaDeclarationKind::SealedInterface && !self.permits.is_empty() {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "only sealed interfaces may have permits clauses",
            ));
        }
        if matches!(
            self.kind,
            JavaDeclarationKind::Interface | JavaDeclarationKind::SealedInterface
        ) && !matches!(self.heritage, JavaHeritage::None)
        {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "portable Java interfaces are flat and cannot extend an interface",
            ));
        }
        violations.extend(verify_declaration_kind_grammar(self));
        violations.extend(verify_sealed_permits(self, context));
        violations.extend(verify_field_initializer_declaration_order(self));
        match &self.heritage {
            JavaHeritage::Interfaces(values) => {
                if values.is_empty() {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java interface conformance list cannot be empty",
                    ));
                }
                let mut seen = BTreeSet::new();
                for value in values {
                    if !seen.insert(value) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java interface conformance is listed more than once",
                        ));
                    }
                    let interface = match value {
                        JavaType::Reference(JavaTypeName::Known(
                            JavaKnownType::RuntimeSemanticValue,
                        )) => true,
                        JavaType::Reference(JavaTypeName::Generated(id)) => {
                            context.generated_type(*id).is_some_and(|item| {
                                matches!(
                                    item.kind,
                                    JavaDeclarationKind::Interface
                                        | JavaDeclarationKind::SealedInterface
                                )
                            })
                        }
                        _ => false,
                    };
                    if !interface {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InterfaceNonconformance,
                            "Java implements clause must reference a declared interface",
                        ));
                    }
                }
            }
            JavaHeritage::None => {}
        }
        let mut field_names = BTreeSet::new();
        let mut component_origins = BTreeSet::new();
        for component in &self.record_components {
            violations.extend(component.ty.verify(JavaTypeUse::Field));
            if java_record_component_name_is_reserved(component.name.as_str()) {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "Java record component name conflicts with a reserved Object member",
                ));
            }
            if !field_names.insert(component.name.clone()) {
                violations.push(AstViolation::new(
                    DiagnosticCode::DuplicateDeclaration,
                    "Java record component is declared more than once",
                ));
            }
            if !component_origins.insert(component.origin) {
                violations.push(AstViolation::new(
                    DiagnosticCode::DuplicateDeclaration,
                    "Java record component origin is declared more than once",
                ));
            }
        }
        if let JavaHeritage::Interfaces(values) = &self.heritage {
            for value in values {
                violations.extend(value.verify(JavaTypeUse::TypeBound));
            }
        }
        let mut method_signatures = BTreeSet::new();
        let mut constructor_signatures = BTreeSet::new();
        let mut nested_type_names = BTreeSet::new();
        for member in &self.members {
            match member {
                JavaMember::Field(field) => {
                    if !field_names.insert(field.name.clone()) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java field conflicts with another field or record component",
                        ));
                    }
                }
                JavaMember::CompileFailField(field) => {
                    if !field_names.insert(field.name.clone()) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java compile-fail field conflicts with another field",
                        ));
                    }
                }
                JavaMember::EnumConstant(value) => {
                    if !field_names.insert(value.name.clone()) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java enum constant is declared more than once",
                        ));
                    }
                }
                JavaMember::Method(method) => {
                    let key = (
                        method.name.clone(),
                        method
                            .parameters
                            .iter()
                            .map(|parameter| erased_java_type(&parameter.ty))
                            .collect::<Vec<_>>(),
                    );
                    if !method_signatures.insert(key) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java method has a duplicate erased declaration signature",
                        ));
                    }
                    violations.extend(verify_method_registration(self, method, context));
                    violations.extend(verify_record_accessor(self, method));
                    violations.extend(verify_inherited_object_method(method));
                }
                JavaMember::Constructor(constructor) => {
                    let key = constructor
                        .parameters
                        .iter()
                        .map(|parameter| erased_java_type(&parameter.ty))
                        .collect::<Vec<_>>();
                    if !constructor_signatures.insert(key) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java constructor has a duplicate declaration signature",
                        ));
                    }
                    if constructor.name != self.name {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InvalidStructure,
                            "Java constructor name does not match its declaring type",
                        ));
                    }
                }
                JavaMember::NestedType(nested) => {
                    if !nested_type_names.insert(nested.name.clone()) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java nested type is declared more than once",
                        ));
                    }
                }
            }
            if let JavaMember::NestedType(nested) = member {
                let mut nested_enclosing = enclosing_names.to_vec();
                nested_enclosing.push(self.name.clone());
                violations.extend(nested.verify_with_enclosing(context, false, &nested_enclosing));
            } else {
                violations.extend(member.verify_with_owner(context, owner.as_ref(), Some(self)));
            }
        }
        if self.kind == JavaDeclarationKind::FinalClass
            && !declaration_blank_instance_finals(self).is_empty()
            && !self
                .members
                .iter()
                .any(|member| matches!(member, JavaMember::Constructor(_)))
        {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                "implicit Java constructor cannot initialize declared blank final instance fields",
            ));
        }
        violations.extend(verify_interface_conformance(self, context));
        violations
    }
}
