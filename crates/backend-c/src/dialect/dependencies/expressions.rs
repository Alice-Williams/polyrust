//! Walk typed expression children; neither names nor cached result types suffice.

use super::{CFileDependencies, CTagDependency, CTypeRequirement};
use crate::ast::{
    CAggregateRef, CCall, CCallableKind, CConversion, CIndexBase, CLiteral, CObjectTypeKind,
    CPlace, CPlaceKind, CPointerTarget, CPointerTest, CValue, CValueKind,
};

impl CFileDependencies {
    pub(super) fn value(&mut self, value: &CValue) {
        self.object_type(value.ty(), CTypeRequirement::Declaration);
        match value.kind() {
            CValueKind::KnownConstant(known) => {
                self.headers.insert(known.header());
            }
            CValueKind::Call(call) => self.call(call),
            CValueKind::Literal(literal) => self.literal(literal),
            CValueKind::Read(place) => {
                self.object_type(place.ty(), CTypeRequirement::Complete);
                self.place(place);
            }
            CValueKind::AddressOf(place) => self.place(place),
            CValueKind::Enumerator(value) => self.tag(
                CTagDependency::Enum(value.owner().clone()),
                CTypeRequirement::Complete,
            ),
            CValueKind::FunctionAddress(function) => {
                self.functions.insert(function.clone());
                self.signature(function.signature(), CTypeRequirement::Declaration);
            }
            CValueKind::Unary { operand, .. } => self.value(operand),
            CValueKind::Binary { left, right, .. } => {
                self.value(left);
                self.value(right);
            }
            CValueKind::PointerTest(test) => match test {
                CPointerTest::IsNull(value) | CPointerTest::IsNonNull(value) => self.value(value),
                CPointerTest::SameSlot { left, right } => {
                    self.value(left);
                    self.value(right);
                }
            },
            CValueKind::Conditional {
                condition,
                then_value,
                else_value,
            } => {
                self.value(condition);
                self.value(then_value);
                self.value(else_value);
            }
            CValueKind::Convert {
                conversion,
                operand,
            } => {
                self.value(operand);
                match conversion {
                    CConversion::AddConst(ty) | CConversion::ObjectToVoid(ty) => {
                        self.object_type(ty, CTypeRequirement::Declaration)
                    }
                    CConversion::Numeric(scalar) => self.headers.extend(scalar.header()),
                    // Their concrete cast types are the value's registered type.
                    CConversion::AllocationRestore(_)
                    | CConversion::AdapterErase(_)
                    | CConversion::AdapterRestore(_) => {}
                }
            }
            CValueKind::SizeOf(ty) | CValueKind::AlignOf(ty) => {
                self.object_type(ty, CTypeRequirement::Complete)
            }
        }
    }

    pub(super) fn literal(&mut self, literal: &CLiteral) {
        self.object_type(&literal.ty(), CTypeRequirement::Declaration);
        if let CLiteral::NullPointer(pointer) = literal {
            self.object_type(pointer.declared_type(), CTypeRequirement::Declaration);
        }
    }

    pub(super) fn place(&mut self, place: &CPlace) {
        match place.kind() {
            CPlaceKind::Local(local) => self.object_type(local.ty(), CTypeRequirement::Declaration),
            CPlaceKind::Parameter(parameter) => {
                self.object_type(parameter.ty(), CTypeRequirement::Declaration)
            }
            CPlaceKind::Global(object) => {
                self.objects.insert(object.clone());
                self.object_type(object.ty(), CTypeRequirement::Declaration);
            }
            CPlaceKind::Member { base, member } => {
                self.members.insert(member.clone());
                self.place(base);
                self.aggregate(member.owner());
                self.object_type(member.ty(), CTypeRequirement::Declaration);
            }
            CPlaceKind::Dereference(pointer) => self.value(pointer),
            CPlaceKind::Index { base, index } => {
                match base {
                    CIndexBase::Array(place) => self.place(place),
                    CIndexBase::Pointer(value) => self.value(value),
                }
                self.object_type(place.ty(), CTypeRequirement::Complete);
                self.value(index);
            }
        }
    }

    pub(super) fn aggregate(&mut self, owner: &CAggregateRef) {
        self.tag(
            match owner {
                CAggregateRef::Struct(value) => CTagDependency::Struct(value.clone()),
                CAggregateRef::Union(value) => CTagDependency::Union(value.clone()),
            },
            CTypeRequirement::Complete,
        );
    }

    pub(super) fn call(&mut self, call: &CCall) {
        match call.callable().kind() {
            CCallableKind::Direct(function) => {
                self.signature(function.signature(), CTypeRequirement::Complete);
                self.functions.insert((**function).clone());
            }
            // A contract witness is proof metadata, not a rendered direct call.
            CCallableKind::Indirect { pointer, .. } => {
                let CObjectTypeKind::Pointer(CPointerTarget::Function(signature)) =
                    pointer.ty().kind()
                else {
                    unreachable!("authenticated indirect call requires a function pointer");
                };
                self.signature(signature, CTypeRequirement::Complete);
                self.value(pointer);
            }
            CCallableKind::Known(known) => {
                self.signature(&known.signature(), CTypeRequirement::Complete);
                self.headers.insert(known.header());
                self.libraries.extend(known.system_library());
            }
        }
        for argument in call.arguments() {
            self.value(argument);
        }
    }
}
