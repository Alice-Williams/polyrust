//! Java AST: lexical scope.

use super::declaration_model::{
    JavaConstructor, JavaField, JavaMember, JavaMethod, JavaMethodDeclaration, JavaModifier,
    JavaTypeDeclaration,
};
use super::expression_model::JavaValueRef;
use super::expression_nodes::{JavaExpr, JavaExprKind};
use super::field_metadata::JavaFieldMetadata;
use super::identifiers::JavaIdentifier;
use super::types::{JavaPrimitive, JavaType};
use portable_codegen::AstViolation;
use portable_diagnostics::DiagnosticCode;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(super) struct JavaLexicalBinding {
    pub(super) ty: JavaType,
    pub(super) mutable: bool,
    pub(super) definitely_assigned: bool,
}

#[derive(Clone, Debug)]
pub(super) struct JavaLexicalScope {
    pub(super) bindings: BTreeMap<JavaIdentifier, JavaLexicalBinding>,
    pub(super) allows_this: bool,
    pub(super) owner: Option<JavaType>,
    pub(super) constructor: bool,
    pub(super) owner_fields: BTreeMap<JavaIdentifier, JavaFieldMetadata>,
}

impl JavaLexicalScope {
    pub(super) fn bind(
        &mut self,
        name: JavaIdentifier,
        binding: JavaLexicalBinding,
        duplicate_message: &'static str,
        violations: &mut Vec<AstViolation>,
    ) -> bool {
        if self.bindings.contains_key(&name) {
            violations.push(AstViolation::new(
                DiagnosticCode::DuplicateDeclaration,
                duplicate_message,
            ));
            return false;
        }
        self.bindings.insert(name, binding);
        true
    }

    pub(super) fn mark_local_assigned(&mut self, target: &JavaExpr) {
        let JavaExprKind::Value(JavaValueRef::Local(name)) = &target.kind else {
            return;
        };
        if let Some(binding) = self.bindings.get_mut(name)
            && binding.mutable
            && binding.ty == target.ty
        {
            binding.definitely_assigned = true;
        }
    }

    pub(super) fn merge_definite_assignments(&mut self, alternatives: &[Self]) {
        for (name, binding) in &mut self.bindings {
            binding.definitely_assigned = alternatives.iter().all(|alternative| {
                alternative
                    .bindings
                    .get(name)
                    .is_some_and(|value| value.definitely_assigned)
            });
        }
    }

    #[cfg(test)]
    pub(super) fn for_method(method: &JavaMethod) -> (Self, Vec<AstViolation>) {
        Self::for_method_in_owner(method, None)
    }

    #[cfg(test)]
    fn for_method_in_owner(
        method: &JavaMethod,
        owner: Option<JavaType>,
    ) -> (Self, Vec<AstViolation>) {
        Self::for_method_in_declaration(method, owner, None)
    }

    pub(super) fn for_method_in_declaration(
        method: &JavaMethod,
        owner: Option<JavaType>,
        declaration: Option<&JavaTypeDeclaration>,
    ) -> (Self, Vec<AstViolation>) {
        let mut scope = Self {
            bindings: BTreeMap::new(),
            allows_this: !method.modifiers.contains(&JavaModifier::Static),
            owner,
            constructor: false,
            owner_fields: declaration
                .map(declared_instance_fields)
                .unwrap_or_default(),
        };
        let mut violations = Vec::new();
        for parameter in &method.parameters {
            scope.bind(
                parameter.name.clone(),
                JavaLexicalBinding {
                    ty: parameter.ty.clone(),
                    mutable: !parameter.final_parameter,
                    definitely_assigned: true,
                },
                "Java method parameter is declared more than once",
                &mut violations,
            );
        }
        (scope, violations)
    }

    pub(super) fn for_constructor_in_declaration(
        constructor: &JavaConstructor,
        owner: Option<JavaType>,
        declaration: Option<&JavaTypeDeclaration>,
    ) -> (Self, Vec<AstViolation>) {
        let method = JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![],
            modifiers: constructor.modifiers.clone(),
            type_parameters: vec![],
            return_type: JavaType::primitive(JavaPrimitive::Void),
            name: constructor.name.clone(),
            parameters: constructor.parameters.clone(),
            body: None,
        };
        let (mut scope, violations) = Self::for_method_in_declaration(&method, owner, declaration);
        scope.allows_this = true;
        scope.constructor = true;
        (scope, violations)
    }

    pub(super) fn for_field_initializer(
        field: &JavaField,
        owner: Option<JavaType>,
        declaration: Option<&JavaTypeDeclaration>,
    ) -> Self {
        Self {
            bindings: BTreeMap::new(),
            allows_this: !field.modifiers.contains(&JavaModifier::Static),
            owner,
            constructor: false,
            owner_fields: declaration
                .map(declared_instance_fields)
                .unwrap_or_default(),
        }
    }
}

fn declared_instance_fields(
    declaration: &JavaTypeDeclaration,
) -> BTreeMap<JavaIdentifier, JavaFieldMetadata> {
    let mut fields = declaration
        .record_components
        .iter()
        .map(|component| {
            (
                component.name.clone(),
                JavaFieldMetadata {
                    ty: component.ty.clone(),
                    final_field: true,
                    blank_final: true,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    for member in &declaration.members {
        if let JavaMember::Field(field) = member
            && !field.modifiers.contains(&JavaModifier::Static)
        {
            fields.insert(
                field.name.clone(),
                JavaFieldMetadata {
                    ty: field.ty.clone(),
                    final_field: field.modifiers.contains(&JavaModifier::Final),
                    blank_final: field.modifiers.contains(&JavaModifier::Final)
                        && field.initializer.is_none(),
                },
            );
        }
    }
    fields
}

pub(super) fn verify_pattern_binding_uniqueness(
    value: &JavaExpr,
    scope: &JavaLexicalScope,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    let mut pending = vec![value];
    let mut expression_bindings = BTreeSet::new();
    while let Some(value) = pending.pop() {
        match &value.kind {
            JavaExprKind::Literal(_) | JavaExprKind::Value(_) | JavaExprKind::Lambda { .. } => {}
            JavaExprKind::Unary { operand, .. } => pending.push(operand),
            JavaExprKind::Binary { left, right, .. } => {
                pending.push(left);
                pending.push(right);
            }
            JavaExprKind::Conditional {
                condition,
                when_true,
                when_false,
            } => {
                pending.push(condition);
                pending.push(when_true);
                pending.push(when_false);
            }
            JavaExprKind::Call {
                receiver,
                arguments,
                ..
            } => {
                pending.extend(receiver.iter().map(Box::as_ref));
                pending.extend(arguments);
            }
            JavaExprKind::New { arguments, .. } => pending.extend(arguments),
            JavaExprKind::NewArray { length, .. } => pending.push(length),
            JavaExprKind::ArrayIndex { array, index } => {
                pending.push(array);
                pending.push(index);
            }
            JavaExprKind::Field { receiver, .. } => pending.push(receiver),
            JavaExprKind::Cast { value, .. }
            | JavaExprKind::InterfaceCoercion { value, .. }
            | JavaExprKind::ArrayOwnershipTransition { value, .. } => pending.push(value),
            JavaExprKind::InstanceOf { value, binding, .. } => {
                pending.push(value);
                if let Some(binding) = binding
                    && (scope.bindings.contains_key(binding)
                        || !expression_bindings.insert(binding.clone()))
                {
                    violations.push(AstViolation::new(
                        DiagnosticCode::DuplicateDeclaration,
                        "Java instanceof binding conflicts with an overlapping lexical or expression binding",
                    ));
                }
            }
        }
    }
    violations
}
