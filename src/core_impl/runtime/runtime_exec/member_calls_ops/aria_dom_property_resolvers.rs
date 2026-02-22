use super::*;

impl Harness {
    pub(crate) fn eval_event_prop_fallback(
        &self,
        event_var: &str,
        value: &Value,
        prop: EventExprProp,
    ) -> Result<Value> {
        let read =
            |value: &Value, key: &str| self.object_property_from_named_value(event_var, value, key);
        match prop {
            EventExprProp::Type => read(value, "type"),
            EventExprProp::Target => read(value, "target"),
            EventExprProp::CurrentTarget => read(value, "currentTarget"),
            EventExprProp::TargetName => {
                let target = read(value, "target")?;
                read(&target, "name")
            }
            EventExprProp::CurrentTargetName => {
                let target = read(value, "currentTarget")?;
                read(&target, "name")
            }
            EventExprProp::DefaultPrevented => read(value, "defaultPrevented"),
            EventExprProp::IsTrusted => read(value, "isTrusted"),
            EventExprProp::Bubbles => read(value, "bubbles"),
            EventExprProp::Cancelable => read(value, "cancelable"),
            EventExprProp::TargetId => {
                let target = read(value, "target")?;
                read(&target, "id")
            }
            EventExprProp::CurrentTargetId => {
                let target = read(value, "currentTarget")?;
                read(&target, "id")
            }
            EventExprProp::EventPhase => read(value, "eventPhase"),
            EventExprProp::TimeStamp => read(value, "timeStamp"),
            EventExprProp::State => read(value, "state"),
            EventExprProp::OldState => read(value, "oldState"),
            EventExprProp::NewState => read(value, "newState"),
        }
    }

    pub(crate) fn aria_property_to_attr_name(prop_name: &str) -> String {
        if !prop_name.starts_with("aria") || prop_name.len() <= 4 {
            return prop_name.to_ascii_lowercase();
        }
        format!("aria-{}", prop_name[4..].to_ascii_lowercase())
    }

