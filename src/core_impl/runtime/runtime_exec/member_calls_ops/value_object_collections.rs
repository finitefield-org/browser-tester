use super::*;
use std::collections::HashSet;

impl Harness {
    pub(crate) fn is_canvas_2d_context_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_CANVAS_2D_CONTEXT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_event_target_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_EVENT_TARGET_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_match_media_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_MATCH_MEDIA_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_hash_change_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_HASH_CHANGE_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_error_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_ERROR_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_before_unload_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_BEFORE_UNLOAD_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_keyboard_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_KEYBOARD_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_wheel_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_WHEEL_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_navigate_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_NAVIGATE_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_pointer_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_POINTER_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_attr_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_ATTR_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_named_node_map_object(entries: &(impl ObjectEntryLookup + ?Sized)) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_NAMED_NODE_MAP_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn named_node_map_owner_node(
        entries: &(impl ObjectEntryLookup + ?Sized),
    ) -> Option<NodeId> {
        match Self::object_get_entry(entries, INTERNAL_NAMED_NODE_MAP_OWNER_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }
    }

    pub(crate) fn named_node_map_entries(&self, owner: NodeId) -> Vec<(String, String)> {
        let Some(element) = self.dom.element(owner) else {
            return Vec::new();
        };
        let mut attrs = element
            .attrs
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<Vec<_>>();
        attrs.sort_by(|(left, _), (right, _)| left.cmp(right));
        attrs
    }

    fn is_named_node_map_builtin_property_name(key: &str) -> bool {
        matches!(
            key,
            "length"
                | "item"
                | "getNamedItem"
                | "setNamedItem"
                | "removeNamedItem"
                | "getNamedItemNS"
                | "setNamedItemNS"
                | "removeNamedItemNS"
                | "forEach"
                | "keys"
                | "values"
                | "entries"
        )
    }

    pub(crate) fn named_node_map_named_property_is_visible(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> bool {
        if Self::is_named_node_map_builtin_property_name(key) {
            return false;
        }

        let mut prototype = Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
            .or_else(|| Some(self.object_constructor_prototype_value()));
        while let Some(current) = prototype {
            match current {
                Value::Null | Value::Undefined => break,
                Value::Object(object) => {
                    let object_value = Value::Object(object.clone());
                    if self
                        .object_has_own_value(&object_value, key)
                        .is_ok_and(|value| value.truthy())
                    {
                        return false;
                    }
                    prototype = self.value_internal_prototype_value(&object_value);
                }
                _ => break,
            }
        }
        true
    }

    fn is_html_collection_builtin_property_name(key: &str) -> bool {
        matches!(
            key,
            "length" | "item" | "namedItem" | "forEach" | "keys" | "values" | "entries"
        )
    }

    pub(crate) fn node_list_is_html_collection(nodes: &Rc<RefCell<NodeListValue>>) -> bool {
        nodes.borrow().kind.is_html_collection_family()
    }

    pub(crate) fn node_list_is_radio_node_list(nodes: &Rc<RefCell<NodeListValue>>) -> bool {
        matches!(nodes.borrow().kind, NodeListKind::RadioNodeList)
    }

    pub(crate) fn node_list_display_name(nodes: &Rc<RefCell<NodeListValue>>) -> &'static str {
        nodes.borrow().kind.display_name()
    }

    pub(crate) fn node_list_item_value(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        node: NodeId,
    ) -> Value {
        if matches!(nodes.borrow().kind, NodeListKind::TextTrackList) {
            return self.text_track_object_value(node);
        }
        Value::Node(node)
    }

    pub(crate) fn html_collection_named_entries(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
    ) -> Vec<(String, NodeId)> {
        if !Self::node_list_is_html_collection(nodes) {
            return Vec::new();
        }

        let mut supported = Vec::new();
        let mut seen = HashSet::new();
        for node in self.node_list_snapshot(nodes) {
            let Some(_) = self.dom.element(node) else {
                continue;
            };
            for candidate in [self.dom.attr(node, "id"), self.dom.attr(node, "name")] {
                let Some(candidate) = candidate.filter(|candidate| !candidate.is_empty()) else {
                    continue;
                };
                if seen.insert(candidate.clone()) {
                    supported.push((candidate, node));
                }
            }
        }
        supported
    }

    pub(crate) fn html_collection_named_property_is_visible(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        key: &str,
    ) -> bool {
        if !Self::node_list_is_html_collection(nodes)
            || key.is_empty()
            || Self::own_property_integer_key(key).is_some()
            || Self::is_html_collection_builtin_property_name(key)
        {
            return false;
        }

        let collection = Value::NodeList(nodes.clone());
        let mut prototype = self.value_internal_prototype_value(&collection);
        while let Some(current) = prototype {
            match current {
                Value::Null | Value::Undefined => break,
                _ => {
                    if self
                        .object_has_own_value(&current, key)
                        .is_ok_and(|value| value.truthy())
                    {
                        return false;
                    }
                    prototype = self.value_internal_prototype_value(&current);
                }
            }
        }
        true
    }

