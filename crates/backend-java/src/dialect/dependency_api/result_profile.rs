//! Indexed selected-family relations for the narrower source-owner body profile.
use super::{JavaResultTypeRole, result_layout::ResultLayout};
use crate::ast::{JavaType, JavaTypeName};
use portable_codegen::GeneratedTypeId;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Default)]
pub(super) struct ResultProfile {
    local: BTreeMap<GeneratedTypeId, (GeneratedTypeId, JavaResultTypeRole)>,
}
impl ResultProfile {
    pub(super) fn new(layouts: &BTreeMap<GeneratedTypeId, Arc<ResultLayout>>) -> Self {
        Self {
            local: layouts
                .values()
                .flat_map(|layout| {
                    [
                        (
                            layout.types.interface,
                            (layout.types.interface, JavaResultTypeRole::Interface),
                        ),
                        (
                            layout.types.success,
                            (layout.types.interface, JavaResultTypeRole::Success),
                        ),
                        (
                            layout.types.error,
                            (layout.types.interface, JavaResultTypeRole::Error),
                        ),
                    ]
                })
                .collect(),
        }
    }
    pub(super) fn role(&self, ty: &JavaType) -> Option<JavaResultTypeRole> {
        match ty {
            JavaType::Reference(JavaTypeName::Generated(id)) => {
                self.local.get(id).map(|(_, role)| *role)
            }
            JavaType::Reference(JavaTypeName::Imported(value)) => Some(value.original().role()),
            _ => None,
        }
    }
    pub(super) fn upcast(&self, source: &JavaType, target: &JavaType) -> bool {
        match (source, target) {
            (
                JavaType::Reference(JavaTypeName::Imported(source)),
                JavaType::Reference(JavaTypeName::Imported(target)),
            ) => source.implements(target),
            (
                JavaType::Reference(JavaTypeName::Generated(source)),
                JavaType::Reference(JavaTypeName::Generated(target)),
            ) => {
                matches!((self.local.get(source), self.local.get(target)),
                    (Some((source_family, JavaResultTypeRole::Success | JavaResultTypeRole::Error)), Some((target_family, JavaResultTypeRole::Interface)))
                    if source_family == target_family)
            }
            _ => false,
        }
    }
    pub(super) fn variant(&self, ty: &JavaType) -> bool {
        matches!(
            self.role(ty),
            Some(JavaResultTypeRole::Success | JavaResultTypeRole::Error)
        )
    }
}
