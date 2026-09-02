use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef, sort_diagnostics};

use crate::{GeneratedTypeId, GeneratedValueId, TargetAstPackage, TypedAstDialect};

/// Closed origin family for a target-only base reference. Portable CoreIR can
/// never construct this type; each language supplies its own external-base
/// enum instead of a source string.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TargetHeritageBase<E> {
    ApprovedExternal(E),
    Generated(GeneratedTypeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetHeritageFinality {
    Final,
    Extensible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetHeritagePurpose {
    ExternalFrameworkAdapter,
    Representation,
    ImplementationReuse,
    InterfaceExtension,
    Mixin,
}

/// One proposed target-only heritage edge. The delegated component is
/// mandatory so adapter behavior remains composition-based.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetHeritageEdge<E> {
    pub subclass: GeneratedTypeId,
    pub base: TargetHeritageBase<E>,
    pub delegated_component: GeneratedValueId,
    pub finality: TargetHeritageFinality,
    pub purpose: TargetHeritagePurpose,
    pub source: SourceRef,
}

/// Opaque bounds used by the shared verifier without making heritage part of
/// the generic target AST package itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetHeritageScope {
    generated_types: usize,
    generated_values: usize,
}

impl<D: TypedAstDialect> TargetAstPackage<D> {
    pub fn heritage_scope(&self) -> TargetHeritageScope {
        TargetHeritageScope {
            generated_types: self.generated_types().len(),
            generated_values: self.values().len(),
        }
    }
}

/// Certifies the deliberately narrow one-edge inheritance escape hatch used by
/// target framework adapters. Any inheritance used for portable semantics or
/// implementation reuse is rejected before rendering.
pub fn verify_target_heritage<E>(
    scope: TargetHeritageScope,
    edges: &[TargetHeritageEdge<E>],
    approved_external: impl Fn(&E) -> bool,
) -> Result<(), Vec<Diagnostic>>
where
    E: Clone + Debug + Eq + Ord,
{
    let mut diagnostics = Vec::new();
    let mut subclasses = BTreeMap::<GeneratedTypeId, usize>::new();
    let generated_bases = edges
        .iter()
        .filter_map(|edge| match edge.base {
            TargetHeritageBase::Generated(base) => Some(base),
            TargetHeritageBase::ApprovedExternal(_) => None,
        })
        .collect::<BTreeSet<_>>();

    for edge in edges {
        *subclasses.entry(edge.subclass).or_default() += 1;
        if edge.subclass.index() >= scope.generated_types {
            heritage_error(
                &mut diagnostics,
                "heritage subclass is not a generated type",
                &edge.source,
            );
        }
        if edge.delegated_component.index() >= scope.generated_values {
            heritage_error(
                &mut diagnostics,
                "heritage adapter has no valid composed delegation component",
                &edge.source,
            );
        }
        if edge.finality != TargetHeritageFinality::Final {
            heritage_error(
                &mut diagnostics,
                "target-only heritage adapter must be final",
                &edge.source,
            );
        }
        if edge.purpose != TargetHeritagePurpose::ExternalFrameworkAdapter {
            heritage_error(
                &mut diagnostics,
                "target-only heritage is allowed only for an external framework adapter",
                &edge.source,
            );
        }
        match &edge.base {
            TargetHeritageBase::ApprovedExternal(base) if approved_external(base) => {}
            TargetHeritageBase::ApprovedExternal(_) => heritage_error(
                &mut diagnostics,
                "target-only heritage base is not in the language's approved external-base enum",
                &edge.source,
            ),
            TargetHeritageBase::Generated(_) => heritage_error(
                &mut diagnostics,
                "target-only heritage cannot extend a generated type",
                &edge.source,
            ),
        }
        if generated_bases.contains(&edge.subclass) {
            heritage_error(
                &mut diagnostics,
                "a generated heritage subclass cannot also be a generated base",
                &edge.source,
            );
        }
    }

    for (subclass, count) in subclasses {
        if count > 1 {
            let source = edges
                .iter()
                .find(|edge| edge.subclass == subclass)
                .map(|edge| edge.source.clone())
                .unwrap_or_else(|| SourceRef::logical(["target-heritage"]));
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                "target-only heritage permits at most one generated edge per adapter",
                source,
            ));
        }
    }

    sort_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn heritage_error(diagnostics: &mut Vec<Diagnostic>, message: &str, source: &SourceRef) {
    diagnostics.push(Diagnostic::error(
        DiagnosticCode::InvalidStructure,
        message,
        source.clone(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum ExternalBase {
        Approved,
        Unapproved,
    }

    fn source(name: &str) -> SourceRef {
        SourceRef::logical(["heritage", name])
    }

    fn valid_edge() -> TargetHeritageEdge<ExternalBase> {
        TargetHeritageEdge {
            subclass: GeneratedTypeId::from_index(0),
            base: TargetHeritageBase::ApprovedExternal(ExternalBase::Approved),
            delegated_component: GeneratedValueId::from_index(0),
            finality: TargetHeritageFinality::Final,
            purpose: TargetHeritagePurpose::ExternalFrameworkAdapter,
            source: source("valid"),
        }
    }

    fn verify(edges: &[TargetHeritageEdge<ExternalBase>]) -> Result<(), Vec<Diagnostic>> {
        verify_target_heritage(
            TargetHeritageScope {
                generated_types: 3,
                generated_values: 1,
            },
            edges,
            |base| *base == ExternalBase::Approved,
        )
    }

    #[test]
    fn final_one_edge_external_adapter_is_accepted() {
        assert_eq!(verify(&[valid_edge()]), Ok(()));
    }

    #[test]
    fn every_forbidden_heritage_shape_is_rejected() {
        let mut non_final = valid_edge();
        non_final.finality = TargetHeritageFinality::Extensible;
        let mut reuse = valid_edge();
        reuse.purpose = TargetHeritagePurpose::ImplementationReuse;
        let mut representation = valid_edge();
        representation.purpose = TargetHeritagePurpose::Representation;
        let mut interface_extension = valid_edge();
        interface_extension.purpose = TargetHeritagePurpose::InterfaceExtension;
        let mut mixin = valid_edge();
        mixin.purpose = TargetHeritagePurpose::Mixin;
        let mut generated_base = valid_edge();
        generated_base.base = TargetHeritageBase::Generated(GeneratedTypeId::from_index(1));
        let mut unapproved = valid_edge();
        unapproved.base = TargetHeritageBase::ApprovedExternal(ExternalBase::Unapproved);
        let mut missing_delegate = valid_edge();
        missing_delegate.delegated_component = GeneratedValueId::from_index(4);

        for invalid in [
            non_final,
            reuse,
            representation,
            interface_extension,
            mixin,
            generated_base,
            unapproved,
            missing_delegate,
        ] {
            assert!(verify(&[invalid]).is_err());
        }

        let mut second = valid_edge();
        second.base = TargetHeritageBase::ApprovedExternal(ExternalBase::Approved);
        second.source = source("second");
        assert!(verify(&[valid_edge(), second]).is_err());
    }
}