    pub(crate) fn html_collection_named_property_value(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        key: &str,
    ) -> Option<Value> {
        if !self.html_collection_named_property_is_visible(nodes, key) {
            return None;
        }
        let owner_form = {
            let nodes_ref = nodes.borrow();
            match nodes_ref.live_source {
                Some(LiveNodeListSource::FormElements { form }) => Some(form),
                _ => None,
            }
        };
        if let Some(form) = owner_form {
            return self
                .form_controls_named_item_value(form, key)
                .ok()
                .flatten();
        }
        self.html_collection_named_entries(nodes)
            .into_iter()
            .find(|(name, _)| name == key)
            .map(|(_, node)| Value::Node(node))
    }

    pub(crate) fn form_controls_named_item_value(
        &mut self,
        form: NodeId,
        key: &str,
    ) -> Result<Option<Value>> {
        let matches = self.form_controls_named_matches(form, key)?;
        Ok(match matches.len() {
            0 => None,
            1 => Some(Value::Node(matches[0])),
            _ => Some(self.form_named_group_live_list_value(form, key)?),
        })
    }

    pub(crate) fn is_html_form_hidden_named_property_name(key: &str) -> bool {
        matches!(
            key,
            "elements"
                | "length"
                | "name"
                | "action"
                | "submit"
                | "requestSubmit"
                | "reset"
                | "checkValidity"
                | "reportValidity"
                | "method"
                | "enctype"
                | "encoding"
                | "target"
                | "noValidate"
                | "acceptCharset"
                | "rel"
        )
    }

    pub(crate) fn form_named_property_value(
        &mut self,
        form: NodeId,
        key: &str,
    ) -> Result<Option<Value>> {
        if key.is_empty() || Self::is_html_form_hidden_named_property_name(key) {
            return Ok(None);
        }
        self.form_controls_named_item_value(form, key)
    }

    pub(crate) fn html_form_builtin_own_string_keys() -> [&'static str; 7] {
        [
            "elements",
            "length",
            "submit",
            "requestSubmit",
            "reset",
            "checkValidity",
            "reportValidity",
        ]
    }

    fn node_cached_receiver_builtin_callable(
        &mut self,
        node: NodeId,
        cache_key: &str,
        family: &str,
        member: &str,
    ) -> Value {
        if let Some(value) = self
            .dom_runtime
            .node_expando_props
            .get(&(node, cache_key.to_string()))
            .cloned()
        {
            return value;
        }
        let value = Self::new_receiver_builtin_callable(family, member);
        self.dom_runtime
            .node_expando_props
            .insert((node, cache_key.to_string()), value.clone());
        value
    }

    pub(crate) fn form_builtin_property_value(&self, key: &str) -> Option<Value> {
        match key {
            "submit" | "requestSubmit" | "reset" | "checkValidity" | "reportValidity" => {
                Some(Self::new_receiver_builtin_callable("html_form", key))
            }
            _ => None,
        }
    }

    pub(crate) fn html_media_builtin_own_string_keys() -> [&'static str; 5] {
        ["play", "pause", "load", "canPlayType", "fastSeek"]
    }

    pub(crate) fn html_media_builtin_property_value(
        &mut self,
        media: NodeId,
        key: &str,
    ) -> Result<Option<Value>> {
        match key {
            "play" => Ok(Some(self.node_cached_receiver_builtin_callable(
                media,
                INTERNAL_MEDIA_PLAY_CALLABLE_KEY,
                "html_media",
                "play",
            ))),
            "pause" => Ok(Some(self.node_cached_receiver_builtin_callable(
                media,
                INTERNAL_MEDIA_PAUSE_CALLABLE_KEY,
                "html_media",
                "pause",
            ))),
            "load" => Ok(Some(self.node_cached_receiver_builtin_callable(
                media,
                INTERNAL_MEDIA_LOAD_CALLABLE_KEY,
                "html_media",
                "load",
            ))),
            "canPlayType" => Ok(Some(self.node_cached_receiver_builtin_callable(
                media,
                INTERNAL_MEDIA_CAN_PLAY_TYPE_CALLABLE_KEY,
                "html_media",
                "canPlayType",
            ))),
            "fastSeek" => Ok(Some(self.node_cached_receiver_builtin_callable(
                media,
                INTERNAL_MEDIA_FAST_SEEK_CALLABLE_KEY,
                "html_media",
                "fastSeek",
            ))),
            _ => Ok(None),
        }
    }

    pub(crate) fn html_form_builtin_property_value(
        &mut self,
        form: NodeId,
        key: &str,
    ) -> Result<Option<Value>> {
        match key {
            "elements" => self.form_elements_live_list_value(form).map(Some),
            "length" => Ok(Some(Value::Number(self.form_elements(form)?.len() as i64))),
            _ => Ok(self.form_builtin_property_value(key)),
        }
    }

