use super::*;

impl Harness {
    pub(crate) fn text_encoder_encode_into_value(
        &mut self,
        source: &str,
        destination: &Rc<RefCell<TypedArrayValue>>,
    ) -> Result<Value> {
        if destination.borrow().kind != TypedArrayKind::Uint8 {
            return Err(Error::ScriptRuntime(
                "TextEncoder.encodeInto destination must be a Uint8Array".into(),
            ));
        }

        let capacity = destination.borrow().observed_length();
        let mut read_utf16_units = 0usize;
        let mut written_bytes = 0usize;

        for ch in source.chars() {
            let mut encoded = [0u8; 4];
            let encoded = ch.encode_utf8(&mut encoded).as_bytes();
            if written_bytes.saturating_add(encoded.len()) > capacity {
                break;
            }

            for byte in encoded {
                self.typed_array_set_index(
                    destination,
                    written_bytes,
                    Value::Number(i64::from(*byte)),
                )?;
                written_bytes = written_bytes.saturating_add(1);
            }
            read_utf16_units = read_utf16_units.saturating_add(ch.len_utf16());
        }

        Ok(Self::new_object_value(vec![
            ("read".to_string(), Value::Number(read_utf16_units as i64)),
            ("written".to_string(), Value::Number(written_bytes as i64)),
        ]))
    }

