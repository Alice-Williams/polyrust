//! Exact typed layout assertions for the current Linux LP64 no-call profile.
use crate::ast::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Object {
    Bool,
    Int,
    I32,
    Size,
    Pointer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Query {
    Size,
    Alignment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Check {
    pub object: Object,
    pub query: Query,
}

impl Object {
    fn ty(self) -> CObjectType {
        match self {
            Self::Bool => CObjectType::scalar(CScalarType::Bool),
            Self::Int => CObjectType::scalar(CScalarType::Int),
            Self::I32 => CObjectType::scalar(CScalarType::I32),
            Self::Size => CObjectType::scalar(CScalarType::Size),
            Self::Pointer => CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified)),
        }
    }
    fn from_type(ty: &CObjectType) -> Result<Self, String> {
        if ty.constness() != CConstness::Unqualified {
            return Err("qualified platform query type".into());
        }
        match ty.kind() {
            CObjectTypeKind::Scalar(CScalarType::Bool) => Ok(Self::Bool),
            CObjectTypeKind::Scalar(CScalarType::Int) => Ok(Self::Int),
            CObjectTypeKind::Scalar(CScalarType::I32) => Ok(Self::I32),
            CObjectTypeKind::Scalar(CScalarType::Size) => Ok(Self::Size),
            CObjectTypeKind::Pointer(CPointerTarget::Void(CConstness::Unqualified)) => {
                Ok(Self::Pointer)
            }
            _ => Err("type is outside the closed C platform query inventory".into()),
        }
    }
    pub(super) const fn bytes(self) -> u64 {
        match self {
            Self::Bool => 1,
            Self::Int | Self::I32 => 4,
            Self::Size | Self::Pointer => 8,
        }
    }
}

fn required() -> BTreeSet<Check> {
    [
        Object::Bool,
        Object::Int,
        Object::I32,
        Object::Size,
        Object::Pointer,
    ]
    .into_iter()
    .flat_map(|object| {
        [Query::Size, Query::Alignment]
            .into_iter()
            .map(move |query| Check { object, query })
    })
    .collect()
}

pub(super) fn classify(assertion: &CStaticAssertion) -> Result<Check, String> {
    let CValueKind::Binary {
        operator: CBinaryOperator::Equal,
        left,
        right,
    } = assertion.condition().kind()
    else {
        return Err("C platform assertion requires an exact layout equality".into());
    };
    let (query, ty) = match left.kind() {
        CValueKind::SizeOf(ty) => (Query::Size, ty),
        CValueKind::AlignOf(ty) => (Query::Alignment, ty),
        _ => return Err("C platform assertion requires sizeof or alignof".into()),
    };
    let object = Object::from_type(ty)?;
    if !matches!(right.kind(), CValueKind::Literal(CLiteral::Unsigned(CUnsignedLiteral::Size(bytes))) if *bytes == object.bytes())
    {
        return Err("C platform assertion has an incorrect expected layout".into());
    }
    Ok(Check { object, query })
}

pub(super) fn verify(source: &CSourceFile) -> Result<(), String> {
    let mut actual = BTreeSet::new();
    for item in source.items() {
        if let CFileItem::StaticAssert(assertion) = item
            && !actual.insert(classify(assertion)?)
        {
            return Err("duplicate C platform assertion".into());
        }
    }
    if actual != required() {
        return Err("missing C platform assertions".into());
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn install(
    registry: &CFrozenRegistry,
    source: CSourceFile,
) -> Result<CSourceFile, String> {
    Ok(install_package(registry, vec![source])?
        .pop()
        .expect("one installed source"))
}

pub(super) fn verify_package(sources: &[CSourceFile]) -> Result<(), String> {
    let ordered = super::profile::ordered_sources(sources)?;
    verify(ordered[0])
}

pub(super) fn install_package(
    registry: &CFrozenRegistry,
    sources: Vec<CSourceFile>,
) -> Result<Vec<CSourceFile>, String> {
    // Guard every supplied tree before cloning or contextual work. In a pair,
    // the header carries platform obligations for both implementation and users.
    super::profile::check_registered_package(registry.registrations(), &sources)?;
    let target = super::profile::ordered_sources(&sources)?[0]
        .identity()
        .clone();
    let installed = sources
        .into_iter()
        .map(|source| {
            if source.identity() == &target {
                install_checked(registry, source)
            } else {
                Ok(source)
            }
        })
        .collect::<Result<Vec<_>, String>>()?;
    super::profile::check_registered_package(registry.registrations(), &installed)?;
    verify_package(&installed)?;
    Ok(installed)
}

fn install_checked(registry: &CFrozenRegistry, source: CSourceFile) -> Result<CSourceFile, String> {
    if source
        .items()
        .iter()
        .any(|item| matches!(item, CFileItem::StaticAssert(_)))
    {
        verify(&source)?;
        return Ok(source);
    }
    let declarations = CDeclarations::new(registry.registrations(), source.identity().clone())
        .map_err(|e| e.to_string())?;
    let expressions = CExpressions::new(registry.registrations());
    let mut items = Vec::new();
    for check in required() {
        let query = match check.query {
            Query::Size => expressions.size_of(check.object.ty()),
            Query::Alignment => expressions.align_of(check.object.ty()),
        }
        .map_err(|e| e.to_string())?;
        let expected = expressions
            .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(
                check.object.bytes(),
            )))
            .map_err(|e| e.to_string())?;
        let condition = expressions
            .binary(CBinaryOperator::Equal, query, expected)
            .map_err(|e| e.to_string())?;
        let diagnostic = CAssertDiagnostic::new(
            format!("C profile {:?} {:?}", check.object, check.query).into_bytes(),
        );
        items.push(CFileItem::StaticAssert(
            declarations
                .static_assert(condition, diagnostic)
                .map_err(|e| e.to_string())?,
        ));
    }
    items.extend(source.items().iter().cloned());
    declarations.source_file(items).map_err(|e| e.to_string())
}