    pub(crate) fn html_form_named_property_keys(&mut self, form: NodeId) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for control in self.form_elements(form)? {
            let id = self.dom.attr(control, "id").unwrap_or_default();
            if !id.is_empty()
                && !Self::is_html_form_hidden_named_property_name(&id)
                && seen.insert(id.clone())
            {
                out.push(id);
            }

            let name = self.dom.attr(control, "name").unwrap_or_default();
            if !name.is_empty()
                && !Self::is_html_form_hidden_named_property_name(&name)
                && seen.insert(name.clone())
            {
                out.push(name);
            }
        }
        Ok(out)
    }

    pub(crate) fn node_explicit_own_property_overrides_dom_property(
        &self,
        node: NodeId,
        key: &str,
    ) -> bool {
        if !self.node_has_explicit_own_property(node, key) {
            return false;
        }
        if self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("form"))
        {
            return true;
        }
        matches!(
            key,
            "id" | "className"
                | "lang"
                | "dir"
                | "accessKey"
                | "accesskey"
                | "autocapitalize"
                | "autocorrect"
                | "contentEditable"
                | "contenteditable"
                | "draggable"
                | "enterKeyHint"
                | "enterkeyhint"
                | "hidden"
                | "inert"
                | "inputMode"
                | "inputmode"
                | "nonce"
                | "popover"
                | "spellcheck"
                | "tabIndex"
                | "tabindex"
                | "title"
                | "translate"
                | "append"
                | "prepend"
                | "replaceChildren"
                | "before"
                | "after"
                | "replaceWith"
                | "remove"
                | "appendChild"
                | "insertBefore"
                | "removeChild"
                | "replaceChild"
                | "hasChildNodes"
                | "contains"
                | "getRootNode"
                | "compareDocumentPosition"
                | "isEqualNode"
                | "isSameNode"
                | "normalize"
                | "isDefaultNamespace"
                | "lookupPrefix"
                | "lookupNamespaceURI"
                | "cloneNode"
                | "querySelector"
                | "querySelectorAll"
                | "getAttributeNames"
                | "toggleAttribute"
                | "matches"
                | "closest"
                | "insertAdjacentElement"
                | "insertAdjacentHTML"
                | "insertAdjacentText"
                | "setHTMLUnsafe"
                | "controlsList"
                | "controlslist"
                | "crossOrigin"
                | "crossorigin"
                | "disableRemotePlayback"
                | "disableremoteplayback"
                | "disablePictureInPicture"
                | "disablepictureinpicture"
                | "playsInline"
                | "playsinline"
                | "clientWidth"
                | "clientHeight"
                | "clientLeft"
                | "clientTop"
                | "currentCSSZoom"
                | "offsetWidth"
                | "offsetHeight"
                | "offsetLeft"
                | "offsetTop"
                | "scrollWidth"
                | "scrollHeight"
                | "scrollLeft"
                | "scrollTop"
                | "scrollLeftMax"
                | "scrollTopMax"
                | "paused"
                | "ended"
                | "seeking"
                | "networkState"
                | "readyState"
                | "defaultMuted"
                | "currentTime"
                | "volume"
                | "duration"
                | "playbackRate"
                | "defaultPlaybackRate"
                | "play"
                | "pause"
                | "load"
                | "canPlayType"
                | "fastSeek"
                | "textTracks"
                | "buffered"
                | "seekable"
                | "played"
                | "value"
                | "open"
                | "closedBy"
                | "closedby"
                | "htmlFor"
                | "slot"
                | "role"
                | "elementTiming"
                | "elementtiming"
                | "name"
                | "cite"
                | "dateTime"
                | "datetime"
                | "clear"
                | "align"
                | "href"
                | "src"
                | "currentSrc"
                | "currentsrc"
                | "autoplay"
                | "controls"
                | "loop"
                | "muted"
                | "alt"
                | "download"
                | "hreflang"
                | "ping"
                | "referrerPolicy"
                | "referrerpolicy"
                | "rel"
                | "target"
                | "noHref"
                | "nohref"
                | "charset"
                | "coords"
                | "rev"
                | "shape"
                | "media"
                | "type"
                | "kind"
                | "label"
                | "srclang"
                | "srcLang"
                | "track"
                | "default"
                | "poster"
                | "preload"
                | "formAction"
                | "attributionSrc"
                | "attributionsrc"
                | "sizes"
                | "srcset"
                | "srcSet"
                | "data"
                | "srcdoc"
                | "srcDoc"
                | "useMap"
                | "usemap"
        )
    }

    pub(crate) fn node_explicit_own_dom_property_shadow_key<'a>(
        &self,
        node: NodeId,
        keys: &[&'a str],
    ) -> Option<&'a str> {
        keys.iter()
            .copied()
            .find(|key| self.node_explicit_own_property_overrides_dom_property(node, key))
    }

    pub(crate) fn node_explicit_own_dom_property_shadow_value(
        &mut self,
        node: NodeId,
        keys: &[&str],
    ) -> Result<Option<Value>> {
        let Some(_) = self.node_explicit_own_dom_property_shadow_key(node, keys) else {
            return Ok(None);
        };
        let entries = self.node_expando_entries(node);
        for key in keys {
            if let Some(value) =
                self.object_property_from_entries_with_getter(&Value::Node(node), &entries, key)?
            {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }
}
