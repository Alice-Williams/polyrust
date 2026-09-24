//! Link-time projection to exact identifiers; spelling performs no resolution.
use super::{
    CDialect, CStdType,
    bindings::{CBindings, CValueBinding},
    violation,
};
use crate::ast::{CFunctionRef, CIdentifier, CStructRef};
use portable_codegen::{AstViolation, GeneratedSymbolId, ResolvedReference, TargetSymbolRef};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CResolvedNames {
    pub types: BTreeMap<CStructRef, CIdentifier>,
    pub functions: BTreeMap<CFunctionRef, CIdentifier>,
    pub values: BTreeMap<CValueBinding, CIdentifier>,
    pub standards: BTreeMap<CStdType, CIdentifier>,
}

impl CResolvedNames {
    pub fn new(
        bindings: &CBindings,
        names: &BTreeMap<TargetSymbolRef<CDialect>, ResolvedReference<CDialect>>,
    ) -> Result<Self, AstViolation> {
        let local = |id| match names.get(&TargetSymbolRef::Generated(id)) {
            Some(ResolvedReference::Local(name)) => Ok(name.clone()),
            _ => Err(violation(
                "generated C declaration must have a local linker binding",
            )),
        };
        let mut types: BTreeMap<_, _> = bindings
            .types
            .iter()
            .map(|(key, id)| Ok((key.clone(), local(GeneratedSymbolId::Type(*id))?)))
            .collect::<Result<_, AstViolation>>()?;
        for (record, proof) in &bindings.imported_types {
            let Some(ResolvedReference::Imported { binding, .. }) = names.get(
                &TargetSymbolRef::KnownType(super::CReferencedType::Certified(proof.clone())),
            ) else {
                return Err(violation(
                    "C dependency type must have its original imported binding",
                ));
            };
            if binding != proof.symbol() || types.insert(record.clone(), binding.clone()).is_some()
            {
                return Err(violation(
                    "C dependency type cannot be renamed or redefine an owned type",
                ));
            }
        }
        let mut functions: BTreeMap<_, _> = bindings
            .functions
            .iter()
            .map(|(key, id)| Ok((key.clone(), local(GeneratedSymbolId::Callable(*id))?)))
            .collect::<Result<_, AstViolation>>()?;
        for (function, import) in &bindings.imports {
            let Some(ResolvedReference::Imported { binding, .. }) =
                names.get(&TargetSymbolRef::DependencyCallable(import.clone()))
            else {
                return Err(violation(
                    "C dependency callable must have an imported binding",
                ));
            };
            if binding != import.dependency().symbol() {
                return Err(violation("C dependency callable cannot be import-aliased"));
            }
            if functions
                .insert(function.clone(), binding.clone())
                .is_some()
            {
                return Err(violation(
                    "C dependency callable conflicts with an owned binding",
                ));
            }
        }
        let mut values: BTreeMap<_, _> = bindings
            .values
            .iter()
            .map(|(key, id)| Ok((key.clone(), local(GeneratedSymbolId::Value(*id))?)))
            .collect::<Result<_, AstViolation>>()?;
        for (object, import) in &bindings.imported_values {
            let Some(ResolvedReference::Imported { binding, .. }) =
                names.get(&TargetSymbolRef::DependencyValue(import.clone()))
            else {
                return Err(violation(
                    "C dependency constant must have an imported binding",
                ));
            };
            if binding != import.dependency().symbol() {
                return Err(violation("C dependency constant cannot be import-aliased"));
            }
            if values
                .insert(CValueBinding::Global(object.clone()), binding.clone())
                .is_some()
            {
                return Err(violation(
                    "C dependency constant conflicts with an owned binding",
                ));
            }
        }
        let mut standards = BTreeMap::new();
        for (member, proof) in &bindings.imported_members {
            let Some(ResolvedReference::Member {
                owner,
                member: spelling,
            }) = names.get(&TargetSymbolRef::KnownField(proof.clone()))
            else {
                return Err(violation(
                    "C dependency field requires a typed owner/member reference",
                ));
            };
            if owner != proof.owner()
                || proof.member() != member
                || owner.member_name(member) != Some(spelling)
                || !types.contains_key(owner.record())
                || values
                    .insert(CValueBinding::Member(member.clone()), spelling.clone())
                    .is_some()
            {
                return Err(violation(
                    "C dependency field spelling or original owner disagrees",
                ));
            }
        }
        for (symbol, resolved) in names {
            if let TargetSymbolRef::KnownType(super::CReferencedType::Standard(kind)) = symbol {
                let ResolvedReference::Imported { binding, .. } = resolved else {
                    return Err(violation("C standard typedef must have an import binding"));
                };
                standards.insert(*kind, binding.clone());
            }
        }
        Ok(Self {
            types,
            functions,
            values,
            standards,
        })
    }
}
