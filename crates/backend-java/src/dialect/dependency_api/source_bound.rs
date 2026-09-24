//! Source reservation for the closed owner subset, independent of text emission.
mod bodies;
mod budget;
mod results;

use super::{JavaDependencyApi, JavaDialect};
use crate::ast::{
    JavaDeclarationKind, JavaFileItem, JavaHeritage, JavaMember, JavaMethodDeclaration,
    JavaParameter, JavaPrimitive, JavaResolvedName, JavaType, JavaTypeDeclaration, JavaTypeName,
};
use budget::Budget;
use portable_codegen::{GeneratedSymbolId, TargetSymbolRef};
use std::collections::BTreeMap;

pub(super) fn measure(api: &JavaDependencyApi) -> Result<u64, String> {
    let [file] = api.package().ast().files() else {
        return Err("source reservation requires one owner file".into());
    };
    let [item] = file.items() else {
        return Err("source reservation requires one facade".into());
    };
    let JavaFileItem::Type { declaration, .. } = &item.item else {
        return Err("source reservation requires a type declaration".into());
    };
    let mut reader = Reader {
        budget: Budget::new(),
        names: &item.names,
        result_types: api.owner.0.result_types.keys().copied().collect(),
    };
    reader.budget.node(0)?; // Header, package keyword, separators and final LF.
    reader.spelling(&file.module().name())?;
    for import in file.imports() {
        reader.budget.node(0)?;
        reader.spelling(import.kind().qualified_name())?;
    }
    // The admitted owner renders each attachment at most once. Comments are
    // already escaped by the documentation projection; charge presentation,
    // not the smaller original Rust text. Saturation is rejected by the budget.
    for (_, attachment) in item.documentation.iter() {
        reader.budget.add(attachment.presentation_len() as u64)?;
    }
    reader.declaration(declaration, 0)?;
    Ok(reader.budget.bytes())
}

struct Reader<'a> {
    budget: Budget,
    names: &'a BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    result_types: std::collections::BTreeSet<portable_codegen::GeneratedTypeId>,
}
impl Reader<'_> {
    fn spelling(&mut self, spelling: &str) -> Result<(), String> {
        self.budget.add(spelling.len() as u64)
    }
    fn symbol(&mut self, symbol: TargetSymbolRef<JavaDialect>) -> Result<(), String> {
        let name = self
            .names
            .get(&symbol)
            .ok_or("source reservation lacks resolved name")?;
        match name {
            JavaResolvedName::Local(name) => self.spelling(name.as_str()),
            JavaResolvedName::Qualified(crate::dialect::JavaQualifiedName::Type(known)) => {
                self.spelling(known.qualified_name())
            }
            JavaResolvedName::DeclaredPath(path)
            | JavaResolvedName::Qualified(crate::dialect::JavaQualifiedName::Dependency(path)) => {
                self.spelling(&path.package().name())?;
                for name in path.owners().iter().chain(std::iter::once(path.member())) {
                    self.budget.add(1)?;
                    self.spelling(name.as_str())?;
                }
                Ok(())
            }
            _ => Err("source reservation encountered an unsupported resolved name".into()),
        }
    }
    fn ty(&mut self, ty: &JavaType) -> Result<(), String> {
        match ty {
            JavaType::Reference(JavaTypeName::Imported(value)) => {
                self.symbol(TargetSymbolRef::KnownType(value.clone().into()))
            }
            JavaType::Primitive(
                JavaPrimitive::Int
                | JavaPrimitive::Long
                | JavaPrimitive::Boolean
                | JavaPrimitive::Double,
            ) => self.budget.add(7),
            JavaType::Reference(JavaTypeName::Generated(id)) => {
                self.symbol(TargetSymbolRef::Generated(GeneratedSymbolId::Type(*id)))
            }
            _ => Err("source reservation encountered an unsupported type".into()),
        }
    }
    fn parameters(&mut self, parameters: &[JavaParameter], depth: usize) -> Result<(), String> {
        for parameter in parameters {
            self.budget.node(depth)?;
            self.ty(&parameter.ty)?;
            self.spelling(parameter.name.as_str())?;
        }
        Ok(())
    }
    fn declaration(&mut self, value: &JavaTypeDeclaration, depth: usize) -> Result<(), String> {
        self.budget.node(depth)?;
        let selected_result = value
            .declared
            .is_some_and(|id| self.result_types.contains(&id));
        if !selected_result
            && (!matches!(
                value.kind,
                JavaDeclarationKind::FinalClass | JavaDeclarationKind::Record
            ) || !value.modifiers.is_empty()
                || !value.type_parameters.is_empty()
                || value.heritage != JavaHeritage::None
                || !value.permits.is_empty())
        {
            return Err("source reservation encountered an unsupported declaration".into());
        }
        if selected_result {
            if let JavaHeritage::Interfaces(interfaces) = &value.heritage {
                for ty in interfaces {
                    self.ty(ty)?;
                }
            }
            for ty in &value.permits {
                self.ty(ty)?;
            }
        }
        self.symbol(TargetSymbolRef::Generated(GeneratedSymbolId::Type(
            value
                .declared
                .ok_or("source declaration identity missing")?,
        )))?;
        for component in &value.record_components {
            self.budget.node(depth + 1)?;
            self.ty(&component.ty)?;
            self.spelling(component.name.as_str())?;
        }
        for member in &value.members {
            self.budget.node(depth + 1)?;
            match member {
                JavaMember::NestedType(value) => self.declaration(value, depth + 1)?,
                JavaMember::Constructor(value) => {
                    self.spelling(value.name.as_str())?;
                    self.parameters(&value.parameters, depth + 1)?;
                    self.block(&value.body, depth + 2)?;
                }
                JavaMember::Field(value) => {
                    self.symbol(TargetSymbolRef::Generated(GeneratedSymbolId::Value(
                        value.declared.ok_or("source constant identity missing")?,
                    )))?;
                    self.ty(&value.ty)?;
                    self.expression(
                        value
                            .initializer
                            .as_ref()
                            .ok_or("source constant initializer missing")?,
                        depth + 2,
                    )?;
                }
                JavaMember::Method(value) => {
                    let JavaMethodDeclaration::Callable(id) = value.declared else {
                        return Err("source method identity missing".into());
                    };
                    if !value.annotations.is_empty() || !value.type_parameters.is_empty() {
                        return Err("source reservation encountered an unsupported method".into());
                    }
                    self.symbol(TargetSymbolRef::Generated(GeneratedSymbolId::Callable(id)))?;
                    if value.return_type == JavaType::primitive(JavaPrimitive::Void) {
                        self.budget.add(4)?;
                    } else {
                        self.ty(&value.return_type)?;
                    }
                    self.parameters(&value.parameters, depth + 1)?;
                    self.block(
                        value.body.as_ref().ok_or("source method body missing")?,
                        depth + 2,
                    )?;
                }
                _ => return Err("source reservation encountered an unsupported member".into()),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/source_output_bound.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/infinite_constant_bounds.rs"]
mod infinite_tests;
