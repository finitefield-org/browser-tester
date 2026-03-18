use super::*;

impl Harness {
    pub(crate) fn css_style_sheet_object_from_receiver(
        receiver: Option<&Value>,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "CSSStyleSheet method called on incompatible receiver".into(),
            ));
        };
        let is_css_style_sheet = {
            let entries_ref = entries.borrow();
            Self::is_css_style_sheet_object(&entries_ref)
        };
        if !is_css_style_sheet {
            return Err(Error::ScriptRuntime(
                "CSSStyleSheet method called on incompatible receiver".into(),
            ));
        }
        Ok(entries.clone())
    }

    pub(crate) fn computed_style_state_from_receiver(
        receiver: Option<&Value>,
    ) -> Result<(NodeId, Option<String>)> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "getPropertyValue called on incompatible receiver".into(),
            ));
        };
        let entries = entries.borrow();
        if !Self::is_computed_style_object(&entries) {
            return Err(Error::ScriptRuntime(
                "getPropertyValue called on incompatible receiver".into(),
            ));
        }
        let Some(node) = Self::computed_style_target_node(&entries) else {
            return Err(Error::ScriptRuntime(
                "getPropertyValue called on incompatible receiver".into(),
            ));
        };
        let pseudo = Self::computed_style_pseudo(&entries);
        Ok((node, pseudo))
    }

    pub(crate) fn get_computed_style_pseudo_from_value(
        value: Option<&Value>,
    ) -> Result<Option<String>> {
        let Some(value) = value else {
            return Ok(None);
        };
        match value {
            Value::Null | Value::Undefined => Ok(None),
            Value::String(raw) => {
                let pseudo = raw.trim();
                if !Self::is_valid_get_computed_style_pseudo_selector(pseudo) {
                    return Err(Error::ScriptRuntime(
                        "TypeError: pseudoElt must be a valid pseudo-element selector and not ::part() or ::slotted()".into(),
                    ));
                }
                Ok(Some(pseudo.to_string()))
            }
            _ => Err(Error::ScriptRuntime(
                "TypeError: pseudoElt must be a valid pseudo-element selector and not ::part() or ::slotted()".into(),
            )),
        }
    }

    pub(crate) fn is_valid_get_computed_style_pseudo_selector(pseudo: &str) -> bool {
        if pseudo.is_empty() {
            return false;
        }
        let lowered = pseudo.to_ascii_lowercase();
        if lowered.starts_with("::part(") || lowered.starts_with("::slotted(") {
            return false;
        }
        let Some(rest) = lowered.strip_prefix("::") else {
            return false;
        };
        if rest.is_empty() {
            return false;
        }
        let (name, maybe_args) = if let Some(paren_idx) = rest.find('(') {
            let Some(stripped) = rest.strip_suffix(')') else {
                return false;
            };
            if paren_idx + 1 > stripped.len() {
                return false;
            }
            (&stripped[..paren_idx], Some(&stripped[paren_idx + 1..]))
        } else {
            (rest, None)
        };
        if name.is_empty()
            || !name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        {
            return false;
        }
        if let Some(args) = maybe_args {
            if args.contains('(') || args.contains(')') {
                return false;
            }
        }
        true
    }

    pub(crate) fn window_post_message_target_window(
        &self,
        this_arg: Option<&Value>,
    ) -> Rc<RefCell<ObjectValue>> {
        let Some(Value::Object(target)) = this_arg else {
            return self.dom_runtime.window_object.clone();
        };
        if Self::is_window_object(&target.borrow()) {
            target.clone()
        } else {
            self.dom_runtime.window_object.clone()
        }
    }

    pub(crate) fn window_post_message_target_origin_from_args(
        &self,
        args: &[Value],
        fallback_origin: &str,
    ) -> String {
        let Some(second) = args.get(1) else {
            return fallback_origin.to_string();
        };

        if matches!(second, Value::Object(_) | Value::Null) {
            if let Value::Object(entries) = second {
                let entries = entries.borrow();
                return match Self::object_get_entry(&entries, "targetOrigin") {
                    Some(Value::Null | Value::Undefined) | None => fallback_origin.to_string(),
                    Some(value) => value.as_string(),
                };
            }
            return fallback_origin.to_string();
        }

        second.as_string()
    }

    pub(crate) fn window_post_message_target_origin_matches(
        target_origin: &str,
        recipient_origin: &str,
        sender_origin: &str,
    ) -> bool {
        if target_origin == "*" {
            return true;
        }
        if target_origin == "/" {
            return sender_origin == recipient_origin;
        }
        target_origin == recipient_origin
    }

    pub(crate) fn class_list_node_from_receiver(receiver: Option<&Value>) -> Result<NodeId> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "DOMTokenList method called on incompatible receiver".into(),
            ));
        };
        let entries = entries.borrow();
        if !Self::is_class_list_object(&entries) {
            return Err(Error::ScriptRuntime(
                "DOMTokenList method called on incompatible receiver".into(),
            ));
        }
        match Self::object_get_entry(&entries, INTERNAL_CLASS_LIST_NODE_KEY) {
            Some(Value::Node(node)) => Ok(node),
            _ => Err(Error::ScriptRuntime(
                "DOMTokenList method called on incompatible receiver".into(),
            )),
        }
    }

    pub(crate) fn named_node_map_receiver_object_and_owner(
        receiver: Option<&Value>,
    ) -> Result<(Rc<RefCell<ObjectValue>>, NodeId)> {
        let Some(Value::Object(object)) = receiver else {
            return Err(Error::ScriptRuntime(
                "NamedNodeMap method called on incompatible receiver".into(),
            ));
        };
        let owner = {
            let entries = object.borrow();
            if !Self::is_named_node_map_object(&entries) {
                return Err(Error::ScriptRuntime(
                    "NamedNodeMap method called on incompatible receiver".into(),
                ));
            }
            match Self::named_node_map_owner_node(&entries) {
                Some(node) => node,
                None => {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap method called on incompatible receiver".into(),
                    ));
                }
            }
        };
        Ok((object.clone(), owner))
    }

    pub(crate) fn time_ranges_receiver_object_and_state(
        receiver: Option<&Value>,
    ) -> Result<(Rc<RefCell<ObjectValue>>, NodeId, String)> {
        let Some(Value::Object(object)) = receiver else {
            return Err(Error::ScriptRuntime(
                "TimeRanges method called on incompatible receiver".into(),
            ));
        };
        let (owner, kind) = {
            let entries = object.borrow();
            if !Self::is_time_ranges_object(&entries) {
                return Err(Error::ScriptRuntime(
                    "TimeRanges method called on incompatible receiver".into(),
                ));
            }
            let Some(owner) = Self::time_ranges_owner_node(&entries) else {
                return Err(Error::ScriptRuntime(
                    "TimeRanges method called on incompatible receiver".into(),
                ));
            };
            let Some(kind) = Self::time_ranges_kind(&entries) else {
                return Err(Error::ScriptRuntime(
                    "TimeRanges method called on incompatible receiver".into(),
                ));
            };
            (owner, kind)
        };
        Ok((object.clone(), owner, kind))
    }

    pub(crate) fn image_bitmap_receiver_object(
        receiver: Option<&Value>,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let Some(Value::Object(object)) = receiver else {
            return Err(Error::ScriptRuntime(
                "ImageBitmap method called on incompatible receiver".into(),
            ));
        };
        {
            let entries = object.borrow();
            if !Self::is_image_bitmap_object(&entries) {
                return Err(Error::ScriptRuntime(
                    "ImageBitmap method called on incompatible receiver".into(),
                ));
            }
        }
        Ok(object.clone())
    }

    pub(crate) fn animation_receiver_object(
        receiver: Option<&Value>,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let Some(Value::Object(object)) = receiver else {
            return Err(Error::ScriptRuntime(
                "Animation method called on incompatible receiver".into(),
            ));
        };
        {
            let entries = object.borrow();
            if !matches!(
                Self::object_get_entry(&entries, INTERNAL_ANIMATION_OBJECT_KEY),
                Some(Value::Bool(true))
            ) {
                return Err(Error::ScriptRuntime(
                    "Animation method called on incompatible receiver".into(),
                ));
            }
        }
        Ok(object.clone())
    }

    pub(crate) fn text_track_receiver_object_and_node(
        receiver: Option<&Value>,
    ) -> Result<(Rc<RefCell<ObjectValue>>, NodeId)> {
        let Some(Value::Object(object)) = receiver else {
            return Err(Error::ScriptRuntime(
                "TextTrack method called on incompatible receiver".into(),
            ));
        };
        let owner = {
            let entries = object.borrow();
            if !Self::is_text_track_object(&entries) {
                return Err(Error::ScriptRuntime(
                    "TextTrack method called on incompatible receiver".into(),
                ));
            }
            let Some(owner) = Self::text_track_owner_node(&entries) else {
                return Err(Error::ScriptRuntime(
                    "TextTrack method called on incompatible receiver".into(),
                ));
            };
            owner
        };
        Ok((object.clone(), owner))
    }
}
