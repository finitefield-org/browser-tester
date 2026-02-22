use super::*;

pub(crate) fn parse_set_interval_expr(src: &str) -> Result<Option<(TimerInvocation, Expr)>> {
    let mut cursor = Cursor::new(src);
    let Some((handler, delay_ms)) = parse_set_interval_call(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((handler, delay_ms)))
}

pub(crate) fn parse_set_timeout_expr(src: &str) -> Result<Option<(TimerInvocation, Expr)>> {
    let mut cursor = Cursor::new(src);
    let Some((handler, delay_ms)) = parse_set_timeout_call(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((handler, delay_ms)))
}

pub(crate) fn parse_request_animation_frame_expr(src: &str) -> Result<Option<TimerCallback>> {
    let mut cursor = Cursor::new(src);
    let callback = parse_request_animation_frame_call(&mut cursor)?;
    cursor.skip_ws();
    if cursor.eof() { Ok(callback) } else { Ok(None) }
}

pub(crate) fn parse_request_animation_frame_call(
    cursor: &mut Cursor<'_>,
) -> Result<Option<TimerCallback>> {
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("requestAnimationFrame") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "requestAnimationFrame requires exactly one argument".into(),
        ));
    }

    let callback_arg = strip_js_comments(args[0]);
    let callback = parse_timer_callback("requestAnimationFrame", callback_arg.as_str().trim())?;
    Ok(Some(callback))
}

pub(crate) fn parse_queue_microtask_expr(src: &str) -> Result<Option<ScriptHandler>> {
    let mut cursor = Cursor::new(src);
    let handler = parse_queue_microtask_call(&mut cursor)?;
    cursor.skip_ws();
    if cursor.eof() { Ok(handler) } else { Ok(None) }
}

pub(crate) fn parse_queue_microtask_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let Some(handler) = parse_queue_microtask_call(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported queueMicrotask statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::QueueMicrotask { handler }))
}

pub(crate) fn parse_queue_microtask_call(cursor: &mut Cursor<'_>) -> Result<Option<ScriptHandler>> {
    cursor.skip_ws();
    if !cursor.consume_ascii("queueMicrotask") {
        return Ok(None);
    }

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() {
        return Err(Error::ScriptParse(
            "queueMicrotask requires 1 argument".into(),
        ));
    }
    if args.len() != 1 {
        return Err(Error::ScriptParse(
            "queueMicrotask supports only 1 argument".into(),
        ));
    }

    let callback_arg = strip_js_comments(args[0]);
    let mut callback_cursor = Cursor::new(callback_arg.as_str().trim());
    let (params, body, _) = parse_callback(&mut callback_cursor, 1, "callback parameters")?;
    callback_cursor.skip_ws();
    if !callback_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported queueMicrotask callback: {}",
            args[0].trim()
        )));
    }

    Ok(Some(ScriptHandler {
        params,
        stmts: parse_block_statements(&body)?,
    }))
}

pub(crate) fn starts_with_window_member_access(src: &str) -> bool {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("window") {
        return false;
    }
    cursor.skip_ws();
    cursor.consume_byte(b'.')
}

pub(crate) fn is_non_dom_var_target(target: &DomQuery) -> bool {
    match target {
        DomQuery::Var(_) | DomQuery::VarPath { .. } => true,
        DomQuery::Index { target, .. } => is_non_dom_var_target(target),
        _ => false,
    }
}

