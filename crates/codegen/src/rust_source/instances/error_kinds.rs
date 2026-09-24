//! Fixed-size original error descriptions, never compiler or target authority.
use super::{
    RustCanonicalInstanceFacts, RustDeclarationId, RustInstanceDefinitionRole as Role,
    RustInstanceIdentityError, check_distinct,
};

/// The complete pinned core vocabulary, not just narrowing's two outcomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RustIntegerErrorKind {
    Empty,
    InvalidDigit,
    PosOverflow,
    NegOverflow,
    Zero,
    NotAPowerOfTwo,
}

impl RustIntegerErrorKind {
    pub const ALL: [Self; 6] = [
        Self::Empty,
        Self::InvalidDigit,
        Self::PosOverflow,
        Self::NegOverflow,
        Self::Zero,
        Self::NotAPowerOfTwo,
    ];

    /// Explicit version-2 transport encoding, NOT Rust ABI discriminant bytes.
    pub fn transport_code(self) -> i32 {
        match self {
            Self::Empty => 0,
            Self::InvalidDigit => 1,
            Self::PosOverflow => 2,
            Self::NegOverflow => 3,
            Self::Zero => 4,
            Self::NotAPowerOfTwo => 5,
        }
    }

    pub fn from_transport_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Empty),
            1 => Some(Self::InvalidDigit),
            2 => Some(Self::PosOverflow),
            3 => Some(Self::NegOverflow),
            4 => Some(Self::Zero),
            5 => Some(Self::NotAPowerOfTwo),
            _ => None,
        }
    }
}

/// Named roles avoid relying on caller array order or a target enum ordinal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RustIntegerErrorVariants {
    pub empty: RustDeclarationId,
    pub invalid_digit: RustDeclarationId,
    pub positive_overflow: RustDeclarationId,
    pub negative_overflow: RustDeclarationId,
    pub zero: RustDeclarationId,
    pub not_a_power_of_two: RustDeclarationId,
}

impl RustIntegerErrorVariants {
    pub fn definition(self, kind: RustIntegerErrorKind) -> RustDeclarationId {
        match kind {
            RustIntegerErrorKind::Empty => self.empty,
            RustIntegerErrorKind::InvalidDigit => self.invalid_digit,
            RustIntegerErrorKind::PosOverflow => self.positive_overflow,
            RustIntegerErrorKind::NegOverflow => self.negative_overflow,
            RustIntegerErrorKind::Zero => self.zero,
            RustIntegerErrorKind::NotAPowerOfTwo => self.not_a_power_of_two,
        }
    }
}

/// Descriptive wrapper/enum/variant facts joined to the original Result facts.
/// Constructors check consistency only. A compiler witness must authenticate
/// the private field's type, enum inventory, discriminants and layout; these
/// caller-supplied IDs cannot open a source capability or mint a certificate.
///
/// ```compile_fail,E0451
/// use portable_codegen::{RustCanonicalErrorKindFacts, RustCanonicalInstanceFacts,
///     RustIntegerErrorVariants, RustDeclarationId};
/// fn forge(instance: RustCanonicalInstanceFacts, id: RustDeclarationId,
///          variants: RustIntegerErrorVariants) -> RustCanonicalErrorKindFacts {
///     RustCanonicalErrorKindFacts { instance, wrapper_field: id,
///         kind_definition: id, variants }
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RustCanonicalErrorKindFacts {
    instance: RustCanonicalInstanceFacts,
    wrapper_field: RustDeclarationId,
    kind_definition: RustDeclarationId,
    variants: RustIntegerErrorVariants,
}

impl RustCanonicalErrorKindFacts {
    pub fn new(
        instance: RustCanonicalInstanceFacts,
        wrapper_field: RustDeclarationId,
        kind_definition: RustDeclarationId,
        variants: RustIntegerErrorVariants,
    ) -> Result<Self, RustInstanceIdentityError> {
        let original = [
            (Role::CoreRoot, instance.core_root()),
            (Role::Result, instance.key().result_definition()),
            (Role::Error, instance.key().error_definition()),
            (Role::Ok, instance.ok().variant),
            (Role::Err, instance.err().variant),
            (Role::OkPayload, instance.ok().payload),
            (Role::ErrPayload, instance.err().payload),
        ];
        let additional = [
            (Role::ErrorWrapperField, wrapper_field),
            (Role::ErrorKind, kind_definition),
            (
                Role::ErrorKindVariant(RustIntegerErrorKind::Empty),
                variants.empty,
            ),
            (
                Role::ErrorKindVariant(RustIntegerErrorKind::InvalidDigit),
                variants.invalid_digit,
            ),
            (
                Role::ErrorKindVariant(RustIntegerErrorKind::PosOverflow),
                variants.positive_overflow,
            ),
            (
                Role::ErrorKindVariant(RustIntegerErrorKind::NegOverflow),
                variants.negative_overflow,
            ),
            (
                Role::ErrorKindVariant(RustIntegerErrorKind::Zero),
                variants.zero,
            ),
            (
                Role::ErrorKindVariant(RustIntegerErrorKind::NotAPowerOfTwo),
                variants.not_a_power_of_two,
            ),
        ];
        for (index, &item) in additional.iter().enumerate() {
            for &other in original.iter().chain(&additional[..index]) {
                check_distinct(other, item)?;
            }
        }
        Ok(Self {
            instance,
            wrapper_field,
            kind_definition,
            variants,
        })
    }

    pub fn instance(self) -> RustCanonicalInstanceFacts {
        self.instance
    }

    pub fn wrapper_field(self) -> RustDeclarationId {
        self.wrapper_field
    }

    pub fn kind_definition(self) -> RustDeclarationId {
        self.kind_definition
    }

    pub fn variant(self, kind: RustIntegerErrorKind) -> RustDeclarationId {
        self.variants.definition(kind)
    }
}
