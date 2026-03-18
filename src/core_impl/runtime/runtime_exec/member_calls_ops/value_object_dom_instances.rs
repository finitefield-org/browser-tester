use super::*;

impl Harness {
    pub(crate) fn create_element_is_option_from_arg(arg: Option<&Value>) -> Option<String> {
        let arg = arg?;
        match arg {
            Value::Undefined | Value::Null => None,
            // Legacy compatibility: allow passing a string as the custom element name.
            Value::String(value) => Some(value.clone()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                match Self::object_get_entry(&entries, "is") {
                    Some(Value::Undefined) | Some(Value::Null) | None => None,
                    Some(value) => Some(value.as_string()),
                }
            }
            _ => None,
        }
    }

    pub(crate) fn canvas_dimension_default(name: &str) -> i64 {
        match name {
            "width" => 300,
            "height" => 150,
            _ => 0,
        }
    }

    pub(crate) fn canvas_dimension_value(&self, node: NodeId, name: &str) -> i64 {
        let default = if self.dom.tag_name(node).is_some_and(|tag| {
            tag.eq_ignore_ascii_case("canvas") || tag.eq_ignore_ascii_case("iframe")
        }) {
            Self::canvas_dimension_default(name)
        } else {
            0
        };
        self.dom
            .attr(node, name)
            .and_then(|raw| Self::parse_non_negative_int(&raw))
            .unwrap_or(default)
    }

    pub(crate) fn set_canvas_dimension_value(
        &mut self,
        node: NodeId,
        name: &str,
        value: &Value,
    ) -> Result<()> {
        let next = match value {
            Value::Number(number) => *number,
            Value::Float(number) if number.is_finite() => *number as i64,
            Value::BigInt(number) => number.to_string().parse::<i64>().unwrap_or(0),
            other => other.as_string().trim().parse::<i64>().unwrap_or(0),
        };
        let next = next.max(0);
        self.dom.set_attr(node, name, &next.to_string())
    }

