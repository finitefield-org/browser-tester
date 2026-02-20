#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntlLocaleMethod {
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

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    String(String),
    Bool(bool),
    Null,
    Undefined,
    Number(i64),
    Float(f64),
    BigInt(JsBigInt),
    DateNow,
    PerformanceNow,
    DateNew {
        value: Option<Box<Expr>>,
    },
    DateParse(Box<Expr>),
    DateUtc {
        args: Vec<Expr>,
    },
    DateGetTime(String),
    DateSetTime {
        target: String,
        value: Box<Expr>,
    },
    DateToIsoString(String),
    DateGetFullYear(String),
    DateGetMonth(String),
    DateGetDate(String),
    DateGetHours(String),
    DateGetMinutes(String),
    DateGetSeconds(String),
    IntlFormatterConstruct {
        kind: IntlFormatterKind,
        locales: Option<Box<Expr>>,
        options: Option<Box<Expr>>,
        called_with_new: bool,
    },
    IntlFormat {
        formatter: Box<Expr>,
        value: Option<Box<Expr>>,
    },
    IntlFormatGetter {
        formatter: Box<Expr>,
    },
    IntlCollatorCompare {
        collator: Box<Expr>,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    IntlCollatorCompareGetter {
        collator: Box<Expr>,
    },
    IntlDateTimeFormatToParts {
        formatter: Box<Expr>,
        value: Option<Box<Expr>>,
    },
    IntlDateTimeFormatRange {
        formatter: Box<Expr>,
        start: Box<Expr>,
        end: Box<Expr>,
    },
    IntlDateTimeFormatRangeToParts {
        formatter: Box<Expr>,
        start: Box<Expr>,
        end: Box<Expr>,
    },
    IntlDateTimeResolvedOptions {
        formatter: Box<Expr>,
    },
    IntlDisplayNamesOf {
        display_names: Box<Expr>,
        code: Box<Expr>,
    },
    IntlPluralRulesSelect {
        plural_rules: Box<Expr>,
        value: Box<Expr>,
    },
    IntlPluralRulesSelectRange {
        plural_rules: Box<Expr>,
        start: Box<Expr>,
        end: Box<Expr>,
    },
    IntlRelativeTimeFormat {
        formatter: Box<Expr>,
        value: Box<Expr>,
        unit: Box<Expr>,
    },
    IntlRelativeTimeFormatToParts {
        formatter: Box<Expr>,
        value: Box<Expr>,
        unit: Box<Expr>,
    },
    IntlSegmenterSegment {
        segmenter: Box<Expr>,
        value: Box<Expr>,
    },
    IntlStaticMethod {
        method: IntlStaticMethod,
        args: Vec<Expr>,
    },
    IntlConstruct {
        args: Vec<Expr>,
    },
    IntlLocaleConstruct {
        tag: Box<Expr>,
        options: Option<Box<Expr>>,
        called_with_new: bool,
    },
    IntlLocaleMethod {
        locale: Box<Expr>,
        method: IntlLocaleMethod,
    },
    RegexLiteral {
        pattern: String,
        flags: String,
    },
    RegexNew {
        pattern: Box<Expr>,
        flags: Option<Box<Expr>>,
    },
    RegExpConstructor,
    RegExpStaticMethod {
        method: RegExpStaticMethod,
        args: Vec<Expr>,
    },
    RegexTest {
        regex: Box<Expr>,
        input: Box<Expr>,
    },
    RegexExec {
        regex: Box<Expr>,
        input: Box<Expr>,
    },
    RegexToString {
        regex: Box<Expr>,
    },
    MathConst(MathConst),
    MathMethod {
        method: MathMethod,
        args: Vec<Expr>,
    },
    StringConstruct {
        value: Option<Box<Expr>>,
        called_with_new: bool,
    },
    StringStaticMethod {
        method: StringStaticMethod,
        args: Vec<Expr>,
    },
    StringConstructor,
    NumberConstruct {
        value: Option<Box<Expr>>,
    },
    NumberConst(NumberConst),
    NumberMethod {
        method: NumberMethod,
        args: Vec<Expr>,
    },
    NumberInstanceMethod {
        value: Box<Expr>,
        method: NumberInstanceMethod,
        args: Vec<Expr>,
    },
    BigIntConstruct {
        value: Option<Box<Expr>>,
        called_with_new: bool,
    },
    BigIntMethod {
        method: BigIntMethod,
        args: Vec<Expr>,
    },
    BigIntInstanceMethod {
        value: Box<Expr>,
        method: BigIntInstanceMethod,
        args: Vec<Expr>,
    },
    BlobConstruct {
        parts: Option<Box<Expr>>,
        options: Option<Box<Expr>>,
        called_with_new: bool,
    },
    BlobConstructor,
    UrlConstruct {
        input: Option<Box<Expr>>,
        base: Option<Box<Expr>>,
        called_with_new: bool,
    },
    UrlConstructor,
    UrlStaticMethod {
        method: UrlStaticMethod,
        args: Vec<Expr>,
    },
    ArrayBufferConstruct {
        byte_length: Option<Box<Expr>>,
        options: Option<Box<Expr>>,
        called_with_new: bool,
    },
    ArrayBufferConstructor,
    ArrayBufferIsView(Box<Expr>),
    ArrayBufferDetached(String),
    ArrayBufferMaxByteLength(String),
    ArrayBufferResizable(String),
    ArrayBufferResize {
        target: String,
        new_byte_length: Box<Expr>,
    },
    ArrayBufferSlice {
        target: String,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    ArrayBufferTransfer {
        target: String,
        to_fixed_length: bool,
    },
    TypedArrayConstructorRef(TypedArrayConstructorKind),
    TypedArrayConstruct {
        kind: TypedArrayKind,
        args: Vec<Expr>,
        called_with_new: bool,
    },
    TypedArrayConstructWithCallee {
        callee: Box<Expr>,
        args: Vec<Expr>,
        called_with_new: bool,
    },
    PromiseConstruct {
        executor: Option<Box<Expr>>,
        called_with_new: bool,
    },
    PromiseConstructor,
    PromiseStaticMethod {
        method: PromiseStaticMethod,
        args: Vec<Expr>,
    },
    PromiseMethod {
        target: Box<Expr>,
        method: PromiseInstanceMethod,
        args: Vec<Expr>,
    },
    MapConstruct {
        iterable: Option<Box<Expr>>,
        called_with_new: bool,
    },
    MapConstructor,
    MapStaticMethod {
        method: MapStaticMethod,
        args: Vec<Expr>,
    },
    MapMethod {
        target: String,
        method: MapInstanceMethod,
        args: Vec<Expr>,
    },
    UrlSearchParamsConstruct {
        init: Option<Box<Expr>>,
        called_with_new: bool,
    },
    UrlSearchParamsMethod {
        target: String,
        method: UrlSearchParamsInstanceMethod,
        args: Vec<Expr>,
    },
    SetConstruct {
        iterable: Option<Box<Expr>>,
        called_with_new: bool,
    },
    SetConstructor,
    SetMethod {
        target: String,
        method: SetInstanceMethod,
        args: Vec<Expr>,
    },
    SymbolConstruct {
        description: Option<Box<Expr>>,
        called_with_new: bool,
    },
    SymbolConstructor,
    SymbolStaticMethod {
        method: SymbolStaticMethod,
        args: Vec<Expr>,
    },
    SymbolStaticProperty(SymbolStaticProperty),
    TypedArrayStaticBytesPerElement(TypedArrayKind),
    TypedArrayStaticMethod {
        kind: TypedArrayKind,
        method: TypedArrayStaticMethod,
        args: Vec<Expr>,
    },
    TypedArrayByteLength(String),
    TypedArrayByteOffset(String),
    TypedArrayBuffer(String),
    TypedArrayBytesPerElement(String),
    TypedArrayMethod {
        target: String,
        method: TypedArrayInstanceMethod,
        args: Vec<Expr>,
    },
    EncodeUri(Box<Expr>),
    EncodeUriComponent(Box<Expr>),
    DecodeUri(Box<Expr>),
    DecodeUriComponent(Box<Expr>),
    Escape(Box<Expr>),
    Unescape(Box<Expr>),
    IsNaN(Box<Expr>),
    IsFinite(Box<Expr>),
    Atob(Box<Expr>),
    Btoa(Box<Expr>),
    ParseInt {
        value: Box<Expr>,
        radix: Option<Box<Expr>>,
    },
    ParseFloat(Box<Expr>),
    JsonParse(Box<Expr>),
    JsonStringify(Box<Expr>),
    ObjectConstruct {
        value: Option<Box<Expr>>,
    },
    ObjectLiteral(Vec<ObjectLiteralEntry>),
    ObjectGet {
        target: String,
        key: String,
    },
    ObjectPathGet {
        target: String,
        path: Vec<String>,
    },
    ObjectGetOwnPropertySymbols(Box<Expr>),
    ObjectKeys(Box<Expr>),
    ObjectValues(Box<Expr>),
    ObjectEntries(Box<Expr>),
    ObjectHasOwn {
        object: Box<Expr>,
        key: Box<Expr>,
    },
    ObjectGetPrototypeOf(Box<Expr>),
    ObjectFreeze(Box<Expr>),
    ObjectHasOwnProperty {
        target: String,
        key: Box<Expr>,
    },
    ArrayLiteral(Vec<Expr>),
    ArrayIsArray(Box<Expr>),
    ArrayFrom {
        source: Box<Expr>,
        map_fn: Option<Box<Expr>>,
    },
    ArrayLength(String),
    ArrayIndex {
        target: String,
        index: Box<Expr>,
    },
    ArrayPush {
        target: String,
        args: Vec<Expr>,
    },
    ArrayPop(String),
    ArrayShift(String),
    ArrayUnshift {
        target: String,
        args: Vec<Expr>,
    },
    ArrayMap {
        target: String,
        callback: ScriptHandler,
    },
    ArrayFilter {
        target: String,
        callback: ScriptHandler,
    },
    ArrayReduce {
        target: String,
        callback: ScriptHandler,
        initial: Option<Box<Expr>>,
    },
    ArrayForEach {
        target: String,
        callback: ScriptHandler,
    },
    ArrayFind {
        target: String,
        callback: ScriptHandler,
    },
    ArrayFindIndex {
        target: String,
        callback: ScriptHandler,
    },
    ArraySome {
        target: String,
        callback: ScriptHandler,
    },
    ArrayEvery {
        target: String,
        callback: ScriptHandler,
    },
    ArrayIncludes {
        target: String,
        search: Box<Expr>,
        from_index: Option<Box<Expr>>,
    },
    ArraySlice {
        target: String,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    ArraySplice {
        target: String,
        start: Box<Expr>,
        delete_count: Option<Box<Expr>>,
        items: Vec<Expr>,
    },
    ArrayJoin {
        target: String,
        separator: Option<Box<Expr>>,
    },
    ArraySort {
        target: String,
        comparator: Option<Box<Expr>>,
    },
    StringTrim {
        value: Box<Expr>,
        mode: StringTrimMode,
    },
    StringToUpperCase(Box<Expr>),
    StringToLowerCase(Box<Expr>),
    StringIncludes {
        value: Box<Expr>,
        search: Box<Expr>,
        position: Option<Box<Expr>>,
    },
    StringStartsWith {
        value: Box<Expr>,
        search: Box<Expr>,
        position: Option<Box<Expr>>,
    },
    StringEndsWith {
        value: Box<Expr>,
        search: Box<Expr>,
        length: Option<Box<Expr>>,
    },
    StringSlice {
        value: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    StringSubstring {
        value: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    StringMatch {
        value: Box<Expr>,
        pattern: Box<Expr>,
    },
    StringSplit {
        value: Box<Expr>,
        separator: Option<Box<Expr>>,
        limit: Option<Box<Expr>>,
    },
    StringReplace {
        value: Box<Expr>,
        from: Box<Expr>,
        to: Box<Expr>,
    },
    StringReplaceAll {
        value: Box<Expr>,
        from: Box<Expr>,
        to: Box<Expr>,
    },
    StringIndexOf {
        value: Box<Expr>,
        search: Box<Expr>,
        position: Option<Box<Expr>>,
    },
    StringLastIndexOf {
        value: Box<Expr>,
        search: Box<Expr>,
        position: Option<Box<Expr>>,
    },
    StringCharAt {
        value: Box<Expr>,
        index: Option<Box<Expr>>,
    },
    StringCharCodeAt {
        value: Box<Expr>,
        index: Option<Box<Expr>>,
    },
    StringCodePointAt {
        value: Box<Expr>,
        index: Option<Box<Expr>>,
    },
    StringAt {
        value: Box<Expr>,
        index: Option<Box<Expr>>,
    },
    StringConcat {
        value: Box<Expr>,
        args: Vec<Expr>,
    },
    StringSearch {
        value: Box<Expr>,
        pattern: Box<Expr>,
    },
    StringRepeat {
        value: Box<Expr>,
        count: Box<Expr>,
    },
    StringPadStart {
        value: Box<Expr>,
        target_length: Box<Expr>,
        pad: Option<Box<Expr>>,
    },
    StringPadEnd {
        value: Box<Expr>,
        target_length: Box<Expr>,
        pad: Option<Box<Expr>>,
    },
    StringLocaleCompare {
        value: Box<Expr>,
        compare: Box<Expr>,
        locales: Option<Box<Expr>>,
        options: Option<Box<Expr>>,
    },
    StringIsWellFormed(Box<Expr>),
    StringToWellFormed(Box<Expr>),
    StringValueOf(Box<Expr>),
    StringToString(Box<Expr>),
    StructuredClone(Box<Expr>),
    Fetch(Box<Expr>),
    MatchMedia(Box<Expr>),
    MatchMediaProp {
        query: Box<Expr>,
        prop: MatchMediaProp,
    },
    Alert(Box<Expr>),
    Confirm(Box<Expr>),
    Prompt {
        message: Box<Expr>,
        default: Option<Box<Expr>>,
    },
    FunctionConstructor {
        args: Vec<Expr>,
    },
    FunctionCall {
        target: String,
        args: Vec<Expr>,
    },
    Call {
        target: Box<Expr>,
        args: Vec<Expr>,
    },
    MemberCall {
        target: Box<Expr>,
        member: String,
        args: Vec<Expr>,
        optional: bool,
    },
    MemberGet {
        target: Box<Expr>,
        member: String,
        optional: bool,
    },
    IndexGet {
        target: Box<Expr>,
        index: Box<Expr>,
        optional: bool,
    },
    Var(String),
    DomRef(DomQuery),
    CreateElement(String),
    CreateTextNode(String),
    SetTimeout {
        handler: TimerInvocation,
        delay_ms: Box<Expr>,
    },
    SetInterval {
        handler: TimerInvocation,
        delay_ms: Box<Expr>,
    },
    RequestAnimationFrame {
        callback: TimerCallback,
    },
    Function {
        handler: ScriptHandler,
        is_async: bool,
    },
    QueueMicrotask {
        handler: ScriptHandler,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    DomRead {
        target: DomQuery,
        prop: DomProp,
    },
    LocationMethodCall {
        method: LocationMethod,
        url: Option<Box<Expr>>,
    },
    HistoryMethodCall {
        method: HistoryMethod,
        args: Vec<Expr>,
    },
    ClipboardMethodCall {
        method: ClipboardMethod,
        args: Vec<Expr>,
    },
    DocumentHasFocus,
    DomMatches {
        target: DomQuery,
        selector: String,
    },
    DomClosest {
        target: DomQuery,
        selector: String,
    },
    DomComputedStyleProperty {
        target: DomQuery,
        property: String,
    },
    ClassListContains {
        target: DomQuery,
        class_name: String,
    },
    QuerySelectorAllLength {
        target: DomQuery,
    },
    FormElementsLength {
        form: DomQuery,
    },
    FormDataNew {
        form: DomQuery,
    },
    FormDataGet {
        source: FormDataSource,
        name: String,
    },
    FormDataHas {
        source: FormDataSource,
        name: String,
    },
    FormDataGetAll {
        source: FormDataSource,
        name: String,
    },
    FormDataGetAllLength {
        source: FormDataSource,
        name: String,
    },
    DomGetAttribute {
        target: DomQuery,
        name: String,
    },
    DomHasAttribute {
        target: DomQuery,
        name: String,
    },
    EventProp {
        event_var: String,
        prop: EventExprProp,
    },
    Neg(Box<Expr>),
    Pos(Box<Expr>),
    BitNot(Box<Expr>),
    Not(Box<Expr>),
    Void(Box<Expr>),
    Delete(Box<Expr>),
    TypeOf(Box<Expr>),
    Await(Box<Expr>),
    Yield(Box<Expr>),
    YieldStar(Box<Expr>),
    Comma(Vec<Expr>),
    Spread(Box<Expr>),
    Add(Vec<Expr>),
    Ternary {
        cond: Box<Expr>,
        on_true: Box<Expr>,
        on_false: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventMethod {
    PreventDefault,
    StopPropagation,
    StopImmediatePropagation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringTrimMode {
    Both,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringStaticMethod {
    FromCharCode,
    FromCodePoint,
    Raw,
}

#[derive(Debug, Clone, PartialEq)]
enum ObjectLiteralKey {
    Static(String),
    Computed(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
enum ObjectLiteralEntry {
    Pair(ObjectLiteralKey, Expr),
    Spread(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MathConst {
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
enum MathMethod {
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
enum NumberConst {
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
enum NumberMethod {
    IsFinite,
    IsInteger,
    IsNaN,
    IsSafeInteger,
    ParseFloat,
    ParseInt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumberInstanceMethod {
    ToExponential,
    ToFixed,
    ToLocaleString,
    ToPrecision,
    ToString,
    ValueOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypedArrayStaticMethod {
    From,
    Of,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypedArrayInstanceMethod {
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
enum MapStaticMethod {
    GroupBy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UrlStaticMethod {
    CanParse,
    Parse,
    CreateObjectUrl,
    RevokeObjectUrl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolStaticMethod {
    For,
    KeyFor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolStaticProperty {
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
enum RegExpStaticMethod {
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromiseStaticMethod {
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
enum MapInstanceMethod {
    Get,
    Has,
    Delete,
    Clear,
    ForEach,
    GetOrInsert,
    GetOrInsertComputed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UrlSearchParamsInstanceMethod {
    Append,
    Delete,
    GetAll,
    Has,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromiseInstanceMethod {
    Then,
    Catch,
    Finally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetInstanceMethod {
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
enum BigIntMethod {
    AsIntN,
    AsUintN,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BigIntInstanceMethod {
    ToLocaleString,
    ToString,
    ValueOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeTreeMethod {
    After,
    Append,
    AppendChild,
    Before,
    ReplaceWith,
    Prepend,
    RemoveChild,
    InsertBefore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsertAdjacentPosition {
    BeforeBegin,
    AfterBegin,
    BeforeEnd,
    AfterEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListenerRegistrationOp {
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq)]
enum Stmt {
    VarDecl {
        name: String,
        expr: Expr,
    },
    FunctionDecl {
        name: String,
        handler: ScriptHandler,
        is_async: bool,
    },
    VarAssign {
        name: String,
        op: VarAssignOp,
        expr: Expr,
    },
    VarUpdate {
        name: String,
        delta: i8,
    },
    ArrayDestructureAssign {
        targets: Vec<Option<String>>,
        expr: Expr,
    },
    ObjectDestructureAssign {
        bindings: Vec<(String, String)>,
        expr: Expr,
    },
    ObjectAssign {
        target: String,
        path: Vec<Expr>,
        expr: Expr,
    },
    FormDataAppend {
        target_var: String,
        name: Expr,
        value: Expr,
    },
    DomAssign {
        target: DomQuery,
        prop: DomProp,
        expr: Expr,
    },
    ClassListCall {
        target: DomQuery,
        method: ClassListMethod,
        class_names: Vec<String>,
        force: Option<Expr>,
    },
    ClassListForEach {
        target: DomQuery,
        item_var: String,
        index_var: Option<String>,
        body: Vec<Stmt>,
    },
    DomSetAttribute {
        target: DomQuery,
        name: String,
        value: Expr,
    },
    DomRemoveAttribute {
        target: DomQuery,
        name: String,
    },
    NodeTreeMutation {
        target: DomQuery,
        method: NodeTreeMethod,
        child: Expr,
        reference: Option<Expr>,
    },
    InsertAdjacentElement {
        target: DomQuery,
        position: InsertAdjacentPosition,
        node: Expr,
    },
    InsertAdjacentText {
        target: DomQuery,
        position: InsertAdjacentPosition,
        text: Expr,
    },
    InsertAdjacentHTML {
        target: DomQuery,
        position: Expr,
        html: Expr,
    },
    SetTimeout {
        handler: TimerInvocation,
        delay_ms: Expr,
    },
    SetInterval {
        handler: TimerInvocation,
        delay_ms: Expr,
    },
    QueueMicrotask {
        handler: ScriptHandler,
    },
    ClearTimeout {
        timer_id: Expr,
    },
    NodeRemove {
        target: DomQuery,
    },
    ForEach {
        target: Option<DomQuery>,
        selector: String,
        item_var: String,
        index_var: Option<String>,
        body: Vec<Stmt>,
    },
    ArrayForEach {
        target: String,
        callback: ScriptHandler,
    },
    ArrayForEachExpr {
        target: Expr,
        callback: ScriptHandler,
    },
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        post: Option<Box<Stmt>>,
        body: Vec<Stmt>,
    },
    ForIn {
        item_var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    ForOf {
        item_var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    DoWhile {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Try {
        try_stmts: Vec<Stmt>,
        catch_binding: Option<CatchBinding>,
        catch_stmts: Option<Vec<Stmt>>,
        finally_stmts: Option<Vec<Stmt>>,
    },
    Throw {
        value: Expr,
    },
    Return {
        value: Option<Expr>,
    },
    If {
        cond: Expr,
        then_stmts: Vec<Stmt>,
        else_stmts: Vec<Stmt>,
    },
    EventCall {
        event_var: String,
        method: EventMethod,
    },
    ListenerMutation {
        target: DomQuery,
        op: ListenerRegistrationOp,
        event_type: String,
        capture: bool,
        handler: ScriptHandler,
    },
    DispatchEvent {
        target: DomQuery,
        event_type: Expr,
    },
    DomMethodCall {
        target: DomQuery,
        method: DomMethod,
        arg: Option<Expr>,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
enum CatchBinding {
    Identifier(String),
    ArrayPattern(Vec<Option<String>>),
    ObjectPattern(Vec<(String, String)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecFlow {
    Continue,
    Break,
    ContinueLoop,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomMethod {
    Focus,
    Blur,
    Click,
    ScrollIntoView,
    Submit,
    RequestSubmit,
    Reset,
    Show,
    ShowModal,
    Close,
    RequestClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocationMethod {
    Assign,
    Reload,
    Replace,
    ToString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HistoryMethod {
    Back,
    Forward,
    Go,
    PushState,
    ReplaceState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipboardMethod {
    ReadText,
    WriteText,
}
