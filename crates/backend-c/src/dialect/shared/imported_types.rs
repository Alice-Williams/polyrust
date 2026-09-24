//! Type and field catalogue metadata is reconstructed from immutable witnesses.
use super::{
    CDependencyStruct, CDialect, CImportKind, CReferencedType, bindings::CBindings, violation,
};
use crate::ast::{CAggregateRef, CMemberRef, CRegistry};
use portable_codegen::{
    AstViolation, DependencyPolicy, KnownFieldSpec, KnownTypeSpec, SymbolCatalogue, SymbolOrigin,
    TypePattern,
};
use portable_diagnostics::SourceRef;

/// A member reference cannot be detached from the original type certificate.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CImportedMember {
    owner: CDependencyStruct,
    member: CMemberRef,
}

impl CImportedMember {
    pub(super) fn new(owner: CDependencyStruct, member: CMemberRef) -> Result<Self, AstViolation> {
        if member.owner() != &CAggregateRef::Struct(owner.record().clone())
            || !owner.members().contains(&member)
            || owner.member_name(&member).is_none()
        {
            return Err(violation(
                "C imported member lacks its exact producer type witness",
            ));
        }
        Ok(Self { owner, member })
    }
    pub(super) fn owner(&self) -> &CDependencyStruct {
        &self.owner
    }
    pub(super) fn member(&self) -> &CMemberRef {
        &self.member
    }
    fn spec(&self) -> KnownFieldSpec<CDialect> {
        let name = self
            .owner
            .member_name(&self.member)
            .expect("private witnessed member")
            .clone();
        KnownFieldSpec {
            symbol: self.clone(),
            owner: CReferencedType::Certified(self.owner.clone()),
            name: name.clone(),
            origin: SymbolOrigin::CertifiedDependency(self.owner.package_identity()),
            ty: TypePattern::Exact(CBindings::default().ty(self.member.ty())),
            policy: DependencyPolicy::Member {
                owner: self.owner.clone(),
                member: name,
            },
            dependency: None,
            source: SourceRef::logical(["c", "certified-member", self.member.key().name.as_str()]),
        }
    }
}

pub(super) fn extend(
    catalogue: &mut SymbolCatalogue<CDialect>,
    registry: &CRegistry,
) -> Result<(), String> {
    // Certificate identity breaks invalid ties only; valid owners order by root/header.
    let mut proofs: Vec<_> = registry
        .imported_structs()
        .map(|(record, _)| registry.imported_struct(record))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    proofs.sort();
    for proof in proofs {
        let owner = proof.package_identity();
        catalogue.types.push(KnownTypeSpec {
            symbol: CReferencedType::Certified(proof.clone()),
            name: proof.symbol().clone(),
            alias_stem: proof.symbol().as_str().into(),
            qualified_name: None,
            origin: SymbolOrigin::CertifiedDependency(owner.clone()),
            arity: 0,
            policy: DependencyPolicy::FixedImport(CImportKind::Dependency(owner)),
            dependency: None,
            source: SourceRef::logical(["c", "certified-type", proof.symbol().as_str()]),
        });
        for member in proof.members() {
            catalogue.fields.push(
                CImportedMember::new(proof.clone(), member.clone())
                    .map_err(|e| e.message)?
                    .spec(),
            );
        }
    }
    Ok(())
}
