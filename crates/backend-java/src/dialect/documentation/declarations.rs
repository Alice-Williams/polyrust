use super::{Projection, symbol_origin};
use crate::ast::{
    JavaDocumentationOwner, JavaDocumentationStyle, JavaMember, JavaMethodDeclaration,
    JavaRecordComponentOrigin, JavaTypeDeclaration,
};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedOrigin, GeneratedSymbolId, TargetAstPackage};

pub(super) fn ty(
    package: &TargetAstPackage<JavaDialect>,
    declaration: &JavaTypeDeclaration,
    depth: usize,
    projection: &mut Projection,
) -> Result<(), AstViolation> {
    if let Some(id) = declaration.declared {
        symbol(package, GeneratedSymbolId::Type(id), depth * 2, projection)?;
        for component in &declaration.record_components {
            if let JavaRecordComponentOrigin::RustSource(field) = &component.origin {
                projection.attach(
                    JavaDocumentationOwner::RecordField {
                        owner: id,
                        field: field.origin.declaration,
                    },
                    (depth + 1) * 2,
                    JavaDocumentationStyle::Declaration,
                    &field.origin.documentation,
                )?;
            }
        }
    }
    for member in &declaration.members {
        match member {
            JavaMember::NestedType(child) => ty(package, child, depth + 1, projection)?,
            JavaMember::Field(field) => {
                if let Some(id) = field.declared {
                    symbol(
                        package,
                        GeneratedSymbolId::Value(id),
                        (depth + 1) * 2,
                        projection,
                    )?;
                }
            }
            JavaMember::EnumConstant(value) => symbol(
                package,
                GeneratedSymbolId::Value(value.declared),
                (depth + 1) * 4,
                projection,
            )?,
            JavaMember::Method(method) => {
                let owner = match method.declared {
                    JavaMethodDeclaration::Callable(id) => Some(GeneratedSymbolId::Callable(id)),
                    JavaMethodDeclaration::Interface(id) => {
                        Some(GeneratedSymbolId::InterfaceMethod(id))
                    }
                    JavaMethodDeclaration::Structural
                    | JavaMethodDeclaration::Implementation { .. }
                    | JavaMethodDeclaration::UninhabitedImplementation(_) => None,
                };
                if let Some(owner) = owner {
                    symbol(package, owner, (depth + 1) * 2, projection)?;
                }
            }
            JavaMember::CompileFailField(_) | JavaMember::Constructor(_) => {}
        }
    }
    Ok(())
}

fn symbol(
    package: &TargetAstPackage<JavaDialect>,
    symbol: GeneratedSymbolId,
    indentation: usize,
    projection: &mut Projection,
) -> Result<(), AstViolation> {
    if let Some(GeneratedOrigin::RustSource(origin)) = symbol_origin(package, symbol) {
        projection.attach(
            JavaDocumentationOwner::Symbol(symbol),
            indentation,
            JavaDocumentationStyle::Declaration,
            &origin.documentation,
        )?;
    }
    Ok(())
}