pub(crate) fn parse_dom_access(src: &str) -> Result<Option<(DomQuery, DomProp)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();

    let head = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse(format!("expected property name in: {src}")))?;

    cursor.skip_ws();
    let nested = if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        Some(
            cursor
                .parse_identifier()
                .ok_or_else(|| Error::ScriptParse(format!("expected nested property in: {src}")))?,
        )
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let is_anchor_target = !matches!(target, DomQuery::DocumentRoot)
        && !matches!(
            &target,
            DomQuery::Var(name)
                if matches!(
                    name.as_str(),
                    "location" | "history" | "window" | "document" | "navigator" | "clipboard"
                )
        );
    let is_media_target = is_anchor_target;

    let prop = match (head.as_str(), nested.as_ref()) {
        ("attributes", None) => DomProp::Attributes,
        ("assignedSlot", None) => DomProp::AssignedSlot,
        ("value", None) => DomProp::Value,
        ("files", None) => DomProp::Files,
        ("files", Some(length)) if length == "length" => DomProp::FilesLength,
        ("valueAsNumber", None) => DomProp::ValueAsNumber,
        ("valueAsDate", None) => DomProp::ValueAsDate,
        ("value", Some(length)) if length == "length" => DomProp::ValueLength,
        ("validationMessage", None) => DomProp::ValidationMessage,
        ("validity", None) => DomProp::Validity,
        ("validity", Some(flag)) if flag == "valueMissing" => DomProp::ValidityValueMissing,
        ("validity", Some(flag)) if flag == "typeMismatch" => DomProp::ValidityTypeMismatch,
        ("validity", Some(flag)) if flag == "patternMismatch" => DomProp::ValidityPatternMismatch,
        ("validity", Some(flag)) if flag == "tooLong" => DomProp::ValidityTooLong,
        ("validity", Some(flag)) if flag == "tooShort" => DomProp::ValidityTooShort,
        ("validity", Some(flag)) if flag == "rangeUnderflow" => DomProp::ValidityRangeUnderflow,
        ("validity", Some(flag)) if flag == "rangeOverflow" => DomProp::ValidityRangeOverflow,
        ("validity", Some(flag)) if flag == "stepMismatch" => DomProp::ValidityStepMismatch,
        ("validity", Some(flag)) if flag == "badInput" => DomProp::ValidityBadInput,
        ("validity", Some(flag)) if flag == "valid" => DomProp::ValidityValid,
        ("validity", Some(flag)) if flag == "customError" => DomProp::ValidityCustomError,
        ("selectionStart", None) => DomProp::SelectionStart,
        ("selectionEnd", None) => DomProp::SelectionEnd,
        ("selectionDirection", None) => DomProp::SelectionDirection,
        ("checked", None) => DomProp::Checked,
        ("indeterminate", None) => DomProp::Indeterminate,
        ("open", None) => DomProp::Open,
        ("returnValue", None) => DomProp::ReturnValue,
        ("closedBy", None) | ("closedby", None) => DomProp::ClosedBy,
        ("readonly", None) | ("readOnly", None) => DomProp::Readonly,
        ("required", None) => DomProp::Required,
        ("disabled", None) => DomProp::Disabled,
        ("textContent", None) => DomProp::TextContent,
        ("innerText", None) if !matches!(target, DomQuery::DocumentRoot) => DomProp::InnerText,
        ("innerHTML", None) => DomProp::InnerHtml,
        ("outerHTML", None) => DomProp::OuterHtml,
        ("className", None) => DomProp::ClassName,
        ("classList", None) => DomProp::ClassList,
        ("classList", Some(length)) if length == "length" => DomProp::ClassListLength,
        ("part", None) => DomProp::Part,
        ("part", Some(length)) if length == "length" => DomProp::PartLength,
        ("id", None) => DomProp::Id,
        ("tagName", None) => DomProp::TagName,
        ("localName", None) => DomProp::LocalName,
        ("namespaceURI", None) => DomProp::NamespaceUri,
        ("prefix", None) => DomProp::Prefix,
        ("nextElementSibling", None) => DomProp::NextElementSibling,
        ("previousElementSibling", None) => DomProp::PreviousElementSibling,
        ("slot", None) => DomProp::Slot,
        ("role", None) => DomProp::Role,
        ("elementTiming", None) => DomProp::ElementTiming,
        ("name", None) => DomProp::Name,
        ("lang", None) => DomProp::Lang,
        ("dir", None) => DomProp::Dir,
        ("cite", None) if !matches!(target, DomQuery::DocumentRoot) => DomProp::Cite,
        ("dateTime", None) | ("datetime", None) if !matches!(target, DomQuery::DocumentRoot) => {
            DomProp::DateTime
        }
        ("clear", None)
            if !matches!(target, DomQuery::DocumentRoot) && !is_non_dom_var_target(&target) =>
        {
            DomProp::BrClear
        }
        ("align", None)
            if !matches!(target, DomQuery::DocumentRoot) && !is_non_dom_var_target(&target) =>
        {
            DomProp::CaptionAlign
        }
        ("span", None)
            if !matches!(target, DomQuery::DocumentRoot) && !is_non_dom_var_target(&target) =>
        {
            DomProp::ColSpan
        }
        ("width", None)
            if !matches!(target, DomQuery::DocumentRoot) && !is_non_dom_var_target(&target) =>
        {
            DomProp::CanvasWidth
        }
        ("height", None)
            if !matches!(target, DomQuery::DocumentRoot) && !is_non_dom_var_target(&target) =>
        {
            DomProp::CanvasHeight
        }
        ("aLink", None) | ("alink", None) if matches!(target, DomQuery::DocumentBody) => {
            DomProp::BodyDeprecatedAttr("alink".to_string())
        }
        ("background", None) if matches!(target, DomQuery::DocumentBody) => {
            DomProp::BodyDeprecatedAttr("background".to_string())
        }
        ("bgColor", None) | ("bgcolor", None) if matches!(target, DomQuery::DocumentBody) => {
            DomProp::BodyDeprecatedAttr("bgcolor".to_string())
        }
        ("bottomMargin", None) | ("bottommargin", None)
            if matches!(target, DomQuery::DocumentBody) =>
        {
            DomProp::BodyDeprecatedAttr("bottommargin".to_string())
        }
        ("leftMargin", None) | ("leftmargin", None) if matches!(target, DomQuery::DocumentBody) => {
            DomProp::BodyDeprecatedAttr("leftmargin".to_string())
        }
        ("link", None) if matches!(target, DomQuery::DocumentBody) => {
            DomProp::BodyDeprecatedAttr("link".to_string())
        }
        ("rightMargin", None) | ("rightmargin", None)
            if matches!(target, DomQuery::DocumentBody) =>
        {
            DomProp::BodyDeprecatedAttr("rightmargin".to_string())
        }
        ("text", None) if matches!(target, DomQuery::DocumentBody) => {
            DomProp::BodyDeprecatedAttr("text".to_string())
        }
        ("topMargin", None) | ("topmargin", None) if matches!(target, DomQuery::DocumentBody) => {
            DomProp::BodyDeprecatedAttr("topmargin".to_string())
        }
        ("vLink", None) | ("vlink", None) if matches!(target, DomQuery::DocumentBody) => {
            DomProp::BodyDeprecatedAttr("vlink".to_string())
        }
        ("clientWidth", None) => DomProp::ClientWidth,
        ("clientHeight", None) => DomProp::ClientHeight,
        ("clientLeft", None) => DomProp::ClientLeft,
        ("clientTop", None) => DomProp::ClientTop,
        ("currentCSSZoom", None) => DomProp::CurrentCssZoom,
        ("offsetWidth", None) => DomProp::OffsetWidth,
        ("offsetHeight", None) => DomProp::OffsetHeight,
        ("offsetLeft", None) => DomProp::OffsetLeft,
        ("offsetTop", None) => DomProp::OffsetTop,
        ("scrollWidth", None) => DomProp::ScrollWidth,
        ("scrollHeight", None) => DomProp::ScrollHeight,
        ("scrollLeft", None) => DomProp::ScrollLeft,
        ("scrollTop", None) => DomProp::ScrollTop,
        ("scrollLeftMax", None) => DomProp::ScrollLeftMax,
        ("scrollTopMax", None) => DomProp::ScrollTopMax,
        ("shadowRoot", None) => DomProp::ShadowRoot,
        ("activeElement", None) if matches!(target, DomQuery::DocumentRoot) => {
            DomProp::ActiveElement
        }
        ("characterSet", None) | ("charset", None) | ("inputEncoding", None)
            if matches!(target, DomQuery::DocumentRoot) =>
        {
            DomProp::CharacterSet
        }
        ("compatMode", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::CompatMode,
        ("contentType", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::ContentType,
        ("readyState", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::ReadyState,
        ("referrer", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Referrer,
        ("title", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Title,
        ("baseURI", None) => DomProp::BaseUri,
        ("URL", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Url,
        ("documentURI", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::DocumentUri,
        ("location", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Location,
        ("location", Some(href)) if matches!(target, DomQuery::DocumentRoot) && href == "href" => {
            DomProp::LocationHref
        }
        ("location", Some(protocol))
            if matches!(target, DomQuery::DocumentRoot) && protocol == "protocol" =>
        {
            DomProp::LocationProtocol
        }
        ("location", Some(host)) if matches!(target, DomQuery::DocumentRoot) && host == "host" => {
            DomProp::LocationHost
        }
        ("location", Some(hostname))
            if matches!(target, DomQuery::DocumentRoot) && hostname == "hostname" =>
        {
            DomProp::LocationHostname
        }
        ("location", Some(port)) if matches!(target, DomQuery::DocumentRoot) && port == "port" => {
            DomProp::LocationPort
        }
        ("location", Some(pathname))
            if matches!(target, DomQuery::DocumentRoot) && pathname == "pathname" =>
        {
            DomProp::LocationPathname
        }
        ("location", Some(search))
            if matches!(target, DomQuery::DocumentRoot) && search == "search" =>
        {
            DomProp::LocationSearch
        }
        ("location", Some(hash)) if matches!(target, DomQuery::DocumentRoot) && hash == "hash" => {
            DomProp::LocationHash
        }
        ("location", Some(origin))
            if matches!(target, DomQuery::DocumentRoot) && origin == "origin" =>
        {
            DomProp::LocationOrigin
        }
        ("location", Some(ancestor_origins))
            if matches!(target, DomQuery::DocumentRoot)
                && ancestor_origins == "ancestorOrigins" =>
        {
            DomProp::LocationAncestorOrigins
        }
        ("history", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::History,
        ("history", Some(length))
            if matches!(target, DomQuery::DocumentRoot) && length == "length" =>
        {
            DomProp::HistoryLength
        }
        ("history", Some(state))
            if matches!(target, DomQuery::DocumentRoot) && state == "state" =>
        {
            DomProp::HistoryState
        }
        ("history", Some(scroll_restoration))
            if matches!(target, DomQuery::DocumentRoot)
                && scroll_restoration == "scrollRestoration" =>
        {
            DomProp::HistoryScrollRestoration
        }
        ("defaultView", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::DefaultView,
        ("hidden", None) => DomProp::Hidden,
        ("visibilityState", None) if matches!(target, DomQuery::DocumentRoot) => {
            DomProp::VisibilityState
        }
        ("forms", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Forms,
        ("images", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Images,
        ("links", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Links,
        ("scripts", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Scripts,
        ("children", None) => DomProp::Children,
        ("childElementCount", None) => DomProp::ChildElementCount,
        ("firstElementChild", None) => DomProp::FirstElementChild,
        ("lastElementChild", None) => DomProp::LastElementChild,
        ("currentScript", None) if matches!(target, DomQuery::DocumentRoot) => {
            DomProp::CurrentScript
        }
        ("forms", Some(length))
            if matches!(target, DomQuery::DocumentRoot) && length == "length" =>
        {
            DomProp::FormsLength
        }
        ("images", Some(length))
            if matches!(target, DomQuery::DocumentRoot) && length == "length" =>
        {
            DomProp::ImagesLength
        }
        ("links", Some(length))
            if matches!(target, DomQuery::DocumentRoot) && length == "length" =>
        {
            DomProp::LinksLength
        }
        ("scripts", Some(length))
            if matches!(target, DomQuery::DocumentRoot) && length == "length" =>
        {
            DomProp::ScriptsLength
        }
        ("children", Some(length)) if length == "length" => DomProp::ChildrenLength,
        ("src", None) if is_media_target => DomProp::AudioSrc,
        ("autoplay", None) if is_media_target => DomProp::AudioAutoplay,
        ("controls", None) if is_media_target => DomProp::AudioControls,
        ("controlsList", None) | ("controlslist", None) if is_media_target => {
            DomProp::AudioControlsList
        }
        ("crossOrigin", None) | ("crossorigin", None) if is_media_target => {
            DomProp::AudioCrossOrigin
        }
        ("disableRemotePlayback", None) | ("disableremoteplayback", None) if is_media_target => {
            DomProp::AudioDisableRemotePlayback
        }
        ("loop", None) if is_media_target => DomProp::AudioLoop,
        ("muted", None) if is_media_target => DomProp::AudioMuted,
        ("preload", None) if is_media_target => DomProp::AudioPreload,
        ("attributionSrc", None) | ("attributionsrc", None) if is_anchor_target => {
            DomProp::AnchorAttributionSrc
        }
        ("download", None) if is_anchor_target => DomProp::AnchorDownload,
        ("hash", None) if is_anchor_target => DomProp::AnchorHash,
        ("host", None) if is_anchor_target => DomProp::AnchorHost,
        ("hostname", None) if is_anchor_target => DomProp::AnchorHostname,
        ("href", None) if is_anchor_target => DomProp::AnchorHref,
        ("hreflang", None) if is_anchor_target => DomProp::AnchorHreflang,
        ("interestForElement", None) if is_anchor_target => DomProp::AnchorInterestForElement,
        ("origin", None) if is_anchor_target => DomProp::AnchorOrigin,
        ("password", None) if is_anchor_target => DomProp::AnchorPassword,
        ("pathname", None) if is_anchor_target => DomProp::AnchorPathname,
        ("ping", None) if is_anchor_target => DomProp::AnchorPing,
        ("port", None) if is_anchor_target => DomProp::AnchorPort,
        ("protocol", None) if is_anchor_target => DomProp::AnchorProtocol,
        ("referrerPolicy", None) if is_anchor_target => DomProp::AnchorReferrerPolicy,
        ("rel", None) if is_anchor_target => DomProp::AnchorRel,
        ("relList", None) if is_anchor_target => DomProp::AnchorRelList,
        ("relList", Some(length)) if is_anchor_target && length == "length" => {
            DomProp::AnchorRelListLength
        }
        ("search", None) if is_anchor_target => DomProp::AnchorSearch,
        ("target", None) if is_anchor_target => DomProp::AnchorTarget,
        ("text", None) if is_anchor_target => DomProp::AnchorText,
        ("type", None) if is_anchor_target => DomProp::AnchorType,
        ("username", None) if is_anchor_target => DomProp::AnchorUsername,
        ("charset", None) if is_anchor_target => DomProp::AnchorCharset,
        ("coords", None) if is_anchor_target => DomProp::AnchorCoords,
        ("rev", None) if is_anchor_target => DomProp::AnchorRev,
        ("shape", None) if is_anchor_target => DomProp::AnchorShape,
        ("dataset", Some(key)) => DomProp::Dataset(key.clone()),
        ("style", Some(name)) => DomProp::Style(name.clone()),
        (prop_name, None) if is_aria_element_ref_single_property(prop_name) => {
            DomProp::AriaElementRefSingle(prop_name.to_string())
        }
        (prop_name, None) if is_aria_element_ref_list_property(prop_name) => {
            DomProp::AriaElementRefList(prop_name.to_string())
        }
        (prop_name, None) if is_aria_string_property(prop_name) => {
            DomProp::AriaString(prop_name.to_string())
        }
        (event_name, None)
            if event_name.starts_with("on")
                && !matches!(target, DomQuery::DocumentRoot)
                && !is_non_dom_var_target(&target) =>
        {
            DomProp::NodeEventHandler(event_name.to_ascii_lowercase())
        }
        _ => {
            if matches!(target, DomQuery::DocumentRoot) && starts_with_window_member_access(src) {
                return Ok(None);
            }
            if is_non_dom_var_target(&target) {
                return Ok(None);
            }
            let prop_label = if let Some(nested) = nested {
                format!("{head}.{nested}")
            } else {
                head
            };
            return Err(Error::ScriptParse(format!(
                "unsupported DOM property '{}' in: {src}",
                prop_label
            )));
        }
    };

    Ok(Some((target, prop)))
}

pub(crate) fn parse_get_attribute_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("getAttribute") {
        return Ok(None);
    }
    if cursor.peek().is_some_and(is_ident_char) {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, name)))
}

pub(crate) fn is_aria_string_property(prop: &str) -> bool {
    if !prop.starts_with("aria") || prop.len() <= 4 {
        return false;
    }
    !is_aria_element_ref_single_property(prop) && !is_aria_element_ref_list_property(prop)
}

pub(crate) fn is_aria_element_ref_single_property(prop: &str) -> bool {
    matches!(prop, "ariaActiveDescendantElement")
}

pub(crate) fn is_aria_element_ref_list_property(prop: &str) -> bool {
    matches!(
        prop,
        "ariaControlsElements"
            | "ariaDescribedByElements"
            | "ariaDetailsElements"
            | "ariaErrorMessageElements"
            | "ariaFlowToElements"
            | "ariaLabelledByElements"
            | "ariaOwnsElements"
    )
}

pub(crate) fn parse_has_attribute_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("hasAttribute") {
        return Ok(None);
    }
    if cursor.peek().is_some_and(is_ident_char) {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, name)))
}

pub(crate) fn parse_dom_matches_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("matches") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'(') {
        return Ok(None);
    }
    cursor.skip_ws();
    let selector = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, selector)))
}

pub(crate) fn parse_dom_closest_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("closest") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'(') {
        return Ok(None);
    }
    cursor.skip_ws();
    let selector = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, selector)))
}

pub(crate) fn parse_dom_computed_style_property_expr(
    src: &str,
) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("getComputedStyle") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let target = parse_element_target(&mut cursor)?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    if !cursor.consume_ascii("getPropertyValue") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let property = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((target, property)))
}
