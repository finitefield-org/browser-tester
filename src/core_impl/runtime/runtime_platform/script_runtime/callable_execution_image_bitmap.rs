use super::*;

impl Harness {
    pub(crate) fn eval_create_image_bitmap_call(&mut self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Err(Error::ScriptRuntime(
                "createImageBitmap requires at least one argument".into(),
            ));
        }

        let promise = self.new_pending_promise();
        match self.create_image_bitmap_dimensions_from_args(args) {
            Ok((width, height)) => {
                let bitmap = self.new_image_bitmap_value(width, height);
                self.promise_resolve(&promise, bitmap)?;
            }
            Err(err) => {
                self.promise_reject(&promise, Value::String(err));
            }
        }
        Ok(Value::Promise(promise))
    }

    fn create_image_bitmap_dimensions_from_args(
        &self,
        args: &[Value],
    ) -> std::result::Result<(i64, i64), String> {
        let (source_width, source_height) =
            self.create_image_bitmap_dimensions_from_value(&args[0])?;
        let mut width = source_width;
        let mut height = source_height;

        let options = match args.len() {
            1 => None,
            2 => Some(&args[1]),
            5 => {
                let crop_width = Self::value_to_i64(&args[3]).abs();
                let crop_height = Self::value_to_i64(&args[4]).abs();
                if crop_width == 0 || crop_height == 0 {
                    return Err("createImageBitmap crop width/height must be non-zero".to_string());
                }
                width = crop_width;
                height = crop_height;
                None
            }
            6 => {
                let crop_width = Self::value_to_i64(&args[3]).abs();
                let crop_height = Self::value_to_i64(&args[4]).abs();
                if crop_width == 0 || crop_height == 0 {
                    return Err("createImageBitmap crop width/height must be non-zero".to_string());
                }
                width = crop_width;
                height = crop_height;
                Some(&args[5])
            }
            _ => {
                return Err("createImageBitmap supports 1, 2, 5, or 6 arguments".to_string());
            }
        };

        let (resize_width, resize_height) =
            self.create_image_bitmap_resize_from_options(options)?;
        if let Some(resize_width) = resize_width {
            width = resize_width;
        }
        if let Some(resize_height) = resize_height {
            height = resize_height;
        }

        Ok((width.max(1), height.max(1)))
    }

    fn create_image_bitmap_resize_from_options(
        &self,
        options: Option<&Value>,
    ) -> std::result::Result<(Option<i64>, Option<i64>), String> {
        let Some(options) = options else {
            return Ok((None, None));
        };

        match options {
            Value::Null | Value::Undefined => Ok((None, None)),
            Value::Object(entries) => {
                let entries = entries.borrow();
                let resize_width = match Self::object_get_entry(&entries, "resizeWidth") {
                    Some(Value::Null | Value::Undefined) | None => None,
                    Some(value) => {
                        let width = Self::value_to_i64(&value);
                        if width <= 0 {
                            return Err("createImageBitmap resizeWidth must be a positive integer"
                                .to_string());
                        }
                        Some(width)
                    }
                };
                let resize_height = match Self::object_get_entry(&entries, "resizeHeight") {
                    Some(Value::Null | Value::Undefined) | None => None,
                    Some(value) => {
                        let height = Self::value_to_i64(&value);
                        if height <= 0 {
                            return Err(
                                "createImageBitmap resizeHeight must be a positive integer"
                                    .to_string(),
                            );
                        }
                        Some(height)
                    }
                };
                Ok((resize_width, resize_height))
            }
            _ => Err("createImageBitmap options must be an object".to_string()),
        }
    }

    fn create_image_bitmap_dimensions_from_value(
        &self,
        source: &Value,
    ) -> std::result::Result<(i64, i64), String> {
        let (bytes, mime_type, logical_size) = match source {
            Value::Blob(blob) => {
                let blob = blob.borrow();
                (
                    blob.bytes.clone(),
                    blob.mime_type.clone(),
                    blob.bytes.len() as i64,
                )
            }
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_image_bitmap_object(&entries) {
                    let width =
                        match Self::object_get_entry(&entries, INTERNAL_IMAGE_BITMAP_WIDTH_KEY) {
                            Some(Value::Number(width)) => width,
                            _ => 0,
                        };
                    let height =
                        match Self::object_get_entry(&entries, INTERNAL_IMAGE_BITMAP_HEIGHT_KEY) {
                            Some(Value::Number(height)) => height,
                            _ => 0,
                        };
                    if width > 0 && height > 0 {
                        return Ok((width, height));
                    }
                }
                let width = Self::object_get_entry(&entries, "width")
                    .map(|value| Self::value_to_i64(&value));
                let height = Self::object_get_entry(&entries, "height")
                    .map(|value| Self::value_to_i64(&value));
                if let (Some(width), Some(height)) = (width, height) {
                    if width > 0 && height > 0 {
                        return Ok((width, height));
                    }
                }

                if !Self::is_mock_file_object(&entries) {
                    return Err(
                        "createImageBitmap requires an image Blob or File source".to_string()
                    );
                }

                let blob = match Self::object_get_entry(&entries, INTERNAL_MOCK_FILE_BLOB_KEY) {
                    Some(Value::Blob(blob)) => blob,
                    _ => {
                        return Err(
                            "createImageBitmap could not access mock file bytes".to_string()
                        );
                    }
                };
                let (bytes, mime_type) = {
                    let blob = blob.borrow();
                    (blob.bytes.clone(), blob.mime_type.clone())
                };
                let logical_size = Self::object_get_entry(&entries, "size")
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(bytes.len() as i64);
                (bytes, mime_type, logical_size)
            }
            Value::Node(node) => {
                let tag = self
                    .dom
                    .tag_name(*node)
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                let dimensions = match tag.as_str() {
                    "canvas" => {
                        let width = self
                            .dom
                            .attr(*node, "width")
                            .and_then(|value| value.parse::<i64>().ok())
                            .unwrap_or(300);
                        let height = self
                            .dom
                            .attr(*node, "height")
                            .and_then(|value| value.parse::<i64>().ok())
                            .unwrap_or(150);
                        Some((width, height))
                    }
                    _ => None,
                };
                let Some((width, height)) = dimensions else {
                    return Err(
                        "createImageBitmap requires an image Blob or File source".to_string()
                    );
                };
                if width <= 0 || height <= 0 {
                    return Err("createImageBitmap could not decode image source".to_string());
                }
                return Ok((width, height));
            }
            _ => {
                return Err("createImageBitmap requires an image Blob or File source".to_string());
            }
        };

        let mime = mime_type.to_ascii_lowercase();
        let (width, height) = Self::decode_image_dimensions(&bytes)
            .or_else(|| {
                if mime.starts_with("image/") && (logical_size > 0 || !bytes.is_empty()) {
                    Some((1, 1))
                } else {
                    None
                }
            })
            .ok_or_else(|| "createImageBitmap could not decode image source".to_string())?;

        Ok((width.max(1), height.max(1)))
    }

    fn decode_image_dimensions(bytes: &[u8]) -> Option<(i64, i64)> {
        Self::decode_png_dimensions(bytes)
            .or_else(|| Self::decode_gif_dimensions(bytes))
            .or_else(|| Self::decode_jpeg_dimensions(bytes))
    }

    fn decode_png_dimensions(bytes: &[u8]) -> Option<(i64, i64)> {
        const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
        if bytes.len() < 24 || bytes[0..8] != PNG_SIGNATURE {
            return None;
        }
        if &bytes[12..16] != b"IHDR" {
            return None;
        }
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        if width == 0 || height == 0 {
            return None;
        }
        Some((width as i64, height as i64))
    }

    fn decode_gif_dimensions(bytes: &[u8]) -> Option<(i64, i64)> {
        if bytes.len() < 10 {
            return None;
        }
        if &bytes[0..6] != b"GIF87a" && &bytes[0..6] != b"GIF89a" {
            return None;
        }
        let width = u16::from_le_bytes([bytes[6], bytes[7]]);
        let height = u16::from_le_bytes([bytes[8], bytes[9]]);
        if width == 0 || height == 0 {
            return None;
        }
        Some((width as i64, height as i64))
    }

    fn decode_jpeg_dimensions(bytes: &[u8]) -> Option<(i64, i64)> {
        if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
            return None;
        }

        let mut offset = 2usize;
        while offset + 1 < bytes.len() {
            if bytes[offset] != 0xFF {
                offset += 1;
                continue;
            }
            while offset < bytes.len() && bytes[offset] == 0xFF {
                offset += 1;
            }
            if offset >= bytes.len() {
                break;
            }
            let marker = bytes[offset];
            offset += 1;

            if marker == 0xD9 || marker == 0xDA {
                break;
            }
            if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
                continue;
            }

            if offset + 1 >= bytes.len() {
                break;
            }
            let segment_len = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
            offset += 2;
            if segment_len < 2 || offset + segment_len - 2 > bytes.len() {
                break;
            }

            let is_sof = matches!(
                marker,
                0xC0 | 0xC1
                    | 0xC2
                    | 0xC3
                    | 0xC5
                    | 0xC6
                    | 0xC7
                    | 0xC9
                    | 0xCA
                    | 0xCB
                    | 0xCD
                    | 0xCE
                    | 0xCF
            );
            if is_sof && segment_len >= 7 {
                let height = u16::from_be_bytes([bytes[offset + 1], bytes[offset + 2]]);
                let width = u16::from_be_bytes([bytes[offset + 3], bytes[offset + 4]]);
                if width > 0 && height > 0 {
                    return Some((width as i64, height as i64));
                }
                return None;
            }

            offset += segment_len - 2;
        }

        None
    }
}