    pub(crate) fn aria_element_ref_attr_name(prop_name: &str) -> Option<&'static str> {
        match prop_name {
            "ariaActiveDescendantElement" => Some("aria-activedescendant"),
            "ariaControlsElements" => Some("aria-controls"),
            "ariaDescribedByElements" => Some("aria-describedby"),
            "ariaDetailsElements" => Some("aria-details"),
            "ariaErrorMessageElements" => Some("aria-errormessage"),
            "ariaFlowToElements" => Some("aria-flowto"),
            "ariaLabelledByElements" => Some("aria-labelledby"),
            "ariaOwnsElements" => Some("aria-owns"),
            _ => None,
        }
    }

    pub(crate) fn resolve_aria_single_element_property(
        &self,
        node: NodeId,
        prop_name: &str,
    ) -> Option<NodeId> {
        let attr_name = Self::aria_element_ref_attr_name(prop_name)?;
        let raw = self.dom.attr(node, attr_name)?;
        let id_ref = raw.split_whitespace().next()?;
        self.dom.by_id(id_ref)
    }

    pub(crate) fn resolve_aria_element_list_property(
        &self,
        node: NodeId,
        prop_name: &str,
    ) -> Vec<NodeId> {
        let Some(attr_name) = Self::aria_element_ref_attr_name(prop_name) else {
            return Vec::new();
        };
        let Some(raw) = self.dom.attr(node, attr_name) else {
            return Vec::new();
        };
        raw.split_whitespace()
            .filter_map(|id_ref| self.dom.by_id(id_ref))
            .collect()
    }

    pub(crate) fn object_key_from_dom_prop(prop: &DomProp) -> Option<&'static str> {
        match prop {
            DomProp::Attributes => Some("attributes"),
            DomProp::AssignedSlot => Some("assignedSlot"),
            DomProp::Value => Some("value"),
            DomProp::Files => Some("files"),
            DomProp::ValueAsNumber => Some("valueAsNumber"),
            DomProp::ValueAsDate => Some("valueAsDate"),
            DomProp::ValidationMessage => Some("validationMessage"),
            DomProp::Validity => Some("validity"),
            DomProp::SelectionStart => Some("selectionStart"),
            DomProp::SelectionEnd => Some("selectionEnd"),
            DomProp::SelectionDirection => Some("selectionDirection"),
            DomProp::Checked => Some("checked"),
            DomProp::Indeterminate => Some("indeterminate"),
            DomProp::Open => Some("open"),
            DomProp::ReturnValue => Some("returnValue"),
            DomProp::ClosedBy => Some("closedBy"),
            DomProp::Readonly => Some("readOnly"),
            DomProp::Required => Some("required"),
            DomProp::Disabled => Some("disabled"),
            DomProp::TextContent => Some("textContent"),
            DomProp::InnerText => Some("innerText"),
            DomProp::InnerHtml => Some("innerHTML"),
            DomProp::OuterHtml => Some("outerHTML"),
            DomProp::ClassName => Some("className"),
            DomProp::ClassList => Some("classList"),
            DomProp::Part => Some("part"),
            DomProp::Id => Some("id"),
            DomProp::TagName => Some("tagName"),
            DomProp::LocalName => Some("localName"),
            DomProp::NamespaceUri => Some("namespaceURI"),
            DomProp::Prefix => Some("prefix"),
            DomProp::NextElementSibling => Some("nextElementSibling"),
            DomProp::PreviousElementSibling => Some("previousElementSibling"),
            DomProp::Slot => Some("slot"),
            DomProp::Role => Some("role"),
            DomProp::ElementTiming => Some("elementTiming"),
            DomProp::Name => Some("name"),
            DomProp::Lang => Some("lang"),
            DomProp::Dir => Some("dir"),
            DomProp::Cite => Some("cite"),
            DomProp::DateTime => Some("dateTime"),
            DomProp::BrClear => Some("clear"),
            DomProp::CaptionAlign => Some("align"),
            DomProp::ColSpan => Some("span"),
            DomProp::CanvasWidth => Some("width"),
            DomProp::CanvasHeight => Some("height"),
            DomProp::ClientWidth => Some("clientWidth"),
            DomProp::ClientHeight => Some("clientHeight"),
            DomProp::ClientLeft => Some("clientLeft"),
            DomProp::ClientTop => Some("clientTop"),
            DomProp::CurrentCssZoom => Some("currentCSSZoom"),
            DomProp::OffsetWidth => Some("offsetWidth"),
            DomProp::OffsetHeight => Some("offsetHeight"),
            DomProp::OffsetLeft => Some("offsetLeft"),
            DomProp::OffsetTop => Some("offsetTop"),
            DomProp::ScrollWidth => Some("scrollWidth"),
            DomProp::ScrollHeight => Some("scrollHeight"),
            DomProp::ScrollLeft => Some("scrollLeft"),
            DomProp::ScrollTop => Some("scrollTop"),
            DomProp::ScrollLeftMax => Some("scrollLeftMax"),
            DomProp::ScrollTopMax => Some("scrollTopMax"),
            DomProp::ShadowRoot => Some("shadowRoot"),
            DomProp::Children => Some("children"),
            DomProp::ChildElementCount => Some("childElementCount"),
            DomProp::FirstElementChild => Some("firstElementChild"),
            DomProp::LastElementChild => Some("lastElementChild"),
            DomProp::Title => Some("title"),
            DomProp::BaseUri => Some("baseURI"),
            DomProp::AudioSrc => Some("src"),
            DomProp::AudioAutoplay => Some("autoplay"),
            DomProp::AudioControls => Some("controls"),
            DomProp::AudioControlsList => Some("controlsList"),
            DomProp::AudioCrossOrigin => Some("crossOrigin"),
            DomProp::AudioDisableRemotePlayback => Some("disableRemotePlayback"),
            DomProp::AudioLoop => Some("loop"),
            DomProp::AudioMuted => Some("muted"),
            DomProp::AudioPreload => Some("preload"),
            DomProp::AnchorAttributionSrc => Some("attributionSrc"),
            DomProp::AnchorDownload => Some("download"),
            DomProp::AnchorHash => Some("hash"),
            DomProp::AnchorHost => Some("host"),
            DomProp::AnchorHostname => Some("hostname"),
            DomProp::AnchorHref => Some("href"),
            DomProp::AnchorHreflang => Some("hreflang"),
            DomProp::AnchorInterestForElement => Some("interestForElement"),
            DomProp::AnchorOrigin => Some("origin"),
            DomProp::AnchorPassword => Some("password"),
            DomProp::AnchorPathname => Some("pathname"),
            DomProp::AnchorPing => Some("ping"),
            DomProp::AnchorPort => Some("port"),
            DomProp::AnchorProtocol => Some("protocol"),
            DomProp::AnchorReferrerPolicy => Some("referrerPolicy"),
            DomProp::AnchorRel => Some("rel"),
            DomProp::AnchorRelList => Some("relList"),
            DomProp::AnchorSearch => Some("search"),
            DomProp::AnchorTarget => Some("target"),
            DomProp::AnchorText => Some("text"),
            DomProp::AnchorType => Some("type"),
            DomProp::AnchorUsername => Some("username"),
            DomProp::AnchorCharset => Some("charset"),
            DomProp::AnchorCoords => Some("coords"),
            DomProp::AnchorRev => Some("rev"),
            DomProp::AnchorShape => Some("shape"),
            DomProp::Dataset(_)
            | DomProp::NodeEventHandler(_)
            | DomProp::BodyDeprecatedAttr(_)
            | DomProp::Style(_)
            | DomProp::ClassListLength
            | DomProp::PartLength
            | DomProp::FilesLength
            | DomProp::AriaString(_)
            | DomProp::AriaElementRefSingle(_)
            | DomProp::AriaElementRefList(_)
            | DomProp::ValueLength
            | DomProp::ValidityValueMissing
            | DomProp::ValidityTypeMismatch
            | DomProp::ValidityPatternMismatch
            | DomProp::ValidityTooLong
            | DomProp::ValidityTooShort
            | DomProp::ValidityRangeUnderflow
            | DomProp::ValidityRangeOverflow
            | DomProp::ValidityStepMismatch
            | DomProp::ValidityBadInput
            | DomProp::ValidityValid
            | DomProp::ValidityCustomError
            | DomProp::ActiveElement
            | DomProp::CharacterSet
            | DomProp::CompatMode
            | DomProp::ContentType
            | DomProp::ReadyState
            | DomProp::Referrer
            | DomProp::Url
            | DomProp::DocumentUri
            | DomProp::Location
            | DomProp::LocationHref
            | DomProp::LocationProtocol
            | DomProp::LocationHost
            | DomProp::LocationHostname
            | DomProp::LocationPort
            | DomProp::LocationPathname
            | DomProp::LocationSearch
            | DomProp::LocationHash
            | DomProp::LocationOrigin
            | DomProp::LocationAncestorOrigins
            | DomProp::History
            | DomProp::HistoryLength
            | DomProp::HistoryState
            | DomProp::HistoryScrollRestoration
            | DomProp::DefaultView
            | DomProp::Hidden
            | DomProp::VisibilityState
            | DomProp::Forms
            | DomProp::Images
            | DomProp::Links
            | DomProp::Scripts
            | DomProp::CurrentScript
            | DomProp::FormsLength
            | DomProp::ImagesLength
            | DomProp::LinksLength
            | DomProp::ScriptsLength
            | DomProp::ChildrenLength
            | DomProp::AnchorRelListLength => None,
        }
    }
}
