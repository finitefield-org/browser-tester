use super::*;

const IMAGE_DATA_MAX_DEFAULT_ELEMENTS: usize = 1_000_000;

impl Harness {
    pub(crate) fn new_event_target_instance_from_constructor(
        &mut self,
        constructor: &Value,
        this_arg: Option<Value>,
    ) -> Result<Value> {
        if let Some(this_value) = this_arg {
            if Self::is_primitive_value(&this_value) {
                return Err(Error::ScriptRuntime(
                    "constructor this value must be an object".into(),
                ));
            }
            if let Value::Object(entries) = &this_value {
                Self::object_set_entry(
                    &mut entries.borrow_mut(),
                    INTERNAL_EVENT_TARGET_OBJECT_KEY.to_string(),
                    Value::Bool(true),
                );
            }
            return Ok(this_value);
        }

        let mut entries = vec![(
            INTERNAL_EVENT_TARGET_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )];
        if let Value::Object(constructor_entries) = constructor {
            let constructor_entries = constructor_entries.borrow();
            if let Some(prototype) = Self::object_get_entry(&constructor_entries, "prototype") {
                if matches!(prototype, Value::Object(_) | Value::Null) {
                    entries.push((INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(), prototype));
                }
            }
        }
        Ok(Self::new_object_value(entries))
    }