    pub(crate) fn new_canvas_2d_context_value(&self, canvas_node: NodeId, alpha: bool) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CANVAS_2D_CONTEXT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (INTERNAL_CANVAS_2D_ALPHA_KEY.to_string(), Value::Bool(alpha)),
            (
                INTERNAL_CANVAS_2D_LINE_DASH_KEY.to_string(),
                Self::new_array_value(Vec::new()),
            ),
            (
                INTERNAL_CANVAS_2D_TRANSFORM_KEY.to_string(),
                Self::new_array_value(vec![
                    Value::Number(1),
                    Value::Number(0),
                    Value::Number(0),
                    Value::Number(1),
                    Value::Number(0),
                    Value::Number(0),
                ]),
            ),
            ("canvas".to_string(), Value::Node(canvas_node)),
            (
                "fillStyle".to_string(),
                Value::String("#000000".to_string()),
            ),
            (
                "strokeStyle".to_string(),
                Value::String("#000000".to_string()),
            ),
            ("lineWidth".to_string(), Value::Number(1)),
            ("lineCap".to_string(), Value::String("butt".to_string())),
            ("lineJoin".to_string(), Value::String("miter".to_string())),
            ("miterLimit".to_string(), Value::Number(10)),
            ("lineDashOffset".to_string(), Value::Number(0)),
            (
                "font".to_string(),
                Value::String("10px sans-serif".to_string()),
            ),
            ("textAlign".to_string(), Value::String("start".to_string())),
            (
                "textBaseline".to_string(),
                Value::String("alphabetic".to_string()),
            ),
            (
                "direction".to_string(),
                Value::String("inherit".to_string()),
            ),
            (
                "letterSpacing".to_string(),
                Value::String("0px".to_string()),
            ),
            ("fontKerning".to_string(), Value::String("auto".to_string())),
            (
                "fontStretch".to_string(),
                Value::String("normal".to_string()),
            ),
            (
                "fontVariantCaps".to_string(),
                Value::String("normal".to_string()),
            ),
            (
                "textRendering".to_string(),
                Value::String("auto".to_string()),
            ),
            ("wordSpacing".to_string(), Value::String("0px".to_string())),
            ("lang".to_string(), Value::String("inherit".to_string())),
            ("shadowBlur".to_string(), Value::Number(0)),
            (
                "shadowColor".to_string(),
                Value::String("rgba(0, 0, 0, 0)".to_string()),
            ),
            ("shadowOffsetX".to_string(), Value::Number(0)),
            ("shadowOffsetY".to_string(), Value::Number(0)),
            ("globalAlpha".to_string(), Value::Number(1)),
            (
                "globalCompositeOperation".to_string(),
                Value::String("source-over".to_string()),
            ),
            ("imageSmoothingEnabled".to_string(), Value::Bool(true)),
            (
                "imageSmoothingQuality".to_string(),
                Value::String("low".to_string()),
            ),
            ("filter".to_string(), Value::String("none".to_string())),
            (
                "clearRect".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "fillRect".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "strokeRect".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "fillText".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "strokeText".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "measureText".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "beginPath".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "closePath".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "moveTo".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "lineTo".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "bezierCurveTo".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "quadraticCurveTo".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("arc".to_string(), Self::new_builtin_placeholder_function()),
            (
                "arcTo".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "ellipse".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("rect".to_string(), Self::new_builtin_placeholder_function()),
            (
                "roundRect".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("fill".to_string(), Self::new_builtin_placeholder_function()),
            (
                "stroke".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "drawFocusIfNeeded".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("clip".to_string(), Self::new_builtin_placeholder_function()),
            (
                "isPointInPath".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "isPointInStroke".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "setLineDash".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getLineDash".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createConicGradient".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createLinearGradient".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createRadialGradient".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createPattern".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "drawImage".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createImageData".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getImageData".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "putImageData".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getTransform".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "rotate".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "scale".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "translate".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "transform".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "setTransform".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "resetTransform".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("save".to_string(), Self::new_builtin_placeholder_function()),
            (
                "restore".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "reset".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getContextAttributes".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "isContextLost".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "toString".to_string(),
                Self::new_receiver_builtin_callable("canvas_2d_context", "toString"),
            ),
            (
                "Symbol.toStringTag".to_string(),
                Value::String("CanvasRenderingContext2D".to_string()),
            ),
        ])
    }

    pub(crate) fn mock_file_to_value(file: &MockFile) -> Value {
        let file_blob = Self::new_blob_value(file.bytes.clone(), file.mime_type.clone());
        Self::new_object_value(vec![
            (INTERNAL_MOCK_FILE_OBJECT_KEY.to_string(), Value::Bool(true)),
            (INTERNAL_MOCK_FILE_BLOB_KEY.to_string(), file_blob),
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
            (
                "arrayBuffer".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("text".to_string(), Self::new_builtin_placeholder_function()),
            (
                "bytes".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "stream".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ])
    }

    fn input_files_type_error() -> Error {
        Error::ScriptRuntime(
            "TypeError: Failed to set the 'files' property on 'HTMLInputElement': The provided value is not of type 'FileList'."
                .into(),
        )
    }

    fn mock_file_from_input_assignment_value(&self, value: &Value) -> Result<MockFile> {
        let Value::Object(entries) = value else {
            return Err(Self::input_files_type_error());
        };
        let entries = entries.borrow();
        if !Self::is_mock_file_object(&entries) {
            return Err(Self::input_files_type_error());
        }

        let (bytes, blob_mime_type) =
            match Self::object_get_entry(&entries, INTERNAL_MOCK_FILE_BLOB_KEY) {
                Some(Value::Blob(blob)) => {
                    let blob = blob.borrow();
                    (blob.bytes.clone(), blob.mime_type.clone())
                }
                _ => (Vec::new(), String::new()),
            };

        let explicit_mime_type = Self::object_get_entry(&entries, "type")
            .map(|value| Self::normalize_blob_type(&value.as_string()))
            .unwrap_or_default();
        let mime_type = if explicit_mime_type.is_empty() {
            blob_mime_type
        } else {
            explicit_mime_type
        };
        let size = Self::object_get_entry(&entries, "size")
            .map(|value| Self::value_to_i64(&value).max(0))
            .unwrap_or(bytes.len() as i64);
        let file = MockFile {
            name: Self::object_get_entry(&entries, "name")
                .map(|value| value.as_string())
                .unwrap_or_default(),
            size,
            mime_type,
            last_modified: Self::object_get_entry(&entries, "lastModified")
                .map(|value| Self::value_to_i64(&value))
                .unwrap_or(0),
            webkit_relative_path: Self::object_get_entry(&entries, "webkitRelativePath")
                .map(|value| value.as_string())
                .unwrap_or_default(),
            bytes,
        };
        Ok(normalize_mock_file(&file))
    }

    pub(crate) fn mock_files_from_input_assignment_value(
        &self,
        value: &Value,
    ) -> Result<Vec<MockFile>> {
        if matches!(value, Value::Null | Value::Undefined) {
            return Ok(Vec::new());
        }

        let file_values = match value {
            Value::Array(values) => values.borrow().clone(),
            Value::Object(entries) => {
                let (is_mock_file, is_iterator, has_length) = {
                    let entries_ref = entries.borrow();
                    (
                        Self::is_mock_file_object(&entries_ref),
                        Self::is_iterator_object(&entries_ref),
                        Self::object_get_entry(&entries_ref, "length").is_some(),
                    )
                };
                if is_mock_file || (!is_iterator && !has_length) {
                    return Err(Self::input_files_type_error());
                }
                self.array_like_values_from_value(value)
                    .map_err(|_| Self::input_files_type_error())?
            }
            _ => self
                .array_like_values_from_value(value)
                .map_err(|_| Self::input_files_type_error())?,
        };

        let mut files = Vec::with_capacity(file_values.len());
        for file_value in file_values {
            files.push(self.mock_file_from_input_assignment_value(&file_value)?);
        }
        Ok(files)
    }

    pub(crate) fn new_dom_string_map_value(&self, node: NodeId) -> Value {
        let entries = vec![
            (
                INTERNAL_DOM_STRING_MAP_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_DOM_STRING_MAP_OWNER_NODE_KEY.to_string(),
                Value::Node(node),
            ),
        ];
        Self::new_object_value(entries)
    }

    pub(crate) fn new_class_list_value(node: NodeId) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CLASS_LIST_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (INTERNAL_CLASS_LIST_NODE_KEY.to_string(), Value::Node(node)),
        ])
    }

    pub(crate) fn new_image_bitmap_value(&mut self, width: i64, height: i64) -> Value {
        let object = Self::new_object_value(vec![
            (
                INTERNAL_IMAGE_BITMAP_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_IMAGE_BITMAP_WIDTH_KEY.to_string(),
                Value::Number(width.max(0)),
            ),
            (
                INTERNAL_IMAGE_BITMAP_HEIGHT_KEY.to_string(),
                Value::Number(height.max(0)),
            ),
        ]);
        if let Value::Object(entries) = &object {
            Self::set_internal_prototype(
                entries,
                self.cached_image_bitmap_constructor_prototype_value(),
            );
        }
        object
    }

    pub(crate) fn new_time_ranges_value(&mut self, media: NodeId, kind: &str) -> Value {
        let object = Self::new_object_value(vec![
            (
                INTERNAL_TIME_RANGES_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_TIME_RANGES_MEDIA_NODE_KEY.to_string(),
                Value::Node(media),
            ),
            (
                INTERNAL_TIME_RANGES_KIND_KEY.to_string(),
                Value::String(kind.to_string()),
            ),
        ]);
        if let Value::Object(entries) = &object {
            Self::set_internal_prototype(
                entries,
                self.cached_time_ranges_constructor_prototype_value(),
            );
        }
        object
    }

    pub(crate) fn text_track_object_value(&mut self, node: NodeId) -> Value {
        let existing = self.dom_runtime.live_text_track_objects.get(&node).cloned();
        let object = existing.unwrap_or_else(|| {
            let object = Rc::new(RefCell::new(ObjectValue::new(vec![
                (
                    INTERNAL_TEXT_TRACK_OBJECT_KEY.to_string(),
                    Value::Bool(true),
                ),
                (INTERNAL_TEXT_TRACK_NODE_KEY.to_string(), Value::Node(node)),
                (
                    INTERNAL_TEXT_TRACK_MODE_KEY.to_string(),
                    Value::String("disabled".to_string()),
                ),
            ])));
            Self::set_internal_prototype(
                &object,
                self.cached_text_track_constructor_prototype_value(),
            );
            self.dom_runtime
                .live_text_track_objects
                .insert(node, object.clone());
            object
        });
        Value::Object(object)
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
}