    pub(crate) fn text_encoder_stream_state_from_receiver(
        receiver: Option<&Value>,
    ) -> Result<(Value, Value)> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "TextEncoderStream getter called on incompatible receiver".into(),
            ));
        };
        let entries = entries.borrow();
        let is_text_encoder_stream = matches!(
            Self::object_get_entry(&entries, INTERNAL_TEXT_ENCODER_STREAM_OBJECT_KEY),
            Some(Value::Bool(true))
        );
        if !is_text_encoder_stream {
            return Err(Error::ScriptRuntime(
                "TextEncoderStream getter called on incompatible receiver".into(),
            ));
        }
        let readable = Self::object_get_entry(&entries, INTERNAL_TEXT_ENCODER_STREAM_READABLE_KEY)
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "TextEncoderStream getter called on incompatible receiver".into(),
                )
            })?;
        let writable = Self::object_get_entry(&entries, INTERNAL_TEXT_ENCODER_STREAM_WRITABLE_KEY)
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "TextEncoderStream getter called on incompatible receiver".into(),
                )
            })?;
        Ok((readable, writable))
    }

    pub(crate) fn text_encoder_receiver_object(
        receiver: Option<&Value>,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "TextEncoder method called on incompatible receiver".into(),
            ));
        };
        let is_text_encoder = {
            let entries_ref = entries.borrow();
            matches!(
                Self::object_get_entry(&entries_ref, INTERNAL_TEXT_ENCODER_OBJECT_KEY),
                Some(Value::Bool(true))
            )
        };
        if !is_text_encoder {
            return Err(Error::ScriptRuntime(
                "TextEncoder method called on incompatible receiver".into(),
            ));
        }
        Ok(entries.clone())
    }

    pub(crate) fn text_decoder_receiver_object(
        receiver: Option<&Value>,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "TextDecoder method called on incompatible receiver".into(),
            ));
        };
        let is_text_decoder = {
            let entries_ref = entries.borrow();
            matches!(
                Self::object_get_entry(&entries_ref, INTERNAL_TEXT_DECODER_OBJECT_KEY),
                Some(Value::Bool(true))
            )
        };
        if !is_text_decoder {
            return Err(Error::ScriptRuntime(
                "TextDecoder method called on incompatible receiver".into(),
            ));
        }
        Ok(entries.clone())
    }

    pub(crate) fn text_decoder_stream_state_from_receiver(
        receiver: Option<&Value>,
    ) -> Result<(String, bool, bool, Value, Value)> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "TextDecoderStream getter called on incompatible receiver".into(),
            ));
        };
        let entries = entries.borrow();
        let is_text_decoder_stream = matches!(
            Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_OBJECT_KEY),
            Some(Value::Bool(true))
        );
        if !is_text_decoder_stream {
            return Err(Error::ScriptRuntime(
                "TextDecoderStream getter called on incompatible receiver".into(),
            ));
        }
        let encoding =
            match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_ENCODING_KEY) {
                Some(Value::String(value)) => value,
                _ => {
                    return Err(Error::ScriptRuntime(
                        "TextDecoderStream getter called on incompatible receiver".into(),
                    ));
                }
            };
        let fatal = match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_FATAL_KEY) {
            Some(Value::Bool(value)) => value,
            _ => false,
        };
        let ignore_bom =
            match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_IGNORE_BOM_KEY) {
                Some(Value::Bool(value)) => value,
                _ => false,
            };
        let readable = Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_READABLE_KEY)
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "TextDecoderStream getter called on incompatible receiver".into(),
                )
            })?;
        let writable = Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_WRITABLE_KEY)
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "TextDecoderStream getter called on incompatible receiver".into(),
                )
            })?;
        Ok((encoding, fatal, ignore_bom, readable, writable))
    }

    pub(crate) fn normalize_text_decoder_label(raw: &str) -> Option<&'static str> {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "utf-8" | "utf8" | "unicode-1-1-utf-8" => Some("utf-8"),
            "windows-1251" | "cp1251" | "x-cp1251" => Some("windows-1251"),
            _ => None,
        }
    }

    pub(crate) fn text_decoder_state_from_receiver(
        receiver: Option<&Value>,
    ) -> Result<(String, bool, bool)> {
        let entries = Self::text_decoder_receiver_object(receiver)?;
        let entries = entries.borrow();
        let encoding = match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_ENCODING_KEY) {
            Some(Value::String(encoding)) => encoding,
            _ => {
                return Err(Error::ScriptRuntime(
                    "TextDecoder method called on incompatible receiver".into(),
                ));
            }
        };
        let fatal = match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_FATAL_KEY) {
            Some(Value::Bool(fatal)) => fatal,
            _ => false,
        };
        let ignore_bom =
            match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_IGNORE_BOM_KEY) {
                Some(Value::Bool(ignore_bom)) => ignore_bom,
                _ => false,
            };
        Ok((encoding, fatal, ignore_bom))
    }

    pub(crate) fn text_decoder_options_from_value(
        options: Option<&Value>,
    ) -> Result<(bool, bool)> {
        let Some(options) = options else {
            return Ok((false, false));
        };
        match options {
            Value::Undefined | Value::Null => Ok((false, false)),
            Value::Object(entries) => {
                let entries = entries.borrow();
                let fatal =
                    Self::object_get_entry(&entries, "fatal").is_some_and(|value| value.truthy());
                let ignore_bom = Self::object_get_entry(&entries, "ignoreBOM")
                    .is_some_and(|value| value.truthy());
                Ok((fatal, ignore_bom))
            }
            _ => Err(Error::ScriptRuntime(
                "TextDecoder constructor options must be an object".into(),
            )),
        }
    }

    pub(crate) fn validate_text_decoder_decode_options(options: Option<&Value>) -> Result<()> {
        let Some(options) = options else {
            return Ok(());
        };
        match options {
            Value::Undefined | Value::Null => Ok(()),
            Value::Object(_) => Ok(()),
            _ => Err(Error::ScriptRuntime(
                "TextDecoder.decode options must be an object".into(),
            )),
        }
    }

    pub(crate) fn text_decoder_input_bytes(&self, input: Option<&Value>) -> Result<Vec<u8>> {
        let Some(input) = input else {
            return Ok(Vec::new());
        };
        match input {
            Value::Undefined => Ok(Vec::new()),
            Value::TypedArray(array) => Ok(self.typed_array_raw_bytes(array)),
            Value::ArrayBuffer(buffer) => Ok(buffer.borrow().bytes.clone()),
            _ => Err(Error::ScriptRuntime(
                "TextDecoder.decode input must be an ArrayBuffer or typed array".into(),
            )),
        }
    }

    pub(crate) fn decode_utf8_bytes(
        bytes: &[u8],
        fatal: bool,
        ignore_bom: bool,
    ) -> Result<String> {
        let mut text = if fatal {
            std::str::from_utf8(bytes)
                .map_err(|_| Error::ScriptRuntime("TextDecoder.decode invalid UTF-8 input".into()))?
                .to_string()
        } else {
            String::from_utf8_lossy(bytes).into_owned()
        };
        if !ignore_bom && text.starts_with('\u{FEFF}') {
            text.remove(0);
        }
        Ok(text)
    }

    pub(crate) fn decode_windows_1251_bytes(bytes: &[u8], ignore_bom: bool) -> String {
        let mut out = String::with_capacity(bytes.len());
        for byte in bytes {
            let ch = match *byte {
                0x00..=0x7F => char::from(*byte),
                0x80 => '\u{0402}',
                0x81 => '\u{0403}',
                0x82 => '\u{201A}',
                0x83 => '\u{0453}',
                0x84 => '\u{201E}',
                0x85 => '\u{2026}',
                0x86 => '\u{2020}',
                0x87 => '\u{2021}',
                0x88 => '\u{20AC}',
                0x89 => '\u{2030}',
                0x8A => '\u{0409}',
                0x8B => '\u{2039}',
                0x8C => '\u{040A}',
                0x8D => '\u{040C}',
                0x8E => '\u{040B}',
                0x8F => '\u{040F}',
                0x90 => '\u{0452}',
                0x91 => '\u{2018}',
                0x92 => '\u{2019}',
                0x93 => '\u{201C}',
                0x94 => '\u{201D}',
                0x95 => '\u{2022}',
                0x96 => '\u{2013}',
                0x97 => '\u{2014}',
                0x98 => '\u{0098}',
                0x99 => '\u{2122}',
                0x9A => '\u{0459}',
                0x9B => '\u{203A}',
                0x9C => '\u{045A}',
                0x9D => '\u{045C}',
                0x9E => '\u{045B}',
                0x9F => '\u{045F}',
                0xA0 => '\u{00A0}',
                0xA1 => '\u{040E}',
                0xA2 => '\u{045E}',
                0xA3 => '\u{0408}',
                0xA4 => '\u{00A4}',
                0xA5 => '\u{0490}',
                0xA6 => '\u{00A6}',
                0xA7 => '\u{00A7}',
                0xA8 => '\u{0401}',
                0xA9 => '\u{00A9}',
                0xAA => '\u{0404}',
                0xAB => '\u{00AB}',
                0xAC => '\u{00AC}',
                0xAD => '\u{00AD}',
                0xAE => '\u{00AE}',
                0xAF => '\u{0407}',
                0xB0 => '\u{00B0}',
                0xB1 => '\u{00B1}',
                0xB2 => '\u{0406}',
                0xB3 => '\u{0456}',
                0xB4 => '\u{0491}',
                0xB5 => '\u{00B5}',
                0xB6 => '\u{00B6}',
                0xB7 => '\u{00B7}',
                0xB8 => '\u{0451}',
                0xB9 => '\u{2116}',
                0xBA => '\u{0454}',
                0xBB => '\u{00BB}',
                0xBC => '\u{0458}',
                0xBD => '\u{0405}',
                0xBE => '\u{0455}',
                0xBF => '\u{0457}',
                0xC0..=0xDF => {
                    char::from_u32(u32::from(*byte) - 0xC0 + 0x0410).unwrap_or('\u{FFFD}')
                }
                0xE0..=0xFF => {
                    char::from_u32(u32::from(*byte) - 0xE0 + 0x0430).unwrap_or('\u{FFFD}')
                }
            };
            out.push(ch);
        }
        if !ignore_bom && out.starts_with('\u{FEFF}') {
            out.remove(0);
        }
        out
    }

    pub(crate) fn decode_text_decoder_bytes(
        encoding: &str,
        bytes: &[u8],
        fatal: bool,
        ignore_bom: bool,
    ) -> Result<String> {
        match encoding {
            "utf-8" => Self::decode_utf8_bytes(bytes, fatal, ignore_bom),
            "windows-1251" => Ok(Self::decode_windows_1251_bytes(bytes, ignore_bom)),
            _ => Err(Error::ScriptRuntime(format!(
                "TextDecoder.decode unsupported encoding: {encoding}"
            ))),
        }
    }
}
