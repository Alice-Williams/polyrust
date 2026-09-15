// Test-only opaque dependency values; their spelling/type comes from a closed kind.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Value(Kind);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Kind {
    First,
    Second,
    Known,
    QualifiedFirst,
    QualifiedOther,
    OwnedCollision,
    CallableCollision,
    CallableOtherOwner,
    QualifiedCollision,
    InvalidType,
    Missing,
}

pub(super) fn value(kind: Kind) -> Value {
    Value(kind)
}

pub(super) fn spec(value: &Value) -> DependencyValueSpec<TestDialect> {
    let owner = if matches!(value.0, Kind::QualifiedOther | Kind::CallableOtherOwner) {
        dependency_tests::Owner::Other
    } else {
        dependency_tests::Owner::First
    };
    let name = match value.0 {
        Kind::OwnedCollision => "entry",
        Kind::CallableCollision | Kind::CallableOtherOwner => "dependency",
        Kind::Second => "imported_flag",
        Kind::Known => "imported_known",
        Kind::Missing => "missing_constant",
        _ => "imported_constant",
    };
    let spelling = match value.0 {
        Kind::QualifiedFirst => DependencySpelling::Qualified(QualifiedName::DependencyValueFirst),
        Kind::QualifiedOther => DependencySpelling::Qualified(QualifiedName::DependencyValueOther),
        Kind::QualifiedCollision => DependencySpelling::Qualified(QualifiedName::DependencyFirst),
        _ => DependencySpelling::FixedImport(ImportKind::Value),
    };
    let ty = match value.0 {
        Kind::Second => TargetTypeRef::Primitive(Primitive::Bool),
        Kind::InvalidType => TargetTypeRef::Known(KnownType::Uncatalogued),
        Kind::Known => TargetTypeRef::Known(KnownType::ScalarAlias),
        _ => i64_type(),
    };
    DependencyValueSpec {
        symbol: value.clone(),
        owner,
        name: id(name),
        ty,
        spelling,
        source: source("certified-value"),
    }
}

pub(super) fn catalogue(mode: CatalogueMode) -> Vec<DependencyValueSpec<TestDialect>> {
    let mut kinds = vec![
        Kind::First,
        Kind::Second,
        Kind::Known,
        Kind::QualifiedFirst,
        Kind::QualifiedOther,
    ];
    match mode {
        CatalogueMode::DependencyValueOwnedCollision => kinds.push(Kind::OwnedCollision),
        CatalogueMode::DependencyValueCallableCollision
        | CatalogueMode::DependencyValueSeparateNamespace => kinds.push(Kind::CallableCollision),
        CatalogueMode::DependencyValueQualifiedCollision => kinds.push(Kind::QualifiedCollision),
        CatalogueMode::DependencyValueSeparateOwnerNamespace => {
            kinds.push(Kind::CallableOtherOwner)
        }
        CatalogueMode::DependencyValueInvalidType => kinds = vec![Kind::InvalidType],
        _ => {}
    }
    kinds.into_iter().map(|kind| spec(&value(kind))).collect()
}
