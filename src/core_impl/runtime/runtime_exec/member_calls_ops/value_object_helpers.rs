use super::*;

impl Harness {
    pub(crate) fn resolved_dir_for_node(&self, node: NodeId) -> String {
        if let Some(explicit) = self.dom.attr(node, "dir") {
            return explicit;
        }
        if self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("bdi"))
        {
            return "auto".to_string();
        }
        String::new()
    }

    pub(crate) fn resolved_role_for_node(&self, node: NodeId) -> String {
        if let Some(explicit) = self.dom.attr(node, "role") {
            return explicit;
        }
        if let Some(tag) = self.dom.tag_name(node) {
            if tag.eq_ignore_ascii_case("address") {
                return "group".to_string();
            }
            if tag.eq_ignore_ascii_case("aside") {
                return "complementary".to_string();
            }
            if tag.eq_ignore_ascii_case("article") {
                return "article".to_string();
            }
            if tag.eq_ignore_ascii_case("blockquote") {
                return "blockquote".to_string();
            }
            if tag.eq_ignore_ascii_case("body") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("b") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("bdi") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("bdo") {
                return "generic".to_string();
            }
            if (tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area"))
                && self.dom.attr(node, "href").is_some()
            {
                return "link".to_string();
            }
        }
        String::new()
    }

    pub(crate) fn new_array_value(values: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(ArrayValue::new(values))))
    }

    pub(crate) fn set_array_property(array: &Rc<RefCell<ArrayValue>>, key: String, value: Value) {
        Self::object_set_entry(&mut array.borrow_mut().properties, key, value);
    }

    pub(crate) fn new_object_value(entries: Vec<(String, Value)>) -> Value {
        Value::Object(Rc::new(RefCell::new(ObjectValue::new(entries))))
    }

    pub(crate) fn mock_file_to_value(file: &MockFile) -> Value {
        Self::new_object_value(vec![
            ("name".to_string(), Value::String(file.name.clone())),
            (
                "lastModified".to_string(),
                Value::Number(file.last_modified),
            ),
            ("size".to_string(), Value::Number(file.size.max(0))),
            ("type".to_string(), Value::String(file.mime_type.clone())),
            (
                "webkitRelativePath".to_string(),
                Value::String(file.webkit_relative_path.clone()),
            ),
        ])
    }

    pub(crate) fn input_files_value(&self, node: NodeId) -> Result<Value> {
        let element = self
            .dom
            .element(node)
            .ok_or_else(|| Error::ScriptRuntime("files target is not an element".into()))?;
        if !is_file_input_element(element) {
            return Ok(Value::Null);
        }
        let files = self.dom.files(node)?;
        Ok(Self::new_array_value(
            files.iter().map(Self::mock_file_to_value).collect(),
        ))
    }

    pub(crate) fn new_boolean_constructor_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("boolean_constructor".to_string()),
        )])
    }

    pub(crate) fn new_string_wrapper_value(value: String) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
            Value::String(value),
        )])
    }

    pub(crate) fn object_set_entry(entries: &mut impl ObjectEntryMut, key: String, value: Value) {
        entries.set_entry(key, value);
    }

    pub(crate) fn object_get_entry(
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> Option<Value> {
        entries.get_entry(key)
    }

    pub(crate) fn callable_kind_from_value(value: &Value) -> Option<&str> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        match Self::object_get_entry(&entries, INTERNAL_CALLABLE_KIND_KEY) {
            Some(Value::String(kind)) => Some(match kind.as_str() {
                "intl_collator_compare" => "intl_collator_compare",
                "intl_date_time_format" => "intl_date_time_format",
                "intl_duration_format" => "intl_duration_format",
                "intl_list_format" => "intl_list_format",
                "intl_number_format" => "intl_number_format",
                "intl_segmenter_segments_iterator" => "intl_segmenter_segments_iterator",
                "intl_segmenter_iterator_next" => "intl_segmenter_iterator_next",
                "boolean_constructor" => "boolean_constructor",
                _ => return None,
            }),
            _ => None,
        }
    }

    pub(crate) fn data_attr_name_to_dataset_key(attr_name: &str) -> Option<String> {
        let raw = attr_name.strip_prefix("data-")?;
        if raw.is_empty() {
            return None;
        }
        let mut out = String::new();
        let mut uppercase_next = false;
        for ch in raw.chars() {
            if ch == '-' {
                uppercase_next = true;
                continue;
            }
            if uppercase_next {
                out.push(ch.to_ascii_uppercase());
                uppercase_next = false;
            } else {
                out.push(ch);
            }
        }
        if out.is_empty() { None } else { Some(out) }
    }

    pub(crate) fn dataset_entries_for_node(&self, node: NodeId) -> Vec<(String, Value)> {
        let Some(element) = self.dom.element(node) else {
            return Vec::new();
        };
        let mut entries = element
            .attrs
            .iter()
            .filter_map(|(attr_name, attr_value)| {
                Self::data_attr_name_to_dataset_key(attr_name)
                    .map(|key| (key, Value::String(attr_value.clone())))
            })
            .collect::<Vec<_>>();
        entries.sort_by(|(left, _), (right, _)| left.cmp(right));
        entries
    }

    pub(crate) fn object_property_from_value(&self, value: &Value, key: &str) -> Result<Value> {
        match value {
            Value::Node(node) => {
                let is_select = self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("select"))
                    .unwrap_or(false);
                let select_options = || {
                    let mut options = Vec::new();
                    self.dom.collect_select_options(*node, &mut options);
                    options
                };

                match key {
                    "textContent" | "innerText" => Ok(Value::String(self.dom.text_content(*node))),
                    "innerHTML" => Ok(Value::String(self.dom.inner_html(*node)?)),
                    "outerHTML" => Ok(Value::String(self.dom.outer_html(*node)?)),
                    "value" => Ok(Value::String(self.dom.value(*node)?)),
                    "files" => self.input_files_value(*node),
                    "valueAsNumber" => Ok(Self::number_value(self.input_value_as_number(*node)?)),
                    "valueAsDate" => Ok(self
                        .input_value_as_date_ms(*node)?
                        .map(Self::new_date_value)
                        .unwrap_or(Value::Null)),
                    "checked" => Ok(Value::Bool(self.dom.checked(*node)?)),
                    "disabled" => Ok(Value::Bool(self.dom.disabled(*node))),
                    "required" => Ok(Value::Bool(self.dom.required(*node))),
                    "readonly" | "readOnly" => Ok(Value::Bool(self.dom.readonly(*node))),
                    "id" => Ok(Value::String(
                        self.dom.attr(*node, "id").unwrap_or_default(),
                    )),
                    "name" => Ok(Value::String(
                        self.dom.attr(*node, "name").unwrap_or_default(),
                    )),
                    "dir" => Ok(Value::String(self.resolved_dir_for_node(*node))),
                    "cite" => Ok(Value::String(
                        self.dom.attr(*node, "cite").unwrap_or_default(),
                    )),
                    "clear" => Ok(Value::String(
                        self.dom.attr(*node, "clear").unwrap_or_default(),
                    )),
                    "aLink" | "alink" => Ok(Value::String(
                        self.dom.attr(*node, "alink").unwrap_or_default(),
                    )),
                    "background" => Ok(Value::String(
                        self.dom.attr(*node, "background").unwrap_or_default(),
                    )),
                    "bgColor" | "bgcolor" => Ok(Value::String(
                        self.dom.attr(*node, "bgcolor").unwrap_or_default(),
                    )),
                    "bottomMargin" | "bottommargin" => Ok(Value::String(
                        self.dom.attr(*node, "bottommargin").unwrap_or_default(),
                    )),
                    "leftMargin" | "leftmargin" => Ok(Value::String(
                        self.dom.attr(*node, "leftmargin").unwrap_or_default(),
                    )),
                    "link" => Ok(Value::String(
                        self.dom.attr(*node, "link").unwrap_or_default(),
                    )),
                    "rightMargin" | "rightmargin" => Ok(Value::String(
                        self.dom.attr(*node, "rightmargin").unwrap_or_default(),
                    )),
                    "text" => Ok(Value::String(
                        if self
                            .dom
                            .tag_name(*node)
                            .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
                        {
                            self.dom.attr(*node, "text").unwrap_or_default()
                        } else {
                            self.dom.text_content(*node)
                        },
                    )),
                    "topMargin" | "topmargin" => Ok(Value::String(
                        self.dom.attr(*node, "topmargin").unwrap_or_default(),
                    )),
                    "vLink" | "vlink" => Ok(Value::String(
                        self.dom.attr(*node, "vlink").unwrap_or_default(),
                    )),
                    "title" => Ok(Value::String(
                        self.dom.attr(*node, "title").unwrap_or_default(),
                    )),
                    "type" => Ok(Value::String(
                        self.dom.attr(*node, "type").unwrap_or_default(),
                    )),
                    "tagName" => Ok(Value::String(
                        self.dom
                            .tag_name(*node)
                            .unwrap_or_default()
                            .to_ascii_uppercase(),
                    )),
                    "className" => Ok(Value::String(
                        self.dom.attr(*node, "class").unwrap_or_default(),
                    )),
                    "role" => Ok(Value::String(self.resolved_role_for_node(*node))),
                    "baseURI" => Ok(Value::String(self.document_base_url())),
                    "dataset" => Ok(Self::new_object_value(self.dataset_entries_for_node(*node))),
                    "options" => {
                        if !is_select {
                            return Ok(Value::Undefined);
                        }
                        Ok(Value::NodeList(select_options()))
                    }
                    "selectedIndex" => {
                        if !is_select {
                            return Ok(Value::Undefined);
                        }
                        let options = select_options();
                        if options.is_empty() {
                            return Ok(Value::Number(-1));
                        }
                        let selected = options
                            .iter()
                            .position(|option| self.dom.attr(*option, "selected").is_some())
                            .unwrap_or(0);
                        Ok(Value::Number(selected as i64))
                    }
                    "length" => {
                        if !is_select {
                            return Ok(Value::Undefined);
                        }
                        Ok(Value::Number(select_options().len() as i64))
                    }
                    _ if key.starts_with("on") => Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(*node, key.to_string()))
                        .cloned()
                        .unwrap_or(Value::Null)),
                    _ => Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(*node, key.to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)),
                }
            }
            Value::String(text) => {
                if key == "length" {
                    Ok(Value::Number(text.chars().count() as i64))
                } else if key == "constructor" {
                    Ok(Value::StringConstructor)
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(text
                        .chars()
                        .nth(index)
                        .map(|ch| Value::String(ch.to_string()))
                        .unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Array(values) => {
                let values = values.borrow();
                if key == "length" {
                    Ok(Value::Number(values.len() as i64))
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(values.get(index).cloned().unwrap_or(Value::Undefined))
                } else if let Some(value) = Self::object_get_entry(&values.properties, key) {
                    Ok(value)
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::NodeList(nodes) => {
                if key == "length" {
                    Ok(Value::Number(nodes.len() as i64))
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(nodes
                        .get(index)
                        .copied()
                        .map(Value::Node)
                        .unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::TypedArray(values) => {
                let snapshot = self.typed_array_snapshot(values)?;
                if key == "length" {
                    Ok(Value::Number(snapshot.len() as i64))
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(snapshot.get(index).cloned().unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(text) = Self::string_wrapper_value_from_object(&entries) {
                    if key == "length" {
                        return Ok(Value::Number(text.chars().count() as i64));
                    }
                    if key == "constructor" {
                        return Ok(Value::StringConstructor);
                    }
                    if let Ok(index) = key.parse::<usize>() {
                        return Ok(text
                            .chars()
                            .nth(index)
                            .map(|ch| Value::String(ch.to_string()))
                            .unwrap_or(Value::Undefined));
                    }
                }
                if Self::is_url_search_params_object(&entries) {
                    if key == "size" {
                        let size =
                            Self::url_search_params_pairs_from_object_entries(&entries).len();
                        return Ok(Value::Number(size as i64));
                    }
                }
                if Self::is_storage_object(&entries) {
                    if key == "length" {
                        let len = Self::storage_pairs_from_object_entries(&entries).len();
                        return Ok(Value::Number(len as i64));
                    }
                    if let Some(value) = Self::object_get_entry(&entries, key) {
                        return Ok(value);
                    }
                    if Self::is_storage_method_name(key) {
                        return Ok(Self::new_builtin_placeholder_function());
                    }
                    if let Some((_, value)) = Self::storage_pairs_from_object_entries(&entries)
                        .into_iter()
                        .find(|(name, _)| name == key)
                    {
                        return Ok(Value::String(value));
                    }
                    return Ok(Value::Undefined);
                }
                let is_document_object = matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                );
                if is_document_object {
                    let value = match key {
                        "body" => self.dom.body().map(Value::Node).unwrap_or(Value::Null),
                        "head" => self.dom.head().map(Value::Node).unwrap_or(Value::Null),
                        "documentElement" => self
                            .dom
                            .document_element()
                            .map(Value::Node)
                            .unwrap_or(Value::Null),
                        _ => Value::Undefined,
                    };
                    if !matches!(value, Value::Undefined) {
                        return Ok(value);
                    }
                }
                if Self::is_url_object(&entries) && key == "constructor" {
                    return Ok(Value::UrlConstructor);
                }
                Ok(Self::object_get_entry(&entries, key).unwrap_or(Value::Undefined))
            }
            Value::Promise(promise) => {
                if key == "constructor" {
                    Ok(Value::PromiseConstructor)
                } else {
                    let promise = promise.borrow();
                    if key == "status" {
                        let status = match &promise.state {
                            PromiseState::Pending => "pending",
                            PromiseState::Fulfilled(_) => "fulfilled",
                            PromiseState::Rejected(_) => "rejected",
                        };
                        Ok(Value::String(status.to_string()))
                    } else {
                        Ok(Value::Undefined)
                    }
                }
            }
            Value::Map(map) => {
                let map = map.borrow();
                if key == "size" {
                    Ok(Value::Number(map.entries.len() as i64))
                } else if key == "constructor" {
                    Ok(Value::MapConstructor)
                } else if let Some(value) = Self::object_get_entry(&map.properties, key) {
                    Ok(value)
                } else if Self::is_map_method_name(key) {
                    Ok(Self::new_builtin_placeholder_function())
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Set(set) => {
                let set = set.borrow();
                if key == "size" {
                    Ok(Value::Number(set.values.len() as i64))
                } else if key == "constructor" {
                    Ok(Value::SetConstructor)
                } else {
                    Ok(Self::object_get_entry(&set.properties, key).unwrap_or(Value::Undefined))
                }
            }
            Value::Blob(blob) => {
                let blob = blob.borrow();
                match key {
                    "size" => Ok(Value::Number(blob.bytes.len() as i64)),
                    "type" => Ok(Value::String(blob.mime_type.clone())),
                    "constructor" => Ok(Value::BlobConstructor),
                    _ => Ok(Value::Undefined),
                }
            }
            Value::ArrayBuffer(_) => {
                if key == "constructor" {
                    Ok(Value::ArrayBufferConstructor)
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Symbol(symbol) => {
                let value = match key {
                    "description" => symbol
                        .description
                        .as_ref()
                        .map(|value| Value::String(value.clone()))
                        .unwrap_or(Value::Undefined),
                    "constructor" => Value::SymbolConstructor,
                    _ => Value::Undefined,
                };
                Ok(value)
            }
            Value::RegExp(regex) => {
                let regex = regex.borrow();
                let value = match key {
                    "source" => Value::String(regex.source.clone()),
                    "flags" => Value::String(regex.flags.clone()),
                    "global" => Value::Bool(regex.global),
                    "ignoreCase" => Value::Bool(regex.ignore_case),
                    "multiline" => Value::Bool(regex.multiline),
                    "dotAll" => Value::Bool(regex.dot_all),
                    "sticky" => Value::Bool(regex.sticky),
                    "hasIndices" => Value::Bool(regex.has_indices),
                    "unicode" => Value::Bool(regex.unicode),
                    "unicodeSets" => Value::Bool(regex.unicode_sets),
                    "lastIndex" => Value::Number(regex.last_index as i64),
                    "constructor" => Value::RegExpConstructor,
                    _ => Self::object_get_entry(&regex.properties, key).unwrap_or(Value::Undefined),
                };
                Ok(value)
            }
            Value::UrlConstructor => {
                if let Some(value) = Self::object_get_entry(
                    &self.browser_apis.url_constructor_properties.borrow(),
                    key,
                ) {
                    return Ok(value);
                }
                if Self::is_url_static_method_name(key) {
                    return Ok(Self::new_builtin_placeholder_function());
                }
                Ok(Value::Undefined)
            }
            Value::StringConstructor => Ok(Value::Undefined),
            _ => Err(Error::ScriptRuntime("value is not an object".into())),
        }
    }

    pub(crate) fn object_property_from_named_value(
        &self,
        variable_name: &str,
        value: &Value,
        key: &str,
    ) -> Result<Value> {
        self.object_property_from_value(value, key)
            .map_err(|err| match err {
                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                    Error::ScriptRuntime(format!(
                        "variable '{}' is not an object (key '{}')",
                        variable_name, key
                    ))
                }
                other => other,
            })
    }
}
