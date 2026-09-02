use portable_ir::v0::{F64Bits, SourceRef, Visibility};
use serde::Serialize;

use crate::{
    CoreAliasId, CoreBlockId, CoreConstantId, CoreEnumId, CoreExprId, CoreFieldId, CoreFunctionId,
    CoreImplementationId, CoreImplementationMethodId, CoreInterfaceId, CoreInterfaceMethodId,
    CoreLocalId, CoreRecordId, CoreTestId, CoreTypeId, CoreVariantId,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum CoreType {
    Unit,
    Bool,
    I32,
    I64,
    F64,
    Char,
    String,
    Bytes,
    List(CoreTypeId),
    Option(CoreTypeId),
    Result { ok: CoreTypeId, error: CoreTypeId },
    Record(CoreRecordId),
    Enum(CoreEnumId),
    Interface(CoreInterfaceId),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CoreTypeArena {
    values: Vec<CoreType>,
}

impl CoreTypeArena {
    pub fn get(&self, id: CoreTypeId) -> Option<&CoreType> {
        self.values.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CoreTypeId, &CoreType)> {
        self.values
            .iter()
            .enumerate()
            .map(|(index, ty)| (CoreTypeId::from_index(index), ty))
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub(crate) fn push(&mut self, ty: CoreType) -> CoreTypeId {
        let id = CoreTypeId::from_index(self.values.len());
        self.values.push(ty);
        id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreDeclarationHeader {
    pub name: String,
    pub visibility: Visibility,
    pub documentation: Vec<String>,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreMemberHeader {
    pub name: String,
    pub documentation: Vec<String>,
    pub source: SourceRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum CoreDeclaration {
    Constant(CoreConstantId),
    Alias(CoreAliasId),
    Record(CoreRecordId),
    Enum(CoreEnumId),
    Interface(CoreInterfaceId),
    Implementation(CoreImplementationId),
    Function(CoreFunctionId),
    Test(CoreTestId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreModule {
    pub name: String,
    pub declarations: Vec<CoreDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreConstant {
    pub header: CoreDeclarationHeader,
    pub ty: CoreTypeId,
    pub value: CoreConstantExpr,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreAlias {
    pub header: CoreDeclarationHeader,
    pub target: CoreTypeId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreRecord {
    pub header: CoreDeclarationHeader,
    pub fields: Vec<CoreFieldId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreEnum {
    pub header: CoreDeclarationHeader,
    pub variants: Vec<CoreVariantId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreVariant {
    pub header: CoreMemberHeader,
    pub enumeration: CoreEnumId,
    pub fields: Vec<CoreFieldId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum CoreFieldOwner {
    Record(CoreRecordId),
    Variant(CoreVariantId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreField {
    pub header: CoreMemberHeader,
    pub owner: CoreFieldOwner,
    pub ty: CoreTypeId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreInterface {
    pub header: CoreDeclarationHeader,
    pub methods: Vec<CoreInterfaceMethodId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreInterfaceMethod {
    pub header: CoreMemberHeader,
    pub interface: CoreInterfaceId,
    pub receiver: InterfaceReceiver,
    pub parameters: Vec<CoreParameter>,
    pub return_type: CoreTypeId,
}

/// Closed portable receiver semantics for interface methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceReceiver {
    Immutable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreImplementation {
    pub header: CoreDeclarationHeader,
    pub interface: CoreInterfaceId,
    pub record: CoreRecordId,
    pub methods: Vec<CoreImplementationMethodId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreImplementationMethod {
    pub header: CoreMemberHeader,
    pub implementation: CoreImplementationId,
    pub interface_method: CoreInterfaceMethodId,
    pub parameters: Vec<CoreParameter>,
    pub return_type: CoreTypeId,
    pub body: CoreBlockId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreFunction {
    pub header: CoreDeclarationHeader,
    pub parameters: Vec<CoreParameter>,
    pub return_type: CoreTypeId,
    pub body: CoreBlockId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreParameter {
    pub header: CoreMemberHeader,
    pub ty: CoreTypeId,
    pub local: Option<CoreLocalId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreLocalKind {
    Parameter,
    Let,
    ForEach,
    Pattern,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreLocal {
    pub name: String,
    pub ty: CoreTypeId,
    pub kind: CoreLocalKind,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreTest {
    pub header: CoreDeclarationHeader,
    pub invocation: CoreTestInvocation,
    pub expected: CoreExpectedOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum CoreTestInvocation {
    Function {
        function: CoreFunctionId,
        arguments: Vec<CoreTypedValue>,
    },
    Method {
        implementation: CoreImplementationId,
        method: CoreImplementationMethodId,
        receiver: CoreTypedValue,
        arguments: Vec<CoreTypedValue>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum CoreExpectedOutcome {
    Value(CoreTypedValue),
    Error(CoreTypedValue),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreTypedValue {
    pub ty: CoreTypeId,
    pub value: CoreValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum CoreValue {
    Unit,
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(F64Bits),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<CoreValue>),
    None,
    Some(Box<CoreValue>),
    Ok(Box<CoreValue>),
    Err(Box<CoreValue>),
    Record {
        record: CoreRecordId,
        fields: Vec<CoreValueField>,
    },
    Enum {
        enumeration: CoreEnumId,
        variant: CoreVariantId,
        fields: Vec<CoreValueField>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreValueField {
    pub field: CoreFieldId,
    pub value: CoreValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreConstantExpr {
    pub source: SourceRef,
    pub kind: CoreConstantExprKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum CoreConstantExprKind {
    Literal(CoreValue),
    Constant(CoreConstantId),
    Record {
        record: CoreRecordId,
        fields: Vec<CoreConstantField>,
    },
    Enum {
        enumeration: CoreEnumId,
        variant: CoreVariantId,
        fields: Vec<CoreConstantField>,
    },
    Some(Box<CoreConstantExpr>),
    None {
        inner: CoreTypeId,
    },
    Ok {
        value: Box<CoreConstantExpr>,
        error: CoreTypeId,
    },
    Err {
        value: Box<CoreConstantExpr>,
        ok: CoreTypeId,
    },
    List {
        element: CoreTypeId,
        elements: Vec<CoreConstantExpr>,
    },
    Intrinsic(Box<CoreIntrinsicExpr<CoreConstantExpr>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreConstantField {
    pub field: CoreFieldId,
    pub value: CoreConstantExpr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreEvaluationOrder {
    OnceLeftToRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreResultOwnership {
    OwnedImmutableValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreExpr {
    pub source: SourceRef,
    pub ty: CoreTypeId,
    pub evaluation: CoreEvaluationOrder,
    pub ownership: CoreResultOwnership,
    pub kind: CoreExprKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CoreExprArena {
    values: Vec<CoreExpr>,
}

impl CoreExprArena {
    pub fn get(&self, id: CoreExprId) -> Option<&CoreExpr> {
        self.values.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CoreExprId, &CoreExpr)> {
        self.values
            .iter()
            .enumerate()
            .map(|(index, value)| (CoreExprId::from_index(index), value))
    }

    pub(crate) fn push(&mut self, expression: CoreExpr) -> CoreExprId {
        let id = CoreExprId::from_index(self.values.len());
        self.values.push(expression);
        id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum CoreExprKind {
    Literal(CoreValue),
    Local(CoreLocalId),
    Constant(CoreConstantId),
    SelfValue(CoreRecordId),
    ConstructRecord {
        record: CoreRecordId,
        fields: Vec<CoreExprField>,
    },
    ConstructEnum {
        enumeration: CoreEnumId,
        variant: CoreVariantId,
        fields: Vec<CoreExprField>,
    },
    ConstructSome(CoreExprId),
    ConstructNone {
        inner: CoreTypeId,
    },
    ConstructOk {
        value: CoreExprId,
        error: CoreTypeId,
    },
    ConstructErr {
        value: CoreExprId,
        ok: CoreTypeId,
    },
    ConstructList {
        element: CoreTypeId,
        elements: Vec<CoreExprId>,
    },
    /// An owned interface value paired with the exact explicit conformance
    /// witness selected by the checked source program.
    CoerceInterface {
        implementation: CoreImplementationId,
        value: CoreExprId,
    },
    Field {
        value: CoreExprId,
        field: CoreFieldId,
    },
    Call {
        function: CoreFunctionId,
        arguments: Vec<CoreExprId>,
    },
    StaticMethodCall {
        implementation: CoreImplementationId,
        method: CoreImplementationMethodId,
        receiver: CoreExprId,
        arguments: Vec<CoreExprId>,
    },
    InterfaceCall {
        interface: CoreInterfaceId,
        method: CoreInterfaceMethodId,
        receiver: CoreExprId,
        arguments: Vec<CoreExprId>,
    },
    Intrinsic(CoreIntrinsicExpr<CoreExprId>),
    If {
        condition: CoreExprId,
        then_block: CoreBlockId,
        else_block: CoreBlockId,
    },
    Match {
        value: CoreExprId,
        arms: Vec<CoreMatchArm>,
    },
    Block(CoreBlockId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreExprField {
    pub field: CoreFieldId,
    pub value: CoreExprId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreMatchArm {
    pub source: SourceRef,
    pub pattern: CorePattern,
    pub body: CoreBlockId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum CorePattern {
    Wildcard {
        source: SourceRef,
    },
    Bool {
        source: SourceRef,
        value: bool,
    },
    EnumVariant {
        source: SourceRef,
        enumeration: CoreEnumId,
        variant: CoreVariantId,
        bindings: Vec<CoreFieldBinding>,
    },
    None {
        source: SourceRef,
    },
    Some {
        source: SourceRef,
        binding: CoreLocalId,
    },
    Ok {
        source: SourceRef,
        binding: CoreLocalId,
    },
    Err {
        source: SourceRef,
        binding: CoreLocalId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreFieldBinding {
    pub field: CoreFieldId,
    pub binding: CoreLocalId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreBlock {
    pub source: SourceRef,
    pub statements: Vec<CoreStatement>,
    pub result: Option<CoreExprId>,
    pub result_type: CoreTypeId,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CoreBlockArena {
    values: Vec<CoreBlock>,
}

impl CoreBlockArena {
    pub fn get(&self, id: CoreBlockId) -> Option<&CoreBlock> {
        self.values.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CoreBlockId, &CoreBlock)> {
        self.values
            .iter()
            .enumerate()
            .map(|(index, value)| (CoreBlockId::from_index(index), value))
    }

    pub(crate) fn push(&mut self, block: CoreBlock) -> CoreBlockId {
        let id = CoreBlockId::from_index(self.values.len());
        self.values.push(block);
        id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum CoreStatement {
    Let {
        source: SourceRef,
        local: CoreLocalId,
        value: CoreExprId,
    },
    ForEach {
        source: SourceRef,
        binding: CoreLocalId,
        iterable: CoreExprId,
        body: CoreBlockId,
    },
    Return {
        source: SourceRef,
        value: Option<CoreExprId>,
    },
    Evaluate {
        source: SourceRef,
        value: CoreExprId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "arity", content = "data", rename_all = "snake_case")]
pub enum CoreIntrinsicExpr<T> {
    Unary {
        operation: CoreUnaryIntrinsic,
        operand: T,
    },
    Binary {
        operation: CoreBinaryIntrinsic,
        left: T,
        right: T,
    },
    Ternary {
        operation: CoreTernaryIntrinsic,
        first: T,
        second: T,
        third: T,
    },
    Variadic {
        operation: CoreVariadicIntrinsic,
        arguments: Vec<T>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreUnaryIntrinsic {
    BoolNot,
    IntNegChecked,
    IntNegWrapping,
    IntBitNot,
    FloatNeg,
    FloatTrunc,
    FloatIsNaN,
    FloatIsNegativeZero,
    FloatAbs,
    StringScalarLength,
    StringUtf16Length,
    StringIsEmpty,
    BytesLength,
    BytesIsEmpty,
    ListLength,
    ListIsEmpty,
    OptionIsSome,
    OptionIsNone,
    ResultIsOk,
    ResultIsErr,
    WidenI32ToI64,
    NarrowI64ToI32Checked,
    StringToUtf8,
    StringFromUtf8Checked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreBinaryIntrinsic {
    BoolAnd,
    BoolOr,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    IntAddChecked,
    IntSubChecked,
    IntMulChecked,
    IntDivChecked,
    IntRemChecked,
    IntAddWrapping,
    IntSubWrapping,
    IntMulWrapping,
    IntBitAnd,
    IntBitOr,
    IntBitXor,
    IntShiftLeftChecked,
    IntShiftRightChecked,
    FloatAdd,
    FloatSub,
    FloatMul,
    FloatDiv,
    FloatRemTrunc,
    StringConcat,
    StringIndexOfLiteral,
    StringContains,
    StringStartsWith,
    StringStripPrefix,
    StringEndsWith,
    StringTruncateUtf8Bytes,
    StringTrimStart,
    StringTrimEnd,
    BytesConcat,
    ListGetChecked,
    ListAppend,
    ListConcat,
    ListContains,
    ListIndexOf,
    OptionUnwrapOr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreTernaryIntrinsic {
    StringSliceScalars,
    StringReplaceAll,
    BytesReplaceAll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreVariadicIntrinsic {
    StringReplaceMany,
}

/// Canonical semantic input accepted by typed target lowerers.
///
/// Production code can only obtain this proof object through checked lowering;
/// its arenas and constructor are deliberately private.
///
/// ```compile_fail
/// use portable_core_ir::CoreProgram;
///
/// let _forged = CoreProgram {};
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreProgram {
    module: CoreModule,
    types: CoreTypeArena,
    constants: Vec<CoreConstant>,
    aliases: Vec<CoreAlias>,
    records: Vec<CoreRecord>,
    enums: Vec<CoreEnum>,
    variants: Vec<CoreVariant>,
    fields: Vec<CoreField>,
    interfaces: Vec<CoreInterface>,
    interface_methods: Vec<CoreInterfaceMethod>,
    implementations: Vec<CoreImplementation>,
    implementation_methods: Vec<CoreImplementationMethod>,
    functions: Vec<CoreFunction>,
    tests: Vec<CoreTest>,
    locals: Vec<CoreLocal>,
    expressions: CoreExprArena,
    blocks: CoreBlockArena,
}

impl CoreProgram {
    pub fn module(&self) -> &CoreModule {
        &self.module
    }

    pub fn types(&self) -> &CoreTypeArena {
        &self.types
    }

    pub fn expressions(&self) -> &CoreExprArena {
        &self.expressions
    }

    pub fn blocks(&self) -> &CoreBlockArena {
        &self.blocks
    }

    pub fn constants(&self) -> &[CoreConstant] {
        &self.constants
    }

    pub fn aliases(&self) -> &[CoreAlias] {
        &self.aliases
    }

    pub fn records(&self) -> &[CoreRecord] {
        &self.records
    }

    pub fn enums(&self) -> &[CoreEnum] {
        &self.enums
    }

    pub fn variants(&self) -> &[CoreVariant] {
        &self.variants
    }

    pub fn fields(&self) -> &[CoreField] {
        &self.fields
    }

    pub fn interfaces(&self) -> &[CoreInterface] {
        &self.interfaces
    }

    pub fn interface_methods(&self) -> &[CoreInterfaceMethod] {
        &self.interface_methods
    }

    pub fn implementations(&self) -> &[CoreImplementation] {
        &self.implementations
    }

    pub fn implementation_methods(&self) -> &[CoreImplementationMethod] {
        &self.implementation_methods
    }

    pub fn functions(&self) -> &[CoreFunction] {
        &self.functions
    }

    pub fn tests(&self) -> &[CoreTest] {
        &self.tests
    }

    pub fn locals(&self) -> &[CoreLocal] {
        &self.locals
    }

    pub fn constant(&self, id: CoreConstantId) -> Option<&CoreConstant> {
        self.constants.get(id.index())
    }

    pub fn alias(&self, id: CoreAliasId) -> Option<&CoreAlias> {
        self.aliases.get(id.index())
    }

    pub fn record(&self, id: CoreRecordId) -> Option<&CoreRecord> {
        self.records.get(id.index())
    }

    pub fn enumeration(&self, id: CoreEnumId) -> Option<&CoreEnum> {
        self.enums.get(id.index())
    }

    pub fn variant(&self, id: CoreVariantId) -> Option<&CoreVariant> {
        self.variants.get(id.index())
    }

    pub fn field(&self, id: CoreFieldId) -> Option<&CoreField> {
        self.fields.get(id.index())
    }

    pub fn interface(&self, id: CoreInterfaceId) -> Option<&CoreInterface> {
        self.interfaces.get(id.index())
    }

    pub fn interface_method(&self, id: CoreInterfaceMethodId) -> Option<&CoreInterfaceMethod> {
        self.interface_methods.get(id.index())
    }

    pub fn implementation(&self, id: CoreImplementationId) -> Option<&CoreImplementation> {
        self.implementations.get(id.index())
    }

    pub fn implementation_method(
        &self,
        id: CoreImplementationMethodId,
    ) -> Option<&CoreImplementationMethod> {
        self.implementation_methods.get(id.index())
    }

    pub fn function(&self, id: CoreFunctionId) -> Option<&CoreFunction> {
        self.functions.get(id.index())
    }

    pub fn test(&self, id: CoreTestId) -> Option<&CoreTest> {
        self.tests.get(id.index())
    }

    pub fn local(&self, id: CoreLocalId) -> Option<&CoreLocal> {
        self.locals.get(id.index())
    }

    pub fn canonical_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("CoreIR always serializes")
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        module: CoreModule,
        types: CoreTypeArena,
        constants: Vec<CoreConstant>,
        aliases: Vec<CoreAlias>,
        records: Vec<CoreRecord>,
        enums: Vec<CoreEnum>,
        variants: Vec<CoreVariant>,
        fields: Vec<CoreField>,
        interfaces: Vec<CoreInterface>,
        interface_methods: Vec<CoreInterfaceMethod>,
        implementations: Vec<CoreImplementation>,
        implementation_methods: Vec<CoreImplementationMethod>,
        functions: Vec<CoreFunction>,
        tests: Vec<CoreTest>,
        locals: Vec<CoreLocal>,
        expressions: CoreExprArena,
        blocks: CoreBlockArena,
    ) -> Self {
        Self {
            module,
            types,
            constants,
            aliases,
            records,
            enums,
            variants,
            fields,
            interfaces,
            interface_methods,
            implementations,
            implementation_methods,
            functions,
            tests,
            locals,
            expressions,
            blocks,
        }
    }

    #[cfg(test)]
    pub(crate) fn expressions_mut(&mut self) -> &mut CoreExprArena {
        &mut self.expressions
    }

    #[cfg(test)]
    pub(crate) fn module_mut(&mut self) -> &mut CoreModule {
        &mut self.module
    }
}

#[cfg(test)]
impl CoreExprArena {
    pub(crate) fn get_mut(&mut self, id: CoreExprId) -> Option<&mut CoreExpr> {
        self.values.get_mut(id.index())
    }
}
