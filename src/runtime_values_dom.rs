#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomProp {
    Attributes,
    AssignedSlot,
    Value,
    Files,
    FilesLength,
    ValueAsNumber,
    ValueAsDate,
    ValueLength,
    ValidationMessage,
    Validity,
    ValidityValueMissing,
    ValidityTypeMismatch,
    ValidityPatternMismatch,
    ValidityTooLong,
    ValidityTooShort,
    ValidityRangeUnderflow,
    ValidityRangeOverflow,
    ValidityStepMismatch,
    ValidityBadInput,
    ValidityValid,
    ValidityCustomError,
    SelectionStart,
    SelectionEnd,
    SelectionDirection,
    Checked,
    Indeterminate,
    Open,
    ReturnValue,
    ClosedBy,
    Readonly,
    Required,
    Disabled,
    NodeType,
    TextContent,
    InnerText,
    InnerHtml,
    OuterHtml,
    ClassName,
    ClassList,
    ClassListLength,
    Part,
    PartLength,
    Id,
    TagName,
    LocalName,
    NamespaceUri,
    Prefix,
    NextElementSibling,
    PreviousElementSibling,
    Slot,
    Role,
    ElementTiming,
    HtmlFor,
    Name,
    Action,
    FormAction,
    Lang,
    Dir,
    AccessKey,
    AutoComplete,
    AutoCapitalize,
    AutoCorrect,
    ContentEditable,
    Draggable,
    EnterKeyHint,
    Inert,
    InputMode,
    Nonce,
    Popover,
    Spellcheck,
    TabIndex,
    Translate,
    Cite,
    DateTime,
    BrClear,
    CaptionAlign,
    ColSpan,
    TableCellColSpan,
    RowSpan,
    CanvasWidth,
    CanvasHeight,
    NodeEventHandler(String),
    BodyDeprecatedAttr(String),
    ClientWidth,
    ClientHeight,
    ClientLeft,
    ClientTop,
    CurrentCssZoom,
    OffsetWidth,
    OffsetHeight,
    OffsetLeft,
    OffsetTop,
    ScrollWidth,
    ScrollHeight,
    ScrollLeft,
    ScrollTop,
    ScrollLeftMax,
    ScrollTopMax,
    ShadowRoot,
    Dataset(String),
    Style(String),
    AriaString(String),
    AriaElementRefSingle(String),
    AriaElementRefList(String),
    ActiveElement,
    ActiveViewTransition,
    AdoptedStyleSheets,
    AdoptedStyleSheetsLength,
    CharacterSet,
    CompatMode,
    ContentType,
    ReadyState,
    Referrer,
    Title,
    Url,
    DocumentUri,
    BaseUri,
    Location,
    LocationHref,
    LocationProtocol,
    LocationHost,
    LocationHostname,
    LocationPort,
    LocationPathname,
    LocationSearch,
    LocationHash,
    LocationOrigin,
    LocationAncestorOrigins,
    History,
    HistoryLength,
    HistoryState,
    HistoryScrollRestoration,
    DefaultView,
    Hidden,
    VisibilityState,
    Forms,
    Images,
    Links,
    Scripts,
    Children,
    ChildElementCount,
    FirstElementChild,
    LastElementChild,
    CurrentScript,
    FormsLength,
    ImagesLength,
    LinksLength,
    ScriptsLength,
    ChildrenLength,
    AudioSrc,
    AudioAutoplay,
    AudioControls,
    AudioControlsList,
    AudioCrossOrigin,
    AudioDisableRemotePlayback,
    AudioLoop,
    AudioMuted,
    AudioPreload,
    VideoDisablePictureInPicture,
    VideoPlaysInline,
    VideoPoster,
    AnchorAlt,
    AnchorAttributionSrc,
    AnchorDownload,
    AnchorHash,
    AnchorHost,
    AnchorHostname,
    AnchorHref,
    AnchorHreflang,
    AnchorInterestForElement,
    AnchorOrigin,
    AnchorPassword,
    AnchorPathname,
    AnchorPing,
    AnchorPort,
    AnchorProtocol,
    AnchorReferrerPolicy,
    AnchorRel,
    AnchorRelList,
    AnchorRelListLength,
    AnchorSearch,
    AnchorTarget,
    AnchorText,
    AnchorType,
    AnchorUsername,
    AnchorNoHref,
    AnchorCharset,
    AnchorCoords,
    AnchorRev,
    AnchorShape,
    Data,
    SrcDoc,
    UseMap,
    Size,
    Min,
    Max,
    Step,
    MaxLength,
    MinLength,
    Rows,
    Cols,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomIndex {
    Static(usize),
    Dynamic(String),
}

