//! Checked match-shape dispatch shared by admission and executable lowering.
use super::{JavaCapabilityRegistry, admission::JavaFeatureAdmission};
use portable_build::{CapabilityId, Enums, PatternMatching};
use portable_codegen::{ControlFeature, CoreFeature, FeatureShape, OperationFeature};
use portable_core_ir::{
    CoreEnumId, CoreExprId, CoreExprKind, CoreMatchArm, CorePattern, CoreProgram, CoreType,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JavaMatchDispatch {
    NativeEnum(CoreEnumId),
    General,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum MatchOwner {
    Enums,
    Patterns,
}

pub(crate) fn payload_free_enum(program: &CoreProgram, id: CoreEnumId) -> bool {
    program.enumeration(id).is_some_and(|enumeration| {
        !enumeration.variants.is_empty()
            && enumeration.variants.iter().all(|variant| {
                program
                    .variant(*variant)
                    .is_some_and(|variant| variant.fields.is_empty())
            })
    })
}

pub(crate) fn match_dispatch(
    program: &CoreProgram,
    value: CoreExprId,
    arms: &[CoreMatchArm],
) -> JavaMatchDispatch {
    if let Some(expression) = program.expressions().get(value)
        && let Some(CoreType::Enum(enumeration)) = program.types().get(expression.ty)
        && payload_free_enum(program, *enumeration)
        && arms
            .iter()
            .all(|arm| matches!(arm.pattern, CorePattern::EnumVariant { .. }))
    {
        JavaMatchDispatch::NativeEnum(*enumeration)
    } else {
        JavaMatchDispatch::General
    }
}

impl JavaCapabilityRegistry {
    pub(super) fn complete_match_admission(
        &self,
        program: &CoreProgram,
        admission: &mut JavaFeatureAdmission,
    ) {
        let usage = &admission.usage;
        let is_match = matches!(
            usage.feature(),
            CoreFeature::Control(ControlFeature::Match)
                | CoreFeature::Operation(OperationFeature::Match)
        );
        let is_enum_pattern = usage.feature() == CoreFeature::Control(ControlFeature::EnumPattern);
        if !is_match && !is_enum_pattern {
            return;
        }
        let mut owners = std::collections::BTreeSet::new();
        // FeatureUse deduplicates feature+shape. Its prerequisites must cover
        // every checked occurrence represented by that collected shape.
        for (_, expression) in program.expressions().iter() {
            let CoreExprKind::Match { value, arms } = &expression.kind else {
                continue;
            };
            if is_match {
                let FeatureShape::Aggregate { field_count } = usage.shape() else {
                    continue;
                };
                if usize::try_from(*field_count).ok() != Some(arms.len()) {
                    continue;
                }
                owners.insert(match match_dispatch(program, *value, arms) {
                    JavaMatchDispatch::NativeEnum(_) => MatchOwner::Enums,
                    JavaMatchDispatch::General => MatchOwner::Patterns,
                });
            } else {
                for arm in arms {
                    if let CorePattern::EnumVariant { enumeration, .. } = &arm.pattern {
                        owners.insert(if payload_free_enum(program, *enumeration) {
                            MatchOwner::Enums
                        } else {
                            MatchOwner::Patterns
                        });
                    }
                }
            }
        }
        for owner in owners {
            // These typed slot reads are mandatory; no presence-only ID table.
            let capability = match owner {
                MatchOwner::Enums => {
                    let _ = self.registered::<Enums>();
                    CapabilityId::Enums
                }
                MatchOwner::Patterns => {
                    let _ = self.registered::<PatternMatching>();
                    CapabilityId::PatternMatching
                }
            };
            admission.prerequisites.push(capability);
        }
    }
}
