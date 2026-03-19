use super::*;

impl Harness {
    pub(crate) fn canvas_2d_alpha_from_options(options: &Value) -> bool {
        match options {
            Value::Object(entries) => {
                let entries = entries.borrow();
                Self::object_get_entry(&entries, "alpha")
                    .map(|value| value.truthy())
                    .unwrap_or(true)
            }
            _ => true,
        }
    }

    pub(crate) fn eval_canvas_2d_context_member_call(
        &mut self,
        context_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let mut context = context_object.borrow_mut();
        match member {
            "fillRect" | "clearRect" | "strokeRect" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} requires exactly four arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            "fillText" | "strokeText" => {
                if !(evaluated_args.len() == 3 || evaluated_args.len() == 4) {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} requires three or four arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            "measureText" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "measureText supports at most one argument".into(),
                    ));
                }
                let text = evaluated_args
                    .first()
                    .map(Value::as_string)
                    .unwrap_or_else(|| "undefined".to_string());
                let width = text.chars().count() as f64 * 10.0;
                Ok(Some(Self::new_object_value(vec![(
                    "width".to_string(),
                    Self::number_value(width),
                )])))
            }
            "beginPath" | "closePath" | "save" | "restore" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(format!("{member} takes no arguments")));
                }
                Ok(Some(Value::Undefined))
            }
            "fill" | "stroke" | "clip" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} supports at most two arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            "moveTo" | "lineTo" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} requires exactly two arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            "arc" => {
                if evaluated_args.len() < 5 || evaluated_args.len() > 6 {
                    return Err(Error::ScriptRuntime(
                        "arc requires five or six arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "arcTo" => {
                if evaluated_args.len() != 5 {
                    return Err(Error::ScriptRuntime(
                        "arcTo requires exactly five arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "bezierCurveTo" => {
                if evaluated_args.len() != 6 {
                    return Err(Error::ScriptRuntime(
                        "bezierCurveTo requires exactly six arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "quadraticCurveTo" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(
                        "quadraticCurveTo requires exactly four arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "ellipse" => {
                if evaluated_args.len() < 7 || evaluated_args.len() > 8 {
                    return Err(Error::ScriptRuntime(
                        "ellipse requires seven or eight arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "rect" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(
                        "rect requires exactly four arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "roundRect" => {
                if evaluated_args.len() != 5 {
                    return Err(Error::ScriptRuntime(
                        "roundRect requires exactly five arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "drawFocusIfNeeded" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "drawFocusIfNeeded supports at most one argument".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "isPointInPath" | "isPointInStroke" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} requires two or three arguments"
                    )));
                }
                Ok(Some(Value::Bool(false)))
            }
            "setLineDash" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setLineDash requires exactly one argument".into(),
                    ));
                }
                let mut line_dash = self.canvas_2d_line_dash_values(&evaluated_args[0])?;
                if line_dash.len() % 2 == 1 {
                    let copy = line_dash.clone();
                    line_dash.extend(copy);
                }
                Self::canvas_2d_store_line_dash(&mut context, &line_dash);
                Ok(Some(Value::Undefined))
            }
            "getLineDash" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getLineDash takes no arguments".into(),
                    ));
                }
                Ok(Some(Self::new_array_value(
                    Self::canvas_2d_read_line_dash(&context)
                        .into_iter()
                        .map(Self::number_value)
                        .collect(),
                )))
            }
            "createLinearGradient" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(
                        "createLinearGradient requires exactly four arguments".into(),
                    ));
                }
                Ok(Some(Self::new_canvas_gradient_value()))
            }
            "createRadialGradient" => {
                if evaluated_args.len() != 6 {
                    return Err(Error::ScriptRuntime(
                        "createRadialGradient requires exactly six arguments".into(),
                    ));
                }
                Ok(Some(Self::new_canvas_gradient_value()))
            }
            "createConicGradient" => {
                if evaluated_args.len() != 3 {
                    return Err(Error::ScriptRuntime(
                        "createConicGradient requires exactly three arguments".into(),
                    ));
                }
                Ok(Some(Self::new_canvas_gradient_value()))
            }
            "createPattern" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "createPattern requires exactly two arguments".into(),
                    ));
                }
                if matches!(evaluated_args[0], Value::Null | Value::Undefined) {
                    return Ok(Some(Value::Null));
                }
                Ok(Some(Self::new_canvas_pattern_value()))
            }
            "drawImage" => {
                if !(evaluated_args.len() == 3
                    || evaluated_args.len() == 5
                    || evaluated_args.len() == 9)
                {
                    return Err(Error::ScriptRuntime(
                        "drawImage requires three, five, or nine arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "createImageData" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "createImageData requires one or two arguments".into(),
                    ));
                }
                let (width, height) = if evaluated_args.len() == 1 {
                    let Value::Object(entries) = &evaluated_args[0] else {
                        return Err(Error::ScriptRuntime(
                            "createImageData(imageData) requires an ImageData-like object".into(),
                        ));
                    };
                    let entries = entries.borrow();
                    let width = Self::object_get_entry(&entries, "width")
                        .map(|value| Self::coerce_number_for_number_constructor(&value) as i64)
                        .unwrap_or(0)
                        .max(0);
                    let height = Self::object_get_entry(&entries, "height")
                        .map(|value| Self::coerce_number_for_number_constructor(&value) as i64)
                        .unwrap_or(0)
                        .max(0);
                    (width, height)
                } else {
                    (
                        Self::coerce_number_for_number_constructor(&evaluated_args[0]) as i64,
                        Self::coerce_number_for_number_constructor(&evaluated_args[1]) as i64,
                    )
                };
                Ok(Some(self.new_canvas_image_data_value(width, height)?))
            }
            "getImageData" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(
                        "getImageData requires exactly four arguments".into(),
                    ));
                }
                let width =
                    Self::coerce_number_for_number_constructor(&evaluated_args[2]).abs() as i64;
                let height =
                    Self::coerce_number_for_number_constructor(&evaluated_args[3]).abs() as i64;
                Ok(Some(self.new_canvas_image_data_value(width, height)?))
            }
            "putImageData" => {
                if !(evaluated_args.len() == 3 || evaluated_args.len() == 7) {
                    return Err(Error::ScriptRuntime(
                        "putImageData requires three or seven arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "getTransform" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getTransform takes no arguments".into(),
                    ));
                }
                let transform = Self::canvas_2d_read_transform(&context);
                Ok(Some(Self::new_canvas_transform_value(transform)))
            }
            "transform" => {
                if evaluated_args.len() != 6 {
                    return Err(Error::ScriptRuntime(
                        "transform requires exactly six arguments".into(),
                    ));
                }
                let next = [
                    Self::coerce_number_for_number_constructor(&evaluated_args[0]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[1]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[2]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[3]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[4]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[5]),
                ];
                let current = Self::canvas_2d_read_transform(&context);
                Self::canvas_2d_store_transform(
                    &mut context,
                    Self::canvas_2d_multiply_transform(current, next),
                );
                Ok(Some(Value::Undefined))
            }
            "setTransform" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 6) {
                    return Err(Error::ScriptRuntime(
                        "setTransform requires one or six arguments".into(),
                    ));
                }
                let next = if evaluated_args.len() == 1 {
                    let Value::Object(entries) = &evaluated_args[0] else {
                        return Err(Error::ScriptRuntime(
                            "setTransform(matrix) requires an object argument".into(),
                        ));
                    };
                    Self::canvas_2d_transform_from_object_entries(&entries.borrow())
                } else {
                    [
                        Self::coerce_number_for_number_constructor(&evaluated_args[0]),
                        Self::coerce_number_for_number_constructor(&evaluated_args[1]),
                        Self::coerce_number_for_number_constructor(&evaluated_args[2]),
                        Self::coerce_number_for_number_constructor(&evaluated_args[3]),
                        Self::coerce_number_for_number_constructor(&evaluated_args[4]),
                        Self::coerce_number_for_number_constructor(&evaluated_args[5]),
                    ]
                };
                Self::canvas_2d_store_transform(&mut context, next);
                Ok(Some(Value::Undefined))
            }
            "resetTransform" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "resetTransform takes no arguments".into(),
                    ));
                }
                Self::canvas_2d_store_transform(&mut context, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
                Ok(Some(Value::Undefined))
            }
            "scale" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "scale requires exactly two arguments".into(),
                    ));
                }
                let current = Self::canvas_2d_read_transform(&context);
                let next = [
                    Self::coerce_number_for_number_constructor(&evaluated_args[0]),
                    0.0,
                    0.0,
                    Self::coerce_number_for_number_constructor(&evaluated_args[1]),
                    0.0,
                    0.0,
                ];
                Self::canvas_2d_store_transform(
                    &mut context,
                    Self::canvas_2d_multiply_transform(current, next),
                );
                Ok(Some(Value::Undefined))
            }
            "translate" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "translate requires exactly two arguments".into(),
                    ));
                }
                let current = Self::canvas_2d_read_transform(&context);
                let next = [
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    Self::coerce_number_for_number_constructor(&evaluated_args[0]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[1]),
                ];
                Self::canvas_2d_store_transform(
                    &mut context,
                    Self::canvas_2d_multiply_transform(current, next),
                );
                Ok(Some(Value::Undefined))
            }
            "rotate" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "rotate requires exactly one argument".into(),
                    ));
                }
                let radians = Self::coerce_number_for_number_constructor(&evaluated_args[0]);
                let current = Self::canvas_2d_read_transform(&context);
                let next = [
                    radians.cos(),
                    radians.sin(),
                    -radians.sin(),
                    radians.cos(),
                    0.0,
                    0.0,
                ];
                Self::canvas_2d_store_transform(
                    &mut context,
                    Self::canvas_2d_multiply_transform(current, next),
                );
                Ok(Some(Value::Undefined))
            }
            "reset" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("reset takes no arguments".into()));
                }
                Self::canvas_2d_reset_context_state(&mut context);
                Ok(Some(Value::Undefined))
            }
            "getContextAttributes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getContextAttributes takes no arguments".into(),
                    ));
                }
                let alpha = Self::object_get_entry(&context, INTERNAL_CANVAS_2D_ALPHA_KEY)
                    .map(|value| value.truthy())
                    .unwrap_or(true);
                Ok(Some(Self::new_object_value(vec![
                    ("alpha".to_string(), Value::Bool(alpha)),
                    ("colorSpace".to_string(), Value::String("srgb".to_string())),
                    ("desynchronized".to_string(), Value::Bool(false)),
                    ("willReadFrequently".to_string(), Value::Bool(false)),
                ])))
            }
            "isContextLost" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "isContextLost takes no arguments".into(),
                    ));
                }
                Ok(Some(Value::Bool(false)))
            }
            "toString" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("toString takes no arguments".into()));
                }
                Ok(Some(Value::String(
                    "[object CanvasRenderingContext2D]".to_string(),
                )))
            }
            _ => Ok(None),
        }
    }

    fn canvas_2d_line_dash_values(&mut self, value: &Value) -> Result<Vec<f64>> {
        let values = match value {
            Value::Array(values) => values.borrow().elements.clone(),
            Value::TypedArray(values) => self.typed_array_snapshot(values)?,
            _ => vec![value.clone()],
        };
        Ok(values
            .into_iter()
            .map(|entry| Self::coerce_number_for_number_constructor(&entry))
            .map(|entry| {
                if entry.is_finite() && entry >= 0.0 {
                    entry
                } else {
                    0.0
                }
            })
            .collect())
    }

    fn canvas_2d_store_line_dash(context: &mut ObjectValue, line_dash: &[f64]) {
        Self::object_set_entry(
            context,
            INTERNAL_CANVAS_2D_LINE_DASH_KEY.to_string(),
            Self::new_array_value(line_dash.iter().copied().map(Self::number_value).collect()),
        );
    }

    fn canvas_2d_read_line_dash(context: &ObjectValue) -> Vec<f64> {
        let Some(Value::Array(values)) =
            Self::object_get_entry(context, INTERNAL_CANVAS_2D_LINE_DASH_KEY)
        else {
            return Vec::new();
        };
        values
            .borrow()
            .elements
            .iter()
            .map(Self::coerce_number_for_number_constructor)
            .map(|value| if value.is_finite() { value } else { 0.0 })
            .collect()
    }

    fn canvas_2d_transform_from_object_entries(entries: &ObjectValue) -> [f64; 6] {
        let get = |key: &str, default: f64| {
            Self::object_get_entry(entries, key)
                .map(|value| Self::coerce_number_for_number_constructor(&value))
                .filter(|value| value.is_finite())
                .unwrap_or(default)
        };
        [
            get("a", 1.0),
            get("b", 0.0),
            get("c", 0.0),
            get("d", 1.0),
            get("e", 0.0),
            get("f", 0.0),
        ]
    }

    fn canvas_2d_read_transform(context: &ObjectValue) -> [f64; 6] {
        let Some(Value::Array(values)) =
            Self::object_get_entry(context, INTERNAL_CANVAS_2D_TRANSFORM_KEY)
        else {
            return [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        };
        let values = values.borrow();
        let read = |index: usize, default: f64| {
            values
                .elements
                .get(index)
                .map(Self::coerce_number_for_number_constructor)
                .filter(|value| value.is_finite())
                .unwrap_or(default)
        };
        [
            read(0, 1.0),
            read(1, 0.0),
            read(2, 0.0),
            read(3, 1.0),
            read(4, 0.0),
            read(5, 0.0),
        ]
    }

    fn canvas_2d_store_transform(context: &mut ObjectValue, transform: [f64; 6]) {
        Self::object_set_entry(
            context,
            INTERNAL_CANVAS_2D_TRANSFORM_KEY.to_string(),
            Self::new_array_value(transform.into_iter().map(Self::number_value).collect()),
        );
    }

    fn canvas_2d_multiply_transform(left: [f64; 6], right: [f64; 6]) -> [f64; 6] {
        [
            left[0] * right[0] + left[2] * right[1],
            left[1] * right[0] + left[3] * right[1],
            left[0] * right[2] + left[2] * right[3],
            left[1] * right[2] + left[3] * right[3],
            left[0] * right[4] + left[2] * right[5] + left[4],
            left[1] * right[4] + left[3] * right[5] + left[5],
        ]
    }

    fn new_canvas_transform_value(transform: [f64; 6]) -> Value {
        Self::new_object_value(vec![
            ("a".to_string(), Self::number_value(transform[0])),
            ("b".to_string(), Self::number_value(transform[1])),
            ("c".to_string(), Self::number_value(transform[2])),
            ("d".to_string(), Self::number_value(transform[3])),
            ("e".to_string(), Self::number_value(transform[4])),
            ("f".to_string(), Self::number_value(transform[5])),
        ])
    }

    fn new_canvas_gradient_value() -> Value {
        Self::new_object_value(vec![(
            "addColorStop".to_string(),
            Self::new_builtin_placeholder_function(),
        )])
    }

    fn new_canvas_pattern_value() -> Value {
        Self::new_object_value(vec![(
            "setTransform".to_string(),
            Self::new_builtin_placeholder_function(),
        )])
    }

    fn new_canvas_image_data_value(&mut self, width: i64, height: i64) -> Result<Value> {
        self.new_image_data_value(
            width.max(0) as usize,
            height.max(0) as usize,
            TypedArrayKind::Uint8Clamped,
            None,
            "srgb",
            "rgba-unorm8",
        )
    }

    fn canvas_2d_reset_context_state(context: &mut ObjectValue) {
        Self::object_set_entry(
            context,
            "fillStyle".to_string(),
            Value::String("#000000".to_string()),
        );
        Self::object_set_entry(
            context,
            "strokeStyle".to_string(),
            Value::String("#000000".to_string()),
        );
        Self::object_set_entry(context, "lineWidth".to_string(), Value::Number(1));
        Self::object_set_entry(
            context,
            "lineCap".to_string(),
            Value::String("butt".to_string()),
        );
        Self::object_set_entry(
            context,
            "lineJoin".to_string(),
            Value::String("miter".to_string()),
        );
        Self::object_set_entry(context, "miterLimit".to_string(), Value::Number(10));
        Self::object_set_entry(context, "lineDashOffset".to_string(), Value::Number(0));
        Self::object_set_entry(
            context,
            "font".to_string(),
            Value::String("10px sans-serif".to_string()),
        );
        Self::object_set_entry(
            context,
            "textAlign".to_string(),
            Value::String("start".to_string()),
        );
        Self::object_set_entry(
            context,
            "textBaseline".to_string(),
            Value::String("alphabetic".to_string()),
        );
        Self::object_set_entry(
            context,
            "direction".to_string(),
            Value::String("inherit".to_string()),
        );
        Self::object_set_entry(
            context,
            "letterSpacing".to_string(),
            Value::String("0px".to_string()),
        );
        Self::object_set_entry(
            context,
            "fontKerning".to_string(),
            Value::String("auto".to_string()),
        );
        Self::object_set_entry(
            context,
            "fontStretch".to_string(),
            Value::String("normal".to_string()),
        );
        Self::object_set_entry(
            context,
            "fontVariantCaps".to_string(),
            Value::String("normal".to_string()),
        );
        Self::object_set_entry(
            context,
            "textRendering".to_string(),
            Value::String("auto".to_string()),
        );
        Self::object_set_entry(
            context,
            "wordSpacing".to_string(),
            Value::String("0px".to_string()),
        );
        Self::object_set_entry(
            context,
            "lang".to_string(),
            Value::String("inherit".to_string()),
        );
        Self::object_set_entry(context, "shadowBlur".to_string(), Value::Number(0));
        Self::object_set_entry(
            context,
            "shadowColor".to_string(),
            Value::String("rgba(0, 0, 0, 0)".to_string()),
        );
        Self::object_set_entry(context, "shadowOffsetX".to_string(), Value::Number(0));
        Self::object_set_entry(context, "shadowOffsetY".to_string(), Value::Number(0));
        Self::object_set_entry(context, "globalAlpha".to_string(), Value::Number(1));
        Self::object_set_entry(
            context,
            "globalCompositeOperation".to_string(),
            Value::String("source-over".to_string()),
        );
        Self::object_set_entry(
            context,
            "imageSmoothingEnabled".to_string(),
            Value::Bool(true),
        );
        Self::object_set_entry(
            context,
            "imageSmoothingQuality".to_string(),
            Value::String("low".to_string()),
        );
        Self::object_set_entry(
            context,
            "filter".to_string(),
            Value::String("none".to_string()),
        );
        Self::canvas_2d_store_line_dash(context, &[]);
        Self::canvas_2d_store_transform(context, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }
}
