//! Java dialect: ast binding.

use super::arena_nodes::{JavaArenaExpression, JavaArenaStatement};
use super::file_checks::{
    declares_reserved_runtime_type, verify_composed_java_file, verify_java_file_identity,
};
use super::known_callables::JavaKnownCallable;
use super::runtime_callables::JavaRuntimeCallable;
use super::{
    JavaConstructedType, JavaDialect, JavaInvocationKind, JavaRuntimeType, JavaSyntheticOrigin,
};
use crate::ast::{
    JavaDeclarationKind, JavaFileItem, JavaFilePlacement, JavaKnownType, JavaMethodSignature,
    JavaPackage, JavaPrimitive, JavaSourceFileKind, JavaType, JavaTypeName, JavaVisibility,
};
use portable_codegen::{
    AstViolation, LinkedTargetPackage, TargetAstContext, TargetAstPackage, TargetCallableSignature,
    TargetDialect, TargetFile, TargetTypeRef, TypedAstDialect, verify_linked_package,
    verify_target_ast,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode};

impl TargetDialect for JavaDialect {
    type Unresolved = TargetAstPackage<Self>;
    type Resolved = LinkedTargetPackage<Self>;

    fn verify_unresolved(&self, ast: &Self::Unresolved) -> Result<(), Vec<Diagnostic>> {
        verify_target_ast(ast)
    }

    fn verify_resolved(&self, ast: &Self::Resolved) -> Result<(), Vec<Diagnostic>> {
        verify_linked_package(ast)
    }
}

impl TypedAstDialect for JavaDialect {
    type PrimitiveType = JavaPrimitive;
    type KnownType = JavaKnownType;
    type RuntimeType = JavaRuntimeType;
    type ConstructedType = JavaConstructedType;
    type KnownCallable = JavaKnownCallable;
    type RuntimeCallable = JavaRuntimeCallable;
    type InvocationKind = JavaInvocationKind;
    type Visibility = JavaVisibility;
    type DeclarationKind = JavaDeclarationKind;
    type SymbolOrigin = JavaSyntheticOrigin;
    type SourceFileKind = JavaSourceFileKind;
    type ModuleDeclaration = JavaPackage;
    type FilePlacement = JavaFilePlacement;
    type Expression = JavaArenaExpression;
    type Statement = JavaArenaStatement;
    type FileItem = JavaFileItem;

    fn known_callable_signature(
        &self,
        callable: &Self::KnownCallable,
    ) -> TargetCallableSignature<Self> {
        self.coarse_signature(&callable.signature())
    }

    fn runtime_callable_signature(
        &self,
        callable: &Self::RuntimeCallable,
    ) -> TargetCallableSignature<Self> {
        self.coarse_signature(&callable.signature())
    }

    fn verify_signature(&self, signature: &TargetCallableSignature<Self>) -> Vec<AstViolation> {
        if signature.invocation == JavaInvocationKind::Constructor && signature.receiver.is_some() {
            vec![AstViolation::new(
                DiagnosticCode::InvalidInvocation,
                "Java constructors cannot have a receiver",
            )]
        } else {
            vec![]
        }
    }

    fn verify_source_file(
        &self,
        file: &TargetFile<Self>,
        context: &TargetAstContext<'_, Self>,
    ) -> Vec<AstViolation> {
        let mut violations = Vec::new();
        violations.extend(verify_java_file_identity(
            file.role(),
            file.path().as_str(),
            file.module(),
            file.placement(),
            file.items().iter().any(declares_reserved_runtime_type),
        ));
        if file
            .items()
            .iter()
            .any(|item| matches!(item, JavaFileItem::RuntimeMembers { .. }))
        {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "Java runtime member fragments must be injected by the typed helper linker",
            ));
        }
        let has_compile_fail_member = file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. }
                if declaration.contains_compile_fail_member())
        });
        match (*file.placement(), has_compile_fail_member) {
            (JavaFilePlacement::NegativeTest, false) => violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "Java negative-test file must contain a typed compile-fail member",
            )),
            (JavaFilePlacement::NegativeTest, true) => {}
            (_, true) => violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "Java compile-fail members are confined to negative-test files",
            )),
            (_, false) => {}
        }
        if !file.path().as_str().ends_with(".java") {
            violations.push(AstViolation::new(
                DiagnosticCode::UnsafeOutputPath,
                "Java source path must end in .java",
            ));
        }
        let public_types = file
            .items()
            .iter()
            .filter_map(|item| match item {
                JavaFileItem::Type { declaration, .. }
                    if declaration.visibility == JavaVisibility::Public =>
                {
                    Some(declaration)
                }
                JavaFileItem::Type { .. } | JavaFileItem::RuntimeMembers { .. } => None,
            })
            .collect::<Vec<_>>();
        if public_types.len() > 1 {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "Java compilation unit has more than one public top-level type",
            ));
        }
        if let [public_type] = public_types.as_slice() {
            let actual = file.path().as_str().rsplit('/').next().unwrap_or_default();
            let expected = format!("{}.java", public_type.name.as_str());
            if actual != expected {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    format!(
                        "Java public top-level type `{}` must be declared in `{expected}`",
                        public_type.name.as_str()
                    ),
                ));
            }
        }
        if file.source_kind() != &JavaSourceFileKind::CompilationUnit {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "Java source files must use the compilation-unit template",
            ));
        }
        violations.extend(verify_composed_java_file(
            file.placement(),
            file.items().iter().collect(),
            context,
        ));
        violations
    }
}

impl JavaDialect {
    pub fn registered_type(&self, ty: &JavaType) -> TargetTypeRef<Self> {
        match ty {
            JavaType::Primitive(value) => TargetTypeRef::Primitive(*value),
            JavaType::Boxed(value) => TargetTypeRef::Known(match value {
                JavaPrimitive::Boolean => JavaKnownType::Boolean,
                JavaPrimitive::Byte => JavaKnownType::Byte,
                JavaPrimitive::Char => JavaKnownType::Character,
                JavaPrimitive::Int => JavaKnownType::Integer,
                JavaPrimitive::Long => JavaKnownType::Long,
                JavaPrimitive::Double => JavaKnownType::Double,
                JavaPrimitive::Void => JavaKnownType::Object,
            }),
            JavaType::Reference(JavaTypeName::Known(value)) => TargetTypeRef::Known(*value),
            JavaType::Reference(JavaTypeName::Generated(value)) => TargetTypeRef::Generated(*value),
            JavaType::Array { .. }
            | JavaType::Generic { .. }
            | JavaType::Wildcard { .. }
            | JavaType::TypeVariable(_) => {
                TargetTypeRef::Constructed(JavaConstructedType(ty.clone()))
            }
        }
    }

    pub(crate) fn coarse_signature(
        &self,
        value: &JavaMethodSignature,
    ) -> TargetCallableSignature<Self> {
        TargetCallableSignature {
            invocation: if value.receiver.is_some() {
                JavaInvocationKind::Instance
            } else {
                JavaInvocationKind::Static
            },
            receiver: value
                .receiver
                .as_ref()
                .map(|value| self.registered_type(value)),
            parameters: value
                .parameters
                .iter()
                .map(|value| self.registered_type(value))
                .collect(),
            return_type: self.registered_type(&value.result),
        }
    }
}
