use super::script_ast_expr::*;
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntlLocaleMethod {
    GetCalendars,
    GetCollations,
    GetHourCycles,
    GetNumberingSystems,
    GetTextInfo,
    GetTimeZones,
    GetWeekInfo,
    Maximize,
    Minimize,
    ToString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StringTrimMode {
    Both,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StringStaticMethod {
    FromCharCode,
    FromCodePoint,
    Raw,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ObjectLiteralKey {
    Static(String),
    Computed(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ObjectLiteralEntry {
    Pair(ObjectLiteralKey, Expr),
    ProtoSetter(Expr),
    Getter(ObjectLiteralKey, ScriptHandler),
    Setter(ObjectLiteralKey, ScriptHandler),
    Spread(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MathConst {
    E,
    Ln10,
    Ln2,
    Log10E,
    Log2E,
    Pi,
    Sqrt1_2,
    Sqrt2,
    ToStringTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MathMethod {
    Abs,
    Acos,
    Acosh,
    Asin,
    Asinh,
    Atan,
    Atan2,
    Atanh,
    Cbrt,
    Ceil,
    Clz32,
    Cos,
    Cosh,
    Exp,
    Expm1,
    Floor,
    F16Round,
    FRound,
    Hypot,
    Imul,
    Log,
    Log10,
    Log1p,
    Log2,
    Max,
    Min,
    Pow,
    Random,
    Round,
    Sign,
    Sin,
    Sinh,
    Sqrt,
    SumPrecise,
    Tan,
    Tanh,
    Trunc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NumberConst {
    Epsilon,
    MaxSafeInteger,
    MaxValue,
    MinSafeInteger,
    MinValue,
    NaN,
    NegativeInfinity,
    PositiveInfinity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NumberMethod {
    IsFinite,
    IsInteger,
    IsNaN,
    IsSafeInteger,
    ParseFloat,
    ParseInt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NumberInstanceMethod {
    ToExponential,
    ToFixed,
    ToLocaleString,
    ToPrecision,
    ToString,
    ValueOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypedArrayStaticMethod {
    From,
    Of,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypedArrayInstanceMethod {
    At,
    CopyWithin,
    Entries,
    Fill,
    FindIndex,
    FindLast,
    FindLastIndex,
    IndexOf,
    Keys,
    LastIndexOf,
    ReduceRight,
    Reverse,
    Set,
    Sort,
    Subarray,
    ToReversed,
    ToSorted,
    Values,
    With,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MapStaticMethod {
    GroupBy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UrlStaticMethod {
    CanParse,
    Parse,
    CreateObjectUrl,
    RevokeObjectUrl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SymbolStaticMethod {
    For,
    KeyFor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SymbolStaticProperty {
    AsyncDispose,
    AsyncIterator,
    Dispose,
    HasInstance,
    IsConcatSpreadable,
    Iterator,
    Match,
    MatchAll,
    Replace,
    Search,
    Species,
    Split,
    ToPrimitive,
    ToStringTag,
    Unscopables,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegExpStaticMethod {
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PromiseStaticMethod {
    Resolve,
    Reject,
    All,
    AllSettled,
    Any,
    Race,
    Try,
    WithResolvers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MapInstanceMethod {
    Get,
    Has,
    Delete,
    Clear,
    ForEach,
    GetOrInsert,
    GetOrInsertComputed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UrlSearchParamsInstanceMethod {
    Append,
    Delete,
    GetAll,
    Has,
    Set,
    Sort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PromiseInstanceMethod {
    Then,
    Catch,
    Finally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SetInstanceMethod {
    Add,
    Union,
    Intersection,
    Difference,
    SymmetricDifference,
    IsDisjointFrom,
    IsSubsetOf,
    IsSupersetOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BigIntMethod {
    AsIntN,
    AsUintN,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BigIntInstanceMethod {
    ToLocaleString,
    ToString,
    ValueOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocationMethod {
    Assign,
    Reload,
    Replace,
    ToString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HistoryMethod {
    Back,
    Forward,
    Go,
    PushState,
    ReplaceState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClipboardMethod {
    ReadText,
    WriteText,
}