    fn image_data_expected_length(width: usize, height: usize) -> Result<usize> {
        width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| Error::ScriptRuntime("ImageData dimensions are too large".into()))
    }

    fn image_data_kind_for_pixel_format(pixel_format: &str) -> Option<TypedArrayKind> {
        match pixel_format {
            "rgba-unorm8" => Some(TypedArrayKind::Uint8Clamped),
            "rgba-float16" => Some(TypedArrayKind::Float16),
            _ => None,
        }
    }

    fn image_data_default_pixel_format_for_kind(kind: TypedArrayKind) -> Option<&'static str> {
        match kind {
            TypedArrayKind::Uint8Clamped => Some("rgba-unorm8"),
            TypedArrayKind::Float16 => Some("rgba-float16"),
            _ => None,
        }
    }

    fn image_data_settings_from_value(options: Option<&Value>) -> Result<(String, Option<String>)> {
        let Some(options) = options else {
            return Ok(("srgb".to_string(), None));
        };
        match options {
            Value::Null | Value::Undefined => Ok(("srgb".to_string(), None)),
            Value::Object(entries) => {
                let entries = entries.borrow();
                let color_space = Self::object_get_entry(&entries, "colorSpace")
                    .map(|value| value.as_string())
                    .unwrap_or_else(|| "srgb".to_string());
                if color_space != "srgb" && color_space != "display-p3" {
                    return Err(Error::ScriptRuntime(
                        "ImageData colorSpace must be \"srgb\" or \"display-p3\"".into(),
                    ));
                }
                let pixel_format =
                    Self::object_get_entry(&entries, "pixelFormat").map(|value| value.as_string());
                if let Some(pixel_format) = &pixel_format {
                    if Self::image_data_kind_for_pixel_format(pixel_format).is_none() {
                        return Err(Error::ScriptRuntime(
                            "ImageData pixelFormat must be \"rgba-unorm8\" or \"rgba-float16\""
                                .into(),
                        ));
                    }
                }
                Ok((color_space, pixel_format))
            }
            _ => Err(Error::ScriptRuntime(
                "ImageData constructor settings argument must be an object".into(),
            )),
        }
    }

    fn image_data_constructor_dimensions_require_positive(
        width: usize,
        height: usize,
    ) -> Result<()> {
        if width == 0 || height == 0 {
            return Err(Error::ScriptRuntime(
                "ImageData width and height must be greater than 0".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn new_image_data_value(
        &mut self,
        width: usize,
        height: usize,
        kind: TypedArrayKind,
        data_override: Option<Value>,
        color_space: &str,
        pixel_format: &str,
    ) -> Result<Value> {
        let width = i64::try_from(width)
            .map_err(|_| Error::ScriptRuntime("ImageData width is too large".into()))?;
        let height = i64::try_from(height)
            .map_err(|_| Error::ScriptRuntime("ImageData height is too large".into()))?;
        let data = if let Some(data) = data_override {
            data
        } else {
            let requested_len = Self::image_data_expected_length(width as usize, height as usize)?;
            let default_len = requested_len.min(IMAGE_DATA_MAX_DEFAULT_ELEMENTS);
            self.new_typed_array_with_length(kind, default_len)?
        };
        Ok(Self::new_object_value(vec![
            ("width".to_string(), Value::Number(width)),
            ("height".to_string(), Value::Number(height)),
            ("data".to_string(), data),
            (
                "colorSpace".to_string(),
                Value::String(color_space.to_string()),
            ),
            (
                "pixelFormat".to_string(),
                Value::String(pixel_format.to_string()),
            ),
        ]))
    }

    pub(crate) fn new_image_data_from_constructor_args(
        &mut self,
        args: &[Value],
    ) -> Result<Value> {
        if args.len() < 2 || args.len() > 4 {
            return Err(Error::ScriptRuntime(
                "ImageData constructor supports two to four arguments".into(),
            ));
        }

        match &args[0] {
            Value::TypedArray(array) => {
                let input_kind = array.borrow().kind;
                if !matches!(
                    input_kind,
                    TypedArrayKind::Uint8Clamped | TypedArrayKind::Float16
                ) {
                    return Err(Error::ScriptRuntime(
                        "ImageData data argument must be a Uint8ClampedArray or Float16Array"
                            .into(),
                    ));
                }

                let width = Self::to_non_negative_usize(&args[1], "ImageData width")?;
                if width == 0 {
                    return Err(Error::ScriptRuntime(
                        "ImageData width and height must be greater than 0".into(),
                    ));
                }

                let (raw_height, settings_value) = match args.len() {
                    2 => (None, None),
                    3 => match args[2] {
                        Value::Object(_) | Value::Null | Value::Undefined => (None, Some(&args[2])),
                        _ => (
                            Some(Self::to_non_negative_usize(&args[2], "ImageData height")?),
                            None,
                        ),
                    },
                    4 => (
                        Some(Self::to_non_negative_usize(&args[2], "ImageData height")?),
                        Some(&args[3]),
                    ),
                    _ => unreachable!(),
                };

                if raw_height == Some(0) {
                    return Err(Error::ScriptRuntime(
                        "ImageData width and height must be greater than 0".into(),
                    ));
                }

                let (color_space, settings_pixel_format) =
                    Self::image_data_settings_from_value(settings_value)?;
                let default_pixel_format =
                    Self::image_data_default_pixel_format_for_kind(input_kind).ok_or_else(
                        || Error::ScriptRuntime("unsupported ImageData typed array kind".into()),
                    )?;
                let pixel_format =
                    settings_pixel_format.unwrap_or_else(|| default_pixel_format.to_string());
                let pixel_format_kind = Self::image_data_kind_for_pixel_format(&pixel_format)
                    .ok_or_else(|| {
                        Error::ScriptRuntime(
                            "ImageData pixelFormat must be \"rgba-unorm8\" or \"rgba-float16\""
                                .into(),
                        )
                    })?;
                if pixel_format_kind != input_kind {
                    return Err(Error::ScriptRuntime(
                        "ImageData pixelFormat does not match data typed array kind".into(),
                    ));
                }

                let data_len = array.borrow().observed_length();
                let height = if let Some(height) = raw_height {
                    let expected = Self::image_data_expected_length(width, height)?;
                    if expected != data_len {
                        return Err(Error::ScriptRuntime(
                            "ImageData data length does not match width and height".into(),
                        ));
                    }
                    height
                } else {
                    let row_stride = width.checked_mul(4).ok_or_else(|| {
                        Error::ScriptRuntime("ImageData dimensions are too large".into())
                    })?;
                    if row_stride == 0 || data_len % row_stride != 0 {
                        return Err(Error::ScriptRuntime(
                            "ImageData data length is not compatible with the given width".into(),
                        ));
                    }
                    let resolved_height = data_len / row_stride;
                    if resolved_height == 0 {
                        return Err(Error::ScriptRuntime(
                            "ImageData width and height must be greater than 0".into(),
                        ));
                    }
                    resolved_height
                };

                Self::image_data_constructor_dimensions_require_positive(width, height)?;

                let data_values = self.typed_array_snapshot(array)?;
                let data_copy = self.new_typed_array_from_values(input_kind, &data_values)?;
                self.new_image_data_value(
                    width,
                    height,
                    input_kind,
                    Some(data_copy),
                    &color_space,
                    &pixel_format,
                )
            }
            _ => {
                if args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "ImageData(width, height) constructor supports up to three arguments"
                            .into(),
                    ));
                }
                let width = Self::to_non_negative_usize(&args[0], "ImageData width")?;
                let height = Self::to_non_negative_usize(&args[1], "ImageData height")?;
                Self::image_data_constructor_dimensions_require_positive(width, height)?;
                let (color_space, settings_pixel_format) =
                    Self::image_data_settings_from_value(args.get(2))?;
                let pixel_format =
                    settings_pixel_format.unwrap_or_else(|| "rgba-unorm8".to_string());
                let kind =
                    Self::image_data_kind_for_pixel_format(&pixel_format).ok_or_else(|| {
                        Error::ScriptRuntime(
                            "ImageData pixelFormat must be \"rgba-unorm8\" or \"rgba-float16\""
                                .into(),
                        )
                    })?;
                self.new_image_data_value(width, height, kind, None, &color_space, &pixel_format)
            }
        }
    }
}