impl DomIndex {
    pub(crate) fn describe(&self) -> String {
        match self {
            Self::Static(index) => index.to_string(),
            Self::Dynamic(expr) => expr.clone(),
        }
    }

    pub(crate) fn static_index(&self) -> Option<usize> {
        match self {
            Self::Static(index) => Some(*index),
            Self::Dynamic(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomQuery {
    DocumentRoot,
    DocumentBody,
    DocumentHead,
    DocumentElement,
    ActiveElement,
    ById(String),
    BySelector(String),
    BySelectorAll {
        selector: String,
    },
    BySelectorAllIndex {
        selector: String,
        index: DomIndex,
    },
    QuerySelector {
        target: Box<DomQuery>,
        selector: String,
    },
    QuerySelectorAll {
        target: Box<DomQuery>,
        selector: String,
    },
    Index {
        target: Box<DomQuery>,
        index: DomIndex,
    },
    QuerySelectorAllIndex {
        target: Box<DomQuery>,
        selector: String,
        index: DomIndex,
    },
    FormElementsIndex {
        form: Box<DomQuery>,
        index: DomIndex,
    },
    Var(String),
    VarPath {
        base: String,
        path: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FormDataSource {
    New {
        form: Option<DomQuery>,
        submitter: Option<DomQuery>,
    },
    Var(String),
}

impl DomQuery {
    pub(crate) fn describe_call(&self) -> String {
        match self {
            Self::DocumentRoot => "document".into(),
            Self::DocumentBody => "document.body".into(),
            Self::DocumentHead => "document.head".into(),
            Self::DocumentElement => "document.documentElement".into(),
            Self::ActiveElement => "document.activeElement".into(),
            Self::ById(id) => format!("document.getElementById('{id}')"),
            Self::BySelector(selector) => format!("document.querySelector('{selector}')"),
            Self::BySelectorAll { selector } => format!("document.querySelectorAll('{selector}')"),
            Self::BySelectorAllIndex { selector, index } => {
                format!(
                    "document.querySelectorAll('{selector}')[{}]",
                    index.describe()
                )
            }
            Self::QuerySelector { target, selector } => {
                format!("{}.querySelector('{selector}')", target.describe_call())
            }
            Self::QuerySelectorAll { target, selector } => {
                format!("{}.querySelectorAll('{selector}')", target.describe_call())
            }
            Self::Index { target, index } => {
                format!("{}[{}]", target.describe_call(), index.describe())
            }
            Self::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                format!(
                    "{}.querySelectorAll('{selector}')[{}]",
                    target.describe_call(),
                    index.describe()
                )
            }
            Self::FormElementsIndex { form, index } => {
                format!("{}.elements[{}]", form.describe_call(), index.describe())
            }
            Self::Var(name) => name.clone(),
            Self::VarPath { base, path } => format!("{base}.{}", path.join(".")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClassListMethod {
    Add,
    Remove,
    Toggle,
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinaryOp {
    Or,
    And,
    Nullish,
    Eq,
    Ne,
    StrictEq,
    StrictNe,
    BitOr,
    BitXor,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    Pow,
    Lt,
    Gt,
    Le,
    Ge,
    In,
    InstanceOf,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VarAssignOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    BitOr,
    BitXor,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    LogicalAnd,
    LogicalOr,
    Nullish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EventExprProp {
    Type,
    Target,
    CurrentTarget,
    TargetName,
    CurrentTargetName,
    DefaultPrevented,
    IsTrusted,
    Bubbles,
    Cancelable,
    TargetId,
    CurrentTargetId,
    EventPhase,
    TimeStamp,
    State,
    OldState,
    NewState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MatchMediaProp {
    Matches,
    Media,
}
