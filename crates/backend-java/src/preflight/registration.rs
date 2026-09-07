//! Registered capability ownership for exact checked feature uses.

use super::*;

impl JavaCapabilityRegistry {
    pub(crate) fn new(features: JavaCapabilitySet) -> Self {
        Self { features }
    }

    pub(super) fn registered<F>(&self) -> JavaFeatureOwner
    where
        F: portable_build::Capability,
        JavaCapabilitySet: Supports<F>,
    {
        let _ = self.features.mapping_for::<F>();
        #[cfg(test)]
        preflight_ownership_tests::record::<F>();
        JavaFeatureOwner::Mapping(F::ID)
    }

    pub(super) fn confirm_registered_mapping(&self, usage: &FeatureUse) -> JavaFeatureAdmission {
        let mut prerequisites = Vec::new();
        let _ = self.registered::<Modules>();
        prerequisites.push(CapabilityId::Modules);
        let owner = match usage.feature() {
            CoreFeature::Declaration(DeclarationFeature::Constant) => {
                self.registered::<Constants>()
            }
            CoreFeature::Declaration(DeclarationFeature::Alias) => self.registered::<TypeAliases>(),
            CoreFeature::Declaration(DeclarationFeature::Record) => self.registered::<Records>(),
            CoreFeature::Declaration(DeclarationFeature::Enum) => self.registered::<Enums>(),
            CoreFeature::Declaration(
                DeclarationFeature::Interface | DeclarationFeature::Implementation,
            ) => self.registered::<Interfaces>(),
            CoreFeature::Declaration(DeclarationFeature::Function) => {
                self.registered::<Functions>()
            }
            CoreFeature::Declaration(DeclarationFeature::Test) => {
                self.registered::<PortableTests>()
            }
            CoreFeature::Type(TypeFeature::Unit) => self.registered::<UnitValues>(),
            CoreFeature::Type(TypeFeature::Bool) => self.registered::<BoolValues>(),
            CoreFeature::Type(TypeFeature::I32) => self.registered::<I32Values>(),
            CoreFeature::Type(TypeFeature::I64) => self.registered::<I64Values>(),
            CoreFeature::Type(TypeFeature::F64) => self.registered::<F64Values>(),
            CoreFeature::Type(TypeFeature::Char) => self.registered::<CharValues>(),
            CoreFeature::Type(TypeFeature::String) => self.registered::<TextValues>(),
            CoreFeature::Type(TypeFeature::Bytes) => self.registered::<BytesValues>(),
            CoreFeature::Type(TypeFeature::List) => self.registered::<ListValues>(),
            CoreFeature::Type(TypeFeature::Option) => self.registered::<OptionValues>(),
            CoreFeature::Type(TypeFeature::Result) => self.registered::<ResultValues>(),
            CoreFeature::Type(TypeFeature::Record) => self.registered::<Records>(),
            CoreFeature::Type(TypeFeature::Enum) => self.registered::<Enums>(),
            CoreFeature::Type(TypeFeature::Interface) => self.registered::<Interfaces>(),
            CoreFeature::Control(ControlFeature::Let) => self.registered::<LocalBindings>(),
            CoreFeature::Control(ControlFeature::ForEach) => self.registered::<Loops>(),
            CoreFeature::Control(ControlFeature::Return) => self.registered::<Functions>(),
            CoreFeature::Control(ControlFeature::If) => self.registered::<Conditionals>(),
            CoreFeature::Control(ControlFeature::EnumPattern) => {
                JavaFeatureOwner::Structural(JavaStructuralAdmission::MatchDispatch)
            }
            CoreFeature::Control(
                ControlFeature::WildcardPattern
                | ControlFeature::BoolPattern
                | ControlFeature::NonePattern
                | ControlFeature::SomePattern
                | ControlFeature::OkPattern
                | ControlFeature::ErrPattern,
            ) => self.registered::<PatternMatching>(),
            CoreFeature::Control(ControlFeature::Block) => {
                JavaFeatureOwner::Structural(JavaStructuralAdmission::BlockAssembly)
            }
            CoreFeature::Control(ControlFeature::Evaluate) => {
                JavaFeatureOwner::Structural(JavaStructuralAdmission::EvaluationSequence)
            }
            CoreFeature::Control(ControlFeature::Match) => {
                JavaFeatureOwner::Structural(JavaStructuralAdmission::MatchDispatch)
            }
            CoreFeature::Interface(
                InterfaceFeature::Declaration
                | InterfaceFeature::Conformance
                | InterfaceFeature::MultipleConformance
                | InterfaceFeature::StaticDispatch
                | InterfaceFeature::DynamicDispatch
                | InterfaceFeature::InterfaceValue,
            ) => self.registered::<Interfaces>(),
            CoreFeature::Ownership(
                OwnershipFeature::OnceLeftToRight | OwnershipFeature::OwnedImmutableValue,
            ) => JavaFeatureOwner::Structural(JavaStructuralAdmission::OwnershipContract),
            CoreFeature::Operation(OperationFeature::Literal) => {
                JavaFeatureOwner::Structural(JavaStructuralAdmission::LiteralDispatch)
            }
            CoreFeature::Operation(OperationFeature::Local) => match usage.shape() {
                FeatureShape::LocalBinding(CoreLocalKind::Parameter) => {
                    self.registered::<Functions>()
                }
                FeatureShape::LocalBinding(CoreLocalKind::Let) => {
                    self.registered::<LocalBindings>()
                }
                FeatureShape::LocalBinding(CoreLocalKind::ForEach) => self.registered::<Loops>(),
                FeatureShape::LocalBinding(CoreLocalKind::Pattern) => {
                    self.registered::<PatternMatching>()
                }
                _ => unreachable!("local binding shape was validated"),
            },
            CoreFeature::Operation(OperationFeature::Call) => {
                let owner = self.registered::<Functions>();
                let _ = self.registered::<ResultPropagation>();
                prerequisites.push(CapabilityId::ResultPropagation);
                owner
            }
            CoreFeature::Operation(OperationFeature::Constant) => self.registered::<Constants>(),
            CoreFeature::Operation(OperationFeature::SelfValue) => self.registered::<Interfaces>(),
            CoreFeature::Operation(OperationFeature::ConstructRecord) => {
                self.registered::<Records>()
            }
            CoreFeature::Operation(OperationFeature::ConstructEnum) => self.registered::<Enums>(),
            CoreFeature::Operation(OperationFeature::Field) => self.registered::<Records>(),
            CoreFeature::Operation(OperationFeature::CoerceInterface) => {
                self.registered::<Interfaces>()
            }
            CoreFeature::Operation(
                OperationFeature::StaticMethodCall | OperationFeature::InterfaceCall,
            ) => {
                let owner = self.registered::<Interfaces>();
                let _ = self.registered::<ResultPropagation>();
                prerequisites.push(CapabilityId::ResultPropagation);
                owner
            }
            CoreFeature::Operation(OperationFeature::ConstructList) => {
                self.registered::<ListValues>()
            }
            CoreFeature::Operation(
                OperationFeature::ConstructSome | OperationFeature::ConstructNone,
            ) => self.registered::<OptionValues>(),
            CoreFeature::Operation(
                OperationFeature::ConstructOk | OperationFeature::ConstructErr,
            ) => self.registered::<ResultValues>(),
            CoreFeature::Operation(OperationFeature::Unary(operation)) => match operation {
                CoreUnaryIntrinsic::BoolNot => self.registered::<BooleanLogic>(),
                CoreUnaryIntrinsic::IntNegChecked => {
                    let owner = self.registered::<CheckedIntegerArithmetic>();
                    let _ = self.registered::<ResultPropagation>();
                    prerequisites.push(CapabilityId::ResultPropagation);
                    owner
                }
                CoreUnaryIntrinsic::IntNegWrapping => {
                    self.registered::<WrappingIntegerArithmetic>()
                }
                CoreUnaryIntrinsic::IntBitNot => self.registered::<IntegerBitwise>(),
                CoreUnaryIntrinsic::FloatNeg => self.registered::<FloatingPointArithmetic>(),
                CoreUnaryIntrinsic::FloatTrunc
                | CoreUnaryIntrinsic::FloatIsNaN
                | CoreUnaryIntrinsic::FloatIsNegativeZero
                | CoreUnaryIntrinsic::FloatAbs => self.registered::<FloatingPointInspection>(),
                CoreUnaryIntrinsic::StringScalarLength => {
                    let owner = self.registered::<StringInspection>();
                    let _ = self.registered::<ResultPropagation>();
                    prerequisites.push(CapabilityId::ResultPropagation);
                    owner
                }
                CoreUnaryIntrinsic::StringUtf16Length | CoreUnaryIntrinsic::StringIsEmpty => {
                    self.registered::<StringInspection>()
                }
                CoreUnaryIntrinsic::BytesLength | CoreUnaryIntrinsic::BytesIsEmpty => {
                    self.registered::<BytesOperations>()
                }
                CoreUnaryIntrinsic::ListLength | CoreUnaryIntrinsic::ListIsEmpty => {
                    self.registered::<ListOperations>()
                }
                CoreUnaryIntrinsic::OptionIsSome | CoreUnaryIntrinsic::OptionIsNone => {
                    self.registered::<OptionOperations>()
                }
                CoreUnaryIntrinsic::ResultIsOk | CoreUnaryIntrinsic::ResultIsErr => {
                    self.registered::<ResultOperations>()
                }
                CoreUnaryIntrinsic::WidenI32ToI64 => self.registered::<IntegerConversions>(),
                CoreUnaryIntrinsic::NarrowI64ToI32Checked => {
                    let owner = self.registered::<IntegerConversions>();
                    let _ = self.registered::<ResultPropagation>();
                    prerequisites.push(CapabilityId::ResultPropagation);
                    owner
                }
                CoreUnaryIntrinsic::StringToUtf8 => self.registered::<Utf8Conversions>(),
                CoreUnaryIntrinsic::StringFromUtf8Checked => {
                    let owner = self.registered::<Utf8Conversions>();
                    let _ = self.registered::<ResultPropagation>();
                    prerequisites.push(CapabilityId::ResultPropagation);
                    owner
                }
            },
            CoreFeature::Operation(OperationFeature::Binary(operation)) => match operation {
                CoreBinaryIntrinsic::BoolAnd | CoreBinaryIntrinsic::BoolOr => {
                    self.registered::<BooleanLogic>()
                }
                CoreBinaryIntrinsic::Equal | CoreBinaryIntrinsic::NotEqual => {
                    if matches!(
                        usage.shape(),
                        FeatureShape::Equality(EqualityOperandShape::PayloadFreeEnum)
                    ) {
                        self.registered::<Enums>()
                    } else {
                        self.registered::<Equality>()
                    }
                }
                CoreBinaryIntrinsic::Less
                | CoreBinaryIntrinsic::LessEqual
                | CoreBinaryIntrinsic::Greater
                | CoreBinaryIntrinsic::GreaterEqual => self.registered::<Ordering>(),
                CoreBinaryIntrinsic::IntAddChecked
                | CoreBinaryIntrinsic::IntSubChecked
                | CoreBinaryIntrinsic::IntMulChecked
                | CoreBinaryIntrinsic::IntDivChecked
                | CoreBinaryIntrinsic::IntRemChecked => {
                    let owner = self.registered::<CheckedIntegerArithmetic>();
                    let _ = self.registered::<ResultPropagation>();
                    prerequisites.push(CapabilityId::ResultPropagation);
                    owner
                }
                CoreBinaryIntrinsic::IntAddWrapping
                | CoreBinaryIntrinsic::IntSubWrapping
                | CoreBinaryIntrinsic::IntMulWrapping => {
                    self.registered::<WrappingIntegerArithmetic>()
                }
                CoreBinaryIntrinsic::FloatAdd
                | CoreBinaryIntrinsic::FloatSub
                | CoreBinaryIntrinsic::FloatMul
                | CoreBinaryIntrinsic::FloatDiv
                | CoreBinaryIntrinsic::FloatRemTrunc => {
                    self.registered::<FloatingPointArithmetic>()
                }
                CoreBinaryIntrinsic::StringConcat => self.registered::<StringConcatenation>(),
                CoreBinaryIntrinsic::IntBitAnd
                | CoreBinaryIntrinsic::IntBitOr
                | CoreBinaryIntrinsic::IntBitXor => self.registered::<IntegerBitwise>(),
                CoreBinaryIntrinsic::IntShiftLeftChecked
                | CoreBinaryIntrinsic::IntShiftRightChecked => {
                    let owner = self.registered::<CheckedIntegerShifts>();
                    let _ = self.registered::<ResultPropagation>();
                    prerequisites.push(CapabilityId::ResultPropagation);
                    owner
                }
                CoreBinaryIntrinsic::StringIndexOfLiteral
                | CoreBinaryIntrinsic::StringContains
                | CoreBinaryIntrinsic::StringStartsWith
                | CoreBinaryIntrinsic::StringEndsWith => self.registered::<StringInspection>(),
                CoreBinaryIntrinsic::StringStripPrefix
                | CoreBinaryIntrinsic::StringTruncateUtf8Bytes
                | CoreBinaryIntrinsic::StringTrimStart
                | CoreBinaryIntrinsic::StringTrimEnd => self.registered::<StringTransformation>(),
                CoreBinaryIntrinsic::BytesConcat => self.registered::<BytesOperations>(),
                CoreBinaryIntrinsic::ListGetChecked => {
                    let owner = self.registered::<ListOperations>();
                    let _ = self.registered::<ResultPropagation>();
                    prerequisites.push(CapabilityId::ResultPropagation);
                    owner
                }
                CoreBinaryIntrinsic::ListAppend
                | CoreBinaryIntrinsic::ListConcat
                | CoreBinaryIntrinsic::ListContains
                | CoreBinaryIntrinsic::ListIndexOf => self.registered::<ListOperations>(),
                CoreBinaryIntrinsic::OptionUnwrapOr => self.registered::<OptionOperations>(),
            },
            CoreFeature::Operation(OperationFeature::Ternary(
                CoreTernaryIntrinsic::StringSliceScalars | CoreTernaryIntrinsic::StringReplaceAll,
            )) => self.registered::<StringTransformation>(),
            CoreFeature::Operation(OperationFeature::Ternary(
                CoreTernaryIntrinsic::BytesReplaceAll,
            )) => self.registered::<BytesOperations>(),
            CoreFeature::Operation(OperationFeature::Variadic(
                CoreVariadicIntrinsic::StringReplaceMany,
            )) => self.registered::<StringTransformation>(),
            CoreFeature::Operation(OperationFeature::If) => self.registered::<Conditionals>(),
            CoreFeature::Operation(OperationFeature::Match) => {
                JavaFeatureOwner::Structural(JavaStructuralAdmission::MatchDispatch)
            }
            CoreFeature::Operation(OperationFeature::Block) => {
                JavaFeatureOwner::Structural(JavaStructuralAdmission::BlockAssembly)
            }
        };
        JavaFeatureAdmission {
            usage: usage.clone(),
            owner,
            prerequisites,
        }
    }
}
