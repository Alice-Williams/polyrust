use super::{CValueBinding, Writer};
use crate::ast::{
    CBinaryOperator, CCallableKind, CConversion, CInitializer, CInitializerKind, CLiteral, CPlace,
    CPlaceKind, CSignedLiteral, CUnaryOperator, CUnsignedLiteral, CValue, CValueKind,
};

impl Writer<'_> {
    pub(super) fn value(&self, value: &CValue) -> String {
        match value.kind() {
            CValueKind::Call(call) => self.call(call),
            CValueKind::Literal(CLiteral::F64(value)) => {
                let parts = value.parts();
                let token = format!("0x{:x}.0p{}", parts.significand(), parts.exponent());
                match parts.sign() {
                    portable_binary64::Binary64Sign::Positive => token,
                    portable_binary64::Binary64Sign::Negative => format!("(-{token})"),
                }
            }
            CValueKind::Literal(CLiteral::Bool(value)) => if *value { "1" } else { "0" }.into(),
            CValueKind::Literal(CLiteral::Unsigned(CUnsignedLiteral::U32(value))) => {
                format!("(({}) {value}U)", self.scalar(crate::ast::CScalarType::U32))
            }
            CValueKind::Literal(CLiteral::Unsigned(CUnsignedLiteral::U64(value))) => {
                // LP64 uint64_t need not be unsigned long long. Keep exact type.
                format!(
                    "(({}) {value}ULL)",
                    self.scalar(crate::ast::CScalarType::U64)
                )
            }
            CValueKind::Literal(CLiteral::Signed(CSignedLiteral::I64(value))) => {
                let literal = if *value == i64::MIN {
                    "(-9223372036854775807LL - 1LL)".into()
                } else if *value < 0 {
                    format!("({value}LL)")
                } else {
                    format!("{value}LL")
                };
                // int64_t need not be a typedef of long long on LP64. Keep the
                // rendered expression's type equal to its certified AST type.
                format!(
                    "(({}) {literal})",
                    self.scalar(crate::ast::CScalarType::I64)
                )
            }
            CValueKind::Literal(CLiteral::Signed(
                CSignedLiteral::I32(value) | CSignedLiteral::Int(value),
            )) => {
                if *value == i32::MIN {
                    "(-2147483647 - 1)".into()
                } else if *value < 0 {
                    format!("({value})")
                } else {
                    value.to_string()
                }
            }
            CValueKind::Read(place) => self.place(place),
            CValueKind::AddressOf(place) => format!("(&{})", self.place(place)),
            CValueKind::Unary {
                operator: CUnaryOperator::LogicalNot,
                operand,
            } => format!("(!{})", self.value(operand)),
            CValueKind::Unary {
                operator: CUnaryOperator::BitNot,
                operand,
            } => format!("(~{})", self.value(operand)),
            CValueKind::Unary {
                operator: CUnaryOperator::Negate,
                operand,
            } => format!("(-{})", self.value(operand)),
            CValueKind::Conditional {
                condition,
                then_value,
                else_value,
            } => format!(
                "({} ? {} : {})",
                self.value(condition),
                self.value(then_value),
                self.value(else_value)
            ),
            CValueKind::Binary {
                operator,
                left,
                right,
            } => {
                let op = match operator {
                    CBinaryOperator::Add => "+",
                    CBinaryOperator::Subtract => "-",
                    CBinaryOperator::Multiply => "*",
                    CBinaryOperator::Divide => "/",
                    CBinaryOperator::Equal => "==",
                    CBinaryOperator::NotEqual => "!=",
                    CBinaryOperator::Less => "<",
                    CBinaryOperator::LessEqual => "<=",
                    CBinaryOperator::Greater => ">",
                    CBinaryOperator::GreaterEqual => ">=",
                    CBinaryOperator::BitAnd => "&",
                    CBinaryOperator::BitOr => "|",
                    CBinaryOperator::BitXor => "^",
                    _ => unreachable!("checked operator profile"),
                };
                format!("({} {op} {})", self.value(left), self.value(right))
            }
            CValueKind::Convert {
                conversion,
                operand,
            } => {
                let ty = match conversion {
                    CConversion::Numeric(scalar) => self.scalar(*scalar).to_owned(),
                    CConversion::AddConst(ty) => self.declarator(ty, ""),
                    _ => unreachable!("checked conversion profile"),
                };
                format!("(({ty}) {})", self.value(operand))
            }
            _ => unreachable!("checked expression profile"),
        }
    }
    pub(super) fn call(&self, call: &crate::ast::CCall) -> String {
        let name = match call.callable().kind() {
            CCallableKind::Direct(function) => self.names.functions[function.as_ref()].as_str(),
            CCallableKind::Known(callable) => callable.spelling(),
            CCallableKind::Indirect { .. } => unreachable!("checked call profile"),
        };
        format!(
            "{}({})",
            name,
            call.arguments()
                .iter()
                .map(|value| self.value(value))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    pub(super) fn place(&self, place: &CPlace) -> String {
        match place.kind() {
            CPlaceKind::Global(object) => self.names.values[&CValueBinding::Global(object.clone())]
                .as_str()
                .into(),
            CPlaceKind::Local(local) => self.names.values[&CValueBinding::Local(local.clone())]
                .as_str()
                .into(),
            CPlaceKind::Parameter(parameter) => self.names.values
                [&CValueBinding::Parameter(parameter.clone())]
                .as_str()
                .into(),
            CPlaceKind::Member { base, member } => format!(
                "({}).{}",
                self.place(base),
                self.names.values[&CValueBinding::Member(member.clone())].as_str()
            ),
            CPlaceKind::Dereference(value) => format!("(*{})", self.value(value)),
            _ => unreachable!("checked place profile"),
        }
    }
    pub(super) fn initializer(&self, value: &CInitializer) -> String {
        match value.kind() {
            CInitializerKind::Expression(value) => self.value(value),
            CInitializerKind::Struct { members, .. } => format!(
                "{{ {} }}",
                members
                    .iter()
                    .map(|(member, value)| format!(
                        ".{} = {}",
                        self.names.values[&CValueBinding::Member(member.clone())].as_str(),
                        self.initializer(value)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            _ => unreachable!("checked initializer profile"),
        }
    }
}
