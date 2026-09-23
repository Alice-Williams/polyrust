//! Descriptive original source types, never reconstructed from a target primitive.
use super::{RustConstantValue, RustDeclarationId};
use std::collections::BTreeMap;

/// Char is a Unicode scalar; it is not interchangeable with I32 or a UTF-16 unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RustScalarKind {
    I32,
    I64,
    Bool,
    F64,
    Char,
}

impl RustScalarKind {
    pub fn spelling(self) -> &'static str {
        match self {
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::Bool => "bool",
            Self::F64 => "f64",
            Self::Char => "char",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RustResultKind {
    Unit,
    Scalar(RustScalarKind),
}

impl RustResultKind {
    pub fn spelling(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Scalar(kind) => kind.spelling(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustFunctionTypes {
    pub parameters: Vec<RustScalarKind>,
    pub result: RustResultKind,
}

impl RustFunctionTypes {
    pub fn contains_char(&self) -> bool {
        self.parameters.contains(&RustScalarKind::Char)
            || self.result == RustResultKind::Scalar(RustScalarKind::Char)
    }
}

/// A field's source scalar and enclosing nominal declaration travel together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RustFieldTypes {
    pub owner: RustDeclarationId,
    pub kind: RustScalarKind,
}

/// Bounded descriptive facts belonging to one original source owner.
///
/// Construction does not authenticate rustc analysis or grant target authority.
/// Compiler adapters must collect facts from original declarations; target APIs
/// reconcile them with their exact declaration/representation inventories.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustSourceTypes {
    root: RustDeclarationId,
    functions: BTreeMap<RustDeclarationId, RustFunctionTypes>,
    fields: BTreeMap<RustDeclarationId, RustFieldTypes>,
    constants: BTreeMap<RustDeclarationId, RustConstantValue>,
}

impl RustSourceTypes {
    pub fn new(
        root: RustDeclarationId,
        functions: BTreeMap<RustDeclarationId, RustFunctionTypes>,
        fields: BTreeMap<RustDeclarationId, RustFieldTypes>,
    ) -> Result<Self, String> {
        if functions.len() > 4096 || fields.len() > 100_000 {
            return Err("source type inventory exceeds declaration budget".into());
        }
        if functions
            .keys()
            .chain(fields.keys())
            .any(|id| id.crate_id != root.crate_id || *id == root)
            || fields.keys().any(|id| functions.contains_key(id))
            || fields.values().any(|field| {
                field.owner.crate_id != root.crate_id
                    || field.owner == root
                    || functions.contains_key(&field.owner)
                    || fields.contains_key(&field.owner)
            })
        {
            return Err("source type inventory has a foreign or conflicting declaration".into());
        }
        let mut count = fields.len();
        for function in functions.values() {
            count = count
                .checked_add(function.parameters.len())
                .and_then(|count| count.checked_add(1))
                .ok_or("source type inventory count overflow")?;
            if count > 100_000 {
                return Err("source type inventory exceeds scalar budget".into());
            }
        }
        Ok(Self {
            root,
            functions,
            fields,
            constants: BTreeMap::new(),
        })
    }

    /// Attach exactly the emitted original constants, not folded private reads.
    /// Ownership/budgets are checked; callers must still authenticate source facts.
    pub fn with_constants(
        mut self,
        constants: BTreeMap<RustDeclarationId, RustConstantValue>,
    ) -> Result<Self, String> {
        if constants.len() > 100_000 {
            return Err("source constant inventory exceeds declaration budget".into());
        }
        if constants.keys().any(|id| {
            id.crate_id != self.root.crate_id
                || *id == self.root
                || self.functions.contains_key(id)
                || self.fields.contains_key(id)
        }) || self
            .fields
            .values()
            .any(|field| constants.contains_key(&field.owner))
        {
            return Err(
                "source constant inventory has a foreign or conflicting declaration".into(),
            );
        }
        let mut count = self
            .fields
            .len()
            .checked_add(constants.len())
            .ok_or("source constant inventory count overflow")?;
        for function in self.functions.values() {
            count = count
                .checked_add(function.parameters.len())
                .and_then(|count| count.checked_add(1))
                .ok_or("source constant inventory count overflow")?;
        }
        if count > 100_000 {
            return Err("source constant inventory exceeds scalar budget".into());
        }
        self.constants = constants;
        Ok(self)
    }

    pub fn root(&self) -> RustDeclarationId {
        self.root
    }
    pub fn functions(&self) -> &BTreeMap<RustDeclarationId, RustFunctionTypes> {
        &self.functions
    }
    pub fn fields(&self) -> &BTreeMap<RustDeclarationId, RustFieldTypes> {
        &self.fields
    }
    pub fn constants(&self) -> &BTreeMap<RustDeclarationId, RustConstantValue> {
        &self.constants
    }
    pub fn contains_char(&self) -> bool {
        self.functions
            .values()
            .any(RustFunctionTypes::contains_char)
            || self
                .fields
                .values()
                .any(|field| field.kind == RustScalarKind::Char)
            || self
                .constants
                .values()
                .any(|value| value.kind() == RustScalarKind::Char)
    }
}

#[cfg(test)]
#[path = "constant_types_tests.rs"]
mod constant_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn id(crate_id: u64, definition_path_hash: u64) -> RustDeclarationId {
        RustDeclarationId {
            crate_id,
            definition_path_hash,
        }
    }

    fn field(kind: RustScalarKind) -> RustFieldTypes {
        RustFieldTypes {
            owner: id(1, 50),
            kind,
        }
    }

    #[test]
    fn source_char_and_integer_remain_distinct_without_target_types() {
        let signature = RustFunctionTypes {
            parameters: vec![RustScalarKind::Char, RustScalarKind::I32],
            result: RustResultKind::Scalar(RustScalarKind::Char),
        };
        let inventory = RustSourceTypes::new(
            id(1, 0),
            BTreeMap::from([(id(1, 1), signature.clone())]),
            BTreeMap::from([(id(1, 2), field(RustScalarKind::Char))]),
        )
        .unwrap();
        assert!(inventory.contains_char());
        assert_eq!(inventory.functions()[&id(1, 1)], signature);
        assert_ne!(signature.parameters[0], signature.parameters[1]);
        assert_eq!(signature.result.spelling(), "char");
        assert_eq!(RustResultKind::Unit.spelling(), "unit");
        assert!(
            !RustSourceTypes::new(id(1, 0), BTreeMap::new(), BTreeMap::new())
                .unwrap()
                .contains_char()
        );
    }

    #[test]
    fn source_type_owner_kind_and_resource_bounds_reject_malformed_metadata() {
        let signature = RustFunctionTypes {
            parameters: vec![],
            result: RustResultKind::Unit,
        };
        assert!(
            RustSourceTypes::new(
                id(1, 0),
                BTreeMap::from([(id(2, 1), signature.clone())]),
                BTreeMap::new()
            )
            .is_err()
        );
        assert!(
            RustSourceTypes::new(
                id(1, 0),
                BTreeMap::from([(id(1, 1), signature.clone())]),
                BTreeMap::from([(id(1, 1), field(RustScalarKind::Char))])
            )
            .is_err()
        );
        assert!(
            RustSourceTypes::new(
                id(1, 0),
                BTreeMap::new(),
                BTreeMap::from([(id(2, 1), field(RustScalarKind::Char))])
            )
            .is_err()
        );
        assert!(
            RustSourceTypes::new(
                id(1, 0),
                BTreeMap::from([(
                    id(1, 1),
                    RustFunctionTypes {
                        parameters: vec![RustScalarKind::Char; 100_000],
                        ..signature.clone()
                    }
                )]),
                BTreeMap::new()
            )
            .is_err()
        );
        assert!(
            RustSourceTypes::new(
                id(1, 0),
                (1..=4097)
                    .map(|index| (id(1, index), signature.clone()))
                    .collect(),
                BTreeMap::new()
            )
            .is_err()
        );
        for kind in [
            RustScalarKind::I32,
            RustScalarKind::I64,
            RustScalarKind::Bool,
            RustScalarKind::F64,
        ] {
            assert!(
                !RustSourceTypes::new(
                    id(1, 0),
                    BTreeMap::new(),
                    BTreeMap::from([(id(1, 1), field(kind))])
                )
                .unwrap()
                .contains_char()
            );
        }
    }

    #[test]
    fn field_owner_must_be_a_distinct_local_record_declaration() {
        for owner in [id(2, 50), id(1, 0), id(1, 1), id(1, 2)] {
            assert!(
                RustSourceTypes::new(
                    id(1, 0),
                    BTreeMap::from([(
                        id(1, 1),
                        RustFunctionTypes {
                            parameters: vec![],
                            result: RustResultKind::Unit,
                        }
                    )]),
                    BTreeMap::from([(
                        id(1, 2),
                        RustFieldTypes {
                            owner,
                            kind: RustScalarKind::Char,
                        }
                    )]),
                )
                .is_err()
            );
        }
    }
}
