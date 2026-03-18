use super::script_ast_expr_enums::*;
use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
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
        args: Vec<Expr>,
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
    DateGetUTCFullYear(String),
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
    BooleanConstruct {
        value: Option<Box<Expr>>,
        called_with_new: bool,
    },
    NumberConstruct {
        value: Option<Box<Expr>>,
        called_with_new: bool,
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
    WeakMapConstruct {
        iterable: Option<Box<Expr>>,
        called_with_new: bool,
    },
    WeakMapConstructor,
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
    UrlSearchParamsConstructor,
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
    WeakSetConstruct {
        iterable: Option<Box<Expr>>,
        called_with_new: bool,
    },
    WeakSetConstructor,
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
    JsonStringify {
        value: Box<Expr>,
        replacer: Option<Box<Expr>>,
        space: Option<Box<Expr>>,
    },
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
    ObjectGetOwnPropertyDescriptor {
        object: Box<Expr>,
        key: Box<Expr>,
    },
    ObjectDefineProperty {
        object: Box<Expr>,
        key: Box<Expr>,
        descriptor: Box<Expr>,
    },
    ObjectGetOwnPropertyNames(Box<Expr>),
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
    ReflectSet {
        target: Box<Expr>,
        key: Box<Expr>,
        value: Box<Expr>,
        receiver: Option<Box<Expr>>,
    },
    ReflectOwnKeys(Box<Expr>),
    ObjectHasOwnProperty {
        target: String,
        key: Box<Expr>,
    },
    ArrayLiteral(Vec<Expr>),
    ArrayConstruct {
        args: Vec<Expr>,
        called_with_new: bool,
    },
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
    StructuredClone {
        value: Box<Expr>,
        options: Option<Box<Expr>>,
    },
    Fetch {
        request: Box<Expr>,
        options: Option<Box<Expr>>,
    },
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
    ImportCall {
        module: Box<Expr>,
        options: Option<Box<Expr>>,
    },
    Call {
        target: Box<Expr>,
        args: Vec<Expr>,
        optional: bool,
    },
    MemberCall {
        target: Box<Expr>,
        member: String,
        args: Vec<Expr>,
        optional: bool,
        optional_call: bool,
    },
    PrivateMemberCall {
        target: Box<Expr>,
        member: String,
        args: Vec<Expr>,
    },
    MemberGet {
        target: Box<Expr>,
        member: String,
        optional: bool,
    },
    PrivateMemberGet {
        target: Box<Expr>,
        member: String,
    },
    PrivateIn {
        member: String,
        target: Box<Expr>,
    },
    IndexGet {
        target: Box<Expr>,
        index: Box<Expr>,
        optional: bool,
    },
    ImportMeta,
    NewTarget,
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
        name: Option<String>,
        is_async: bool,
        is_generator: bool,
        is_arrow: bool,
        is_method: bool,
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
        form: Option<DomQuery>,
        submitter: Option<DomQuery>,
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
