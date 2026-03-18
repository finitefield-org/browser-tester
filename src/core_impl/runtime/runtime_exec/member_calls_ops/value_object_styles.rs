use super::*;

impl Harness {
    pub(crate) fn new_css_style_sheet_replace_sync_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("css_style_sheet_replace_sync".to_string()),
        )])
    }

    pub(crate) fn new_css_style_sheet_insert_rule_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("css_style_sheet_insert_rule".to_string()),
        )])
    }

    pub(crate) fn new_computed_style_get_property_value_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("computed_style_get_property_value".to_string()),
        )])
    }

    pub(crate) fn new_computed_style_item_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("computed_style_item".to_string()),
        )])
    }

    pub(crate) fn new_dom_rect_list_item_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("dom_rect_list_item".to_string()),
        )])
    }

    pub(crate) fn new_css_style_sheet_instance_value(owner_document: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CSS_STYLE_SHEET_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_CSS_STYLE_SHEET_OWNER_DOCUMENT_KEY.to_string(),
                owner_document,
            ),
            (
                INTERNAL_CSS_STYLE_SHEET_RULES_KEY.to_string(),
                Self::new_array_value(Vec::new()),
            ),
            (
                "replaceSync".to_string(),
                Self::new_css_style_sheet_replace_sync_callable(),
            ),
            (
                "insertRule".to_string(),
                Self::new_css_style_sheet_insert_rule_callable(),
            ),
        ])
    }

    pub(crate) fn is_css_style_sheet_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_CSS_STYLE_SHEET_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn css_style_sheet_owner_document(
        entries: &[(String, Value)],
    ) -> Option<Rc<RefCell<ObjectValue>>> {
        match Self::object_get_entry(entries, INTERNAL_CSS_STYLE_SHEET_OWNER_DOCUMENT_KEY) {
            Some(Value::Object(document)) => Some(document),
            _ => None,
        }
    }

    pub(crate) fn is_css_style_sheet_for_document(
        &self,
        value: &Value,
        document_object: &Rc<RefCell<ObjectValue>>,
    ) -> bool {
        let Value::Object(entries) = value else {
            return false;
        };
        let entries = entries.borrow();
        if !Self::is_css_style_sheet_object(&entries) {
            return false;
        }
        let Some(owner_document) = Self::css_style_sheet_owner_document(&entries) else {
            return false;
        };
        Rc::ptr_eq(&owner_document, document_object)
    }

    pub(crate) fn new_adopted_style_sheets_array_value(owner_document: Value) -> Value {
        let array = Self::new_array_value(Vec::new());
        if let Value::Array(values) = &array {
            let mut values_ref = values.borrow_mut();
            Self::object_set_entry(
                &mut values_ref.properties,
                INTERNAL_ADOPTED_STYLE_SHEETS_ARRAY_KEY.to_string(),
                Value::Bool(true),
            );
            Self::object_set_entry(
                &mut values_ref.properties,
                INTERNAL_ADOPTED_STYLE_SHEETS_OWNER_DOCUMENT_KEY.to_string(),
                owner_document,
            );
        }
        array
    }

    pub(crate) fn mark_as_adopted_style_sheets_array(
        &self,
        values: &Rc<RefCell<ArrayValue>>,
        owner_document: Value,
    ) {
        let mut values_ref = values.borrow_mut();
        Self::object_set_entry(
            &mut values_ref.properties,
            INTERNAL_ADOPTED_STYLE_SHEETS_ARRAY_KEY.to_string(),
            Value::Bool(true),
        );
        Self::object_set_entry(
            &mut values_ref.properties,
            INTERNAL_ADOPTED_STYLE_SHEETS_OWNER_DOCUMENT_KEY.to_string(),
            owner_document,
        );
    }

    pub(crate) fn adopted_style_sheets_owner_document(
        values: &ArrayValue,
    ) -> Option<Rc<RefCell<ObjectValue>>> {
        let is_adopted_array = matches!(
            Self::object_get_entry(&values.properties, INTERNAL_ADOPTED_STYLE_SHEETS_ARRAY_KEY),
            Some(Value::Bool(true))
        );
        if !is_adopted_array {
            return None;
        }
        match Self::object_get_entry(
            &values.properties,
            INTERNAL_ADOPTED_STYLE_SHEETS_OWNER_DOCUMENT_KEY,
        ) {
            Some(Value::Object(document)) => Some(document),
            _ => None,
        }
    }

    pub(crate) fn adopted_style_sheets_not_allowed_error() -> Error {
        Error::ScriptRuntime(
            "NotAllowedError: adoptedStyleSheets items must be CSSStyleSheet instances created in the same document".into(),
        )
    }

    pub(crate) fn ensure_document_adopted_style_sheets_property(&mut self) -> Value {
        if let Some(existing) = Self::object_get_entry(
            &self.dom_runtime.document_object.borrow(),
            "adoptedStyleSheets",
        ) {
            return existing;
        }
        let value = Self::new_adopted_style_sheets_array_value(Value::Object(
            self.dom_runtime.document_object.clone(),
        ));
        Self::object_set_entry(
            &mut self.dom_runtime.document_object.borrow_mut(),
            "adoptedStyleSheets".to_string(),
            value.clone(),
        );
        value
    }

    pub(crate) fn set_document_adopted_style_sheets_property(
        &mut self,
        value: Value,
    ) -> Result<()> {
        let Value::Array(values) = value else {
            return Err(Self::adopted_style_sheets_not_allowed_error());
        };
        let owner_document = self.dom_runtime.document_object.clone();
        for item in values.borrow().iter() {
            if !self.is_css_style_sheet_for_document(item, &owner_document) {
                return Err(Self::adopted_style_sheets_not_allowed_error());
            }
        }
        self.mark_as_adopted_style_sheets_array(
            &values,
            Value::Object(self.dom_runtime.document_object.clone()),
        );
        Self::object_set_entry(
            &mut self.dom_runtime.document_object.borrow_mut(),
            "adoptedStyleSheets".to_string(),
            Value::Array(values),
        );
        Ok(())
    }

    pub(crate) fn new_computed_style_object_value(node: NodeId, pseudo: Option<String>) -> Value {
        let value = Self::new_object_value(vec![
            (
                INTERNAL_COMPUTED_STYLE_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_COMPUTED_STYLE_TARGET_NODE_KEY.to_string(),
                Value::Node(node),
            ),
            (
                INTERNAL_COMPUTED_STYLE_PSEUDO_KEY.to_string(),
                pseudo.map(Value::String).unwrap_or(Value::Null),
            ),
            (
                "getPropertyValue".to_string(),
                Self::new_computed_style_get_property_value_callable(),
            ),
            ("item".to_string(), Self::new_computed_style_item_callable()),
        ]);
        let Value::Object(entries) = &value else {
            return value;
        };
        let mut entries = entries.borrow_mut();
        Self::mark_object_properties_non_enumerable(&mut *entries, &["getPropertyValue", "item"]);
        drop(entries);
        value
    }

    pub(crate) fn new_dom_rect_value(
        left: i64,
        top: i64,
        right: i64,
        bottom: i64,
        width: i64,
        height: i64,
    ) -> Value {
        let value = Self::new_object_value(vec![
            (INTERNAL_DOM_RECT_OBJECT_KEY.to_string(), Value::Bool(true)),
            ("x".to_string(), Value::Number(left)),
            ("y".to_string(), Value::Number(top)),
            ("left".to_string(), Value::Number(left)),
            ("top".to_string(), Value::Number(top)),
            ("right".to_string(), Value::Number(right)),
            ("bottom".to_string(), Value::Number(bottom)),
            ("width".to_string(), Value::Number(width)),
            ("height".to_string(), Value::Number(height)),
        ]);
        let Value::Object(entries) = &value else {
            return value;
        };
        let mut entries = entries.borrow_mut();
        for key in [
            "x", "y", "left", "top", "right", "bottom", "width", "height",
        ] {
            Self::object_set_entry(
                &mut *entries,
                Self::object_non_enumerable_storage_key(key),
                Value::Bool(true),
            );
            Self::object_set_entry(
                &mut *entries,
                Self::object_non_writable_storage_key(key),
                Value::Bool(true),
            );
            Self::object_set_entry(
                &mut *entries,
                Self::object_non_configurable_storage_key(key),
                Value::Bool(true),
            );
        }
        drop(entries);
        value
    }

    pub(crate) fn new_dom_rect_list_value(values: Vec<Value>) -> Value {
        let value = Self::new_array_value(values);
        let Value::Array(values) = &value else {
            return value;
        };
        let mut values = values.borrow_mut();
        Self::object_set_entry(
            &mut values.properties,
            INTERNAL_DOM_RECT_LIST_OBJECT_KEY.to_string(),
            Value::Bool(true),
        );
        Self::object_set_entry(
            &mut values.properties,
            "item".to_string(),
            Self::new_dom_rect_list_item_callable(),
        );
        Self::object_set_entry(
            &mut values.properties,
            Self::object_non_enumerable_storage_key("item"),
            Value::Bool(true),
        );
        drop(values);
        value
    }

    pub(crate) fn is_computed_style_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_COMPUTED_STYLE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_dom_rect_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_DOM_RECT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn computed_style_target_node(entries: &[(String, Value)]) -> Option<NodeId> {
        match Self::object_get_entry(entries, INTERNAL_COMPUTED_STYLE_TARGET_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }
    }

    pub(crate) fn computed_style_pseudo(entries: &[(String, Value)]) -> Option<String> {
        match Self::object_get_entry(entries, INTERNAL_COMPUTED_STYLE_PSEUDO_KEY) {
            Some(Value::String(pseudo)) => Some(pseudo),
            _ => None,
        }
    }

    fn computed_style_rule_value_from_style_nodes(
        &self,
        node: NodeId,
        pseudo: Option<&str>,
        property_name: &str,
    ) -> Option<String> {
        let mut resolved = None;
        for index in 0..self.dom.nodes.len() {
            let node_id = NodeId(index);
            let is_style_tag = self
                .dom
                .tag_name(node_id)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("style"));
            if !is_style_tag {
                continue;
            }
            let css_source = self.dom.text_content(node_id);
            for (selector_text, declarations_text) in Self::parse_css_rule_blocks(&css_source) {
                for selector in selector_text.split(',').map(str::trim) {
                    if selector.is_empty() {
                        continue;
                    }
                    let (base_selector, selector_pseudo) =
                        Self::split_selector_and_pseudo(selector);

                    let pseudo_matches = match (pseudo, selector_pseudo.as_deref()) {
                        (None, None) => true,
                        (Some(expected), Some(actual)) => actual.eq_ignore_ascii_case(expected),
                        _ => false,
                    };
                    if !pseudo_matches {
                        continue;
                    }

                    let selector_matches = if base_selector.is_empty() || base_selector == "*" {
                        true
                    } else {
                        matches!(
                            self.eval_matches_selector_value(node, base_selector),
                            Ok(Value::Bool(true))
                        )
                    };
                    if !selector_matches {
                        continue;
                    }

                    for (name, value) in parse_style_declarations(Some(declarations_text)) {
                        if name == property_name {
                            resolved = Some(value);
                        }
                    }
                }
            }
        }
        resolved
    }

    fn split_selector_and_pseudo(selector: &str) -> (&str, Option<String>) {
        let normalized = selector.trim();
        let Some(pseudo_pos) = normalized.find("::") else {
            return (normalized, None);
        };
        let base = normalized[..pseudo_pos].trim_end();
        let pseudo = normalized[pseudo_pos..].trim();
        (base, Some(pseudo.to_string()))
    }

    fn parse_css_rule_blocks(css_source: &str) -> Vec<(&str, &str)> {
        let bytes = css_source.as_bytes();
        let mut blocks = Vec::new();
        let mut cursor = 0usize;
        let mut selector_start = 0usize;
        while cursor < bytes.len() {
            if bytes[cursor] != b'{' {
                cursor += 1;
                continue;
            }
            let selector_end = cursor;
            cursor += 1;
            let declarations_start = cursor;
            let mut depth = 1usize;
            while cursor < bytes.len() && depth > 0 {
                match bytes[cursor] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                cursor += 1;
            }
            if depth != 0 || cursor == 0 {
                break;
            }
            let declarations_end = cursor.saturating_sub(1);
            let selector = css_source[selector_start..selector_end].trim();
            let declarations = css_source[declarations_start..declarations_end].trim();
            if !selector.is_empty() && !declarations.is_empty() {
                blocks.push((selector, declarations));
            }
            selector_start = cursor;
        }
        blocks
    }

    pub(crate) fn computed_style_property_value(
        &self,
        node: NodeId,
        pseudo: Option<&str>,
        property_name: &str,
    ) -> Result<String> {
        if self.dom.element(node).is_none() {
            return Err(Error::ScriptRuntime(
                "TypeError: getComputedStyle target must be an Element".into(),
            ));
        }
        let css_property = js_prop_to_css_name(property_name.trim());

        if pseudo.is_none() {
            let inline = self.dom.style_get(node, &css_property)?;
            if !inline.is_empty() {
                return Ok(inline);
            }
        }

        if let Some(from_rules) =
            self.computed_style_rule_value_from_style_nodes(node, pseudo, &css_property)
        {
            return Ok(from_rules);
        }

        Ok(String::new())
    }

    pub(crate) fn computed_style_object_property_from_entries(
        &self,
        entries: &[(String, Value)],
        key: &str,
    ) -> Result<Option<Value>> {
        if !Self::is_computed_style_object(entries) {
            return Ok(None);
        }

        if self.is_to_string_tag_property_key(key) {
            return Ok(Some(Value::String("CSSStyleDeclaration".to_string())));
        }

        match key {
            "getPropertyValue" | "item" => Ok(Some(
                Self::object_get_entry(entries, key).unwrap_or(Value::Undefined),
            )),
            "setProperty" | "removeProperty" => Ok(Some(Self::new_builtin_placeholder_function())),
            "cssText" => Ok(Some(Value::String(String::new()))),
            "length" => Ok(Some(Value::Number(0))),
            "parentRule" => Ok(Some(Value::Null)),
            "constructor" => Ok(Some(Value::Undefined)),
            _ => {
                let reserved = matches!(
                    key,
                    "__proto__"
                        | "toString"
                        | "valueOf"
                        | "hasOwnProperty"
                        | "isPrototypeOf"
                        | "propertyIsEnumerable"
                );
                if reserved {
                    return Ok(None);
                }
                let Some(node) = Self::computed_style_target_node(entries) else {
                    return Ok(Some(Value::Undefined));
                };
                let pseudo = Self::computed_style_pseudo(entries);
                let value = self.computed_style_property_value(node, pseudo.as_deref(), key)?;
                Ok(Some(Value::String(value)))
            }
        }
    }
}
