use super::*;

impl Harness {
    pub(crate) fn new_date_value(timestamp_ms: i64) -> Value {
        Value::Date(Rc::new(RefCell::new(timestamp_ms)))
    }

    pub(crate) fn resolve_date_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<i64>>> {
        match env.get(target) {
            Some(Value::Date(value)) => Ok(value.clone()),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a Date",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(crate) fn coerce_date_timestamp_ms(&self, value: &Value) -> i64 {
        match value {
            Value::Date(value) => *value.borrow(),
            Value::String(value) => Self::parse_date_string_to_epoch_ms(value).unwrap_or(0),
            _ => Self::value_to_i64(value),
        }
    }

    pub(crate) fn resolve_array_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<Vec<Value>>>> {
        match env.get(target) {
            Some(Value::Array(values)) => Ok(values.clone()),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not an array",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(crate) fn resolve_array_buffer_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<ArrayBufferValue>>> {
        match env.get(target) {
            Some(Value::ArrayBuffer(buffer)) => Ok(buffer.clone()),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not an ArrayBuffer",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(crate) fn resolve_typed_array_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<TypedArrayValue>>> {
        match env.get(target) {
            Some(Value::TypedArray(array)) => Ok(array.clone()),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a TypedArray",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(crate) fn normalize_blob_type(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.as_bytes().iter().all(|b| (0x20..=0x7e).contains(b)) {
            trimmed.to_ascii_lowercase()
        } else {
            String::new()
        }
    }

    pub(crate) fn new_blob_value(bytes: Vec<u8>, mime_type: String) -> Value {
        Value::Blob(Rc::new(RefCell::new(BlobValue { bytes, mime_type })))
    }

    pub(crate) fn new_readable_stream_placeholder_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_READABLE_STREAM_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )])
    }

    pub(crate) fn new_uint8_typed_array_from_bytes(bytes: &[u8]) -> Value {
        let buffer = Rc::new(RefCell::new(ArrayBufferValue {
            bytes: bytes.to_vec(),
            max_byte_length: None,
            detached: false,
        }));
        Value::TypedArray(Rc::new(RefCell::new(TypedArrayValue {
            kind: TypedArrayKind::Uint8,
            buffer,
            byte_offset: 0,
            fixed_length: Some(bytes.len()),
        })))
    }

    pub(crate) fn typed_array_raw_bytes(&self, array: &Rc<RefCell<TypedArrayValue>>) -> Vec<u8> {
        let (buffer, byte_offset, byte_length) = {
            let array = array.borrow();
            (
                array.buffer.clone(),
                array.byte_offset,
                array.observed_byte_length(),
            )
        };
        if byte_length == 0 {
            return Vec::new();
        }
        let buffer = buffer.borrow();
        let start = byte_offset.min(buffer.byte_length());
        let end = start.saturating_add(byte_length).min(buffer.byte_length());
        if end <= start {
            Vec::new()
        } else {
            buffer.bytes[start..end].to_vec()
        }
    }

    pub(crate) fn blob_part_bytes(&self, part: &Value) -> Vec<u8> {
        match part {
            Value::Blob(blob) => blob.borrow().bytes.clone(),
            Value::ArrayBuffer(buffer) => buffer.borrow().bytes.clone(),
            Value::TypedArray(array) => self.typed_array_raw_bytes(array),
            Value::String(text) => text.as_bytes().to_vec(),
            other => other.as_string().into_bytes(),
        }
    }

    pub(crate) fn eval_blob_construct(
        &mut self,
        parts: &Option<Box<Expr>>,
        options: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Blob constructor must be called with new".into(),
            ));
        }

        let mut bytes = Vec::new();
        if let Some(parts) = parts {
            let parts_value = self.eval_expr(parts, env, event_param, event)?;
            if !matches!(parts_value, Value::Undefined | Value::Null) {
                let items = self
                    .array_like_values_from_value(&parts_value)
                    .map_err(|_| {
                        Error::ScriptRuntime(
                            "Blob constructor first argument must be an array-like or iterable"
                                .into(),
                        )
                    })?;
                for item in items {
                    bytes.extend(self.blob_part_bytes(&item));
                }
            }
        }

        let mut mime_type = String::new();
        if let Some(options) = options {
            let options = self.eval_expr(options, env, event_param, event)?;
            match options {
                Value::Undefined | Value::Null => {}
                Value::Object(entries) => {
                    let entries = entries.borrow();
                    if let Some(value) = Self::object_get_entry(&entries, "type") {
                        mime_type = Self::normalize_blob_type(&value.as_string());
                    }
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "Blob options must be an object".into(),
                    ));
                }
            }
        }

        Ok(Self::new_blob_value(bytes, mime_type))
    }

    pub(crate) fn eval_blob_member_call(
        &mut self,
        blob: &Rc<RefCell<BlobValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "text" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.text does not take arguments".into(),
                    ));
                }
                let text = String::from_utf8_lossy(&blob.borrow().bytes).to_string();
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, Value::String(text))?;
                Ok(Some(Value::Promise(promise)))
            }
            "arrayBuffer" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.arrayBuffer does not take arguments".into(),
                    ));
                }
                let bytes = blob.borrow().bytes.clone();
                let promise = self.new_pending_promise();
                self.promise_resolve(
                    &promise,
                    Value::ArrayBuffer(Rc::new(RefCell::new(ArrayBufferValue {
                        bytes,
                        max_byte_length: None,
                        detached: false,
                    }))),
                )?;
                Ok(Some(Value::Promise(promise)))
            }
            "bytes" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.bytes does not take arguments".into(),
                    ));
                }
                let bytes = blob.borrow().bytes.clone();
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, Self::new_uint8_typed_array_from_bytes(&bytes))?;
                Ok(Some(Value::Promise(promise)))
            }
            "stream" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.stream does not take arguments".into(),
                    ));
                }
                Ok(Some(Self::new_readable_stream_placeholder_value()))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_typed_array_member_call(
        &mut self,
        array: &Rc<RefCell<TypedArrayValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "join" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.join supports at most one argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let separator = if let Some(first) = args.first() {
                    if matches!(first, Value::Undefined) {
                        ",".to_string()
                    } else {
                        first.as_string()
                    }
                } else {
                    ",".to_string()
                };
                let joined = self
                    .typed_array_snapshot(array)?
                    .into_iter()
                    .map(|value| value.as_string())
                    .collect::<Vec<_>>()
                    .join(&separator);
                Ok(Some(Value::String(joined)))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn to_non_negative_usize(value: &Value, label: &str) -> Result<usize> {
        let n = Self::value_to_i64(value);
        if n < 0 {
            return Err(Error::ScriptRuntime(format!(
                "{label} must be a non-negative integer"
            )));
        }
        usize::try_from(n).map_err(|_| Error::ScriptRuntime(format!("{label} is too large")))
    }

    pub(crate) fn eval_call_args_with_spread(
        &mut self,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Vec<Value>> {
        let mut evaluated = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                Expr::Spread(inner) => {
                    let spread_value = self.eval_expr(inner, env, event_param, event)?;
                    evaluated.extend(self.spread_iterable_values_from_value(&spread_value)?);
                }
                _ => evaluated.push(self.eval_expr(arg, env, event_param, event)?),
            }
        }
        Ok(evaluated)
    }

    pub(crate) fn spread_iterable_values_from_value(&self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Array(values) => Ok(values.borrow().clone()),
            Value::TypedArray(values) => self.typed_array_snapshot(values),
            Value::Map(map) => {
                let map = map.borrow();
                Ok(map
                    .entries
                    .iter()
                    .map(|(key, value)| Self::new_array_value(vec![key.clone(), value.clone()]))
                    .collect::<Vec<_>>())
            }
            Value::Set(set) => Ok(set.borrow().values.clone()),
            Value::String(text) => Ok(text
                .chars()
                .map(|ch| Value::String(ch.to_string()))
                .collect::<Vec<_>>()),
            Value::NodeList(nodes) => Ok(nodes.iter().copied().map(Value::Node).collect()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    return Ok(Self::url_search_params_pairs_from_object_entries(&entries)
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>());
                }
                Err(Error::ScriptRuntime("spread source is not iterable".into()))
            }
            _ => Err(Error::ScriptRuntime("spread source is not iterable".into())),
        }
    }

    pub(crate) fn array_like_values_from_value(&self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Array(values) => Ok(values.borrow().clone()),
            Value::TypedArray(values) => self.typed_array_snapshot(values),
            Value::Map(map) => {
                let map = map.borrow();
                Ok(map
                    .entries
                    .iter()
                    .map(|(key, value)| Self::new_array_value(vec![key.clone(), value.clone()]))
                    .collect::<Vec<_>>())
            }
            Value::Set(set) => Ok(set.borrow().values.clone()),
            Value::String(text) => Ok(text
                .chars()
                .map(|ch| Value::String(ch.to_string()))
                .collect::<Vec<_>>()),
            Value::NodeList(nodes) => Ok(nodes.iter().copied().map(Value::Node).collect()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    return Ok(Self::url_search_params_pairs_from_object_entries(&entries)
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>());
                }
                let length_value =
                    Self::object_get_entry(&entries, "length").unwrap_or(Value::Number(0));
                let length = Self::to_non_negative_usize(&length_value, "array-like length")?;
                let mut out = Vec::with_capacity(length);
                for index in 0..length {
                    let key = index.to_string();
                    out.push(Self::object_get_entry(&entries, &key).unwrap_or(Value::Undefined));
                }
                Ok(out)
            }
            _ => Err(Error::ScriptRuntime(
                "expected an array-like or iterable source".into(),
            )),
        }
    }

    pub(crate) fn new_array_buffer_value(
        byte_length: usize,
        max_byte_length: Option<usize>,
    ) -> Value {
        Value::ArrayBuffer(Rc::new(RefCell::new(ArrayBufferValue {
            bytes: vec![0; byte_length],
            max_byte_length,
            detached: false,
        })))
    }

    pub(crate) fn new_typed_array_with_length(
        &mut self,
        kind: TypedArrayKind,
        length: usize,
    ) -> Result<Value> {
        let byte_length = length.saturating_mul(kind.bytes_per_element());
        let buffer = Rc::new(RefCell::new(ArrayBufferValue {
            bytes: vec![0; byte_length],
            max_byte_length: None,
            detached: false,
        }));
        Ok(Value::TypedArray(Rc::new(RefCell::new(TypedArrayValue {
            kind,
            buffer,
            byte_offset: 0,
            fixed_length: Some(length),
        }))))
    }

    pub(crate) fn new_typed_array_view(
        &self,
        kind: TypedArrayKind,
        buffer: Rc<RefCell<ArrayBufferValue>>,
        byte_offset: usize,
        length: Option<usize>,
    ) -> Result<Value> {
        let bytes_per_element = kind.bytes_per_element();
        if byte_offset % bytes_per_element != 0 {
            return Err(Error::ScriptRuntime(format!(
                "start offset of {} should be a multiple of {}",
                kind.name(),
                bytes_per_element
            )));
        }

        let buffer_len = buffer.borrow().byte_length();
        if byte_offset > buffer_len {
            return Err(Error::ScriptRuntime(
                "typed array view bounds are outside the buffer".into(),
            ));
        }

        if let Some(length) = length {
            let required = byte_offset.saturating_add(length.saturating_mul(bytes_per_element));
            if required > buffer_len {
                return Err(Error::ScriptRuntime(
                    "typed array view bounds are outside the buffer".into(),
                ));
            }
        } else {
            let remaining = buffer_len.saturating_sub(byte_offset);
            if remaining % bytes_per_element != 0 {
                return Err(Error::ScriptRuntime(format!(
                    "byte length of {} should be a multiple of {}",
                    kind.name(),
                    bytes_per_element
                )));
            }
        }

        Ok(Value::TypedArray(Rc::new(RefCell::new(TypedArrayValue {
            kind,
            buffer,
            byte_offset,
            fixed_length: length,
        }))))
    }

    pub(crate) fn new_typed_array_from_values(
        &mut self,
        kind: TypedArrayKind,
        values: &[Value],
    ) -> Result<Value> {
        let array = self.new_typed_array_with_length(kind, values.len())?;
        let Value::TypedArray(array) = array else {
            unreachable!();
        };
        for (index, value) in values.iter().enumerate() {
            self.typed_array_set_index(&array, index, value.clone())?;
        }
        Ok(Value::TypedArray(array))
    }

    pub(crate) fn typed_array_snapshot(
        &self,
        array: &Rc<RefCell<TypedArrayValue>>,
    ) -> Result<Vec<Value>> {
        let length = array.borrow().observed_length();
        let mut out = Vec::with_capacity(length);
        for index in 0..length {
            out.push(self.typed_array_get_index(array, index)?);
        }
        Ok(out)
    }

    pub(crate) fn typed_array_get_index(
        &self,
        array: &Rc<RefCell<TypedArrayValue>>,
        index: usize,
    ) -> Result<Value> {
        let (kind, buffer, byte_offset, length) = {
            let array = array.borrow();
            (
                array.kind,
                array.buffer.clone(),
                array.byte_offset,
                array.observed_length(),
            )
        };
        if index >= length {
            return Ok(Value::Undefined);
        }

        let bytes_per_element = kind.bytes_per_element();
        let start = byte_offset.saturating_add(index.saturating_mul(bytes_per_element));
        let buffer = buffer.borrow();
        if start.saturating_add(bytes_per_element) > buffer.byte_length() {
            return Ok(Value::Undefined);
        }
        let bytes = &buffer.bytes[start..start + bytes_per_element];
        let value = match kind {
            TypedArrayKind::Int8 => Value::Number(i64::from(i8::from_le_bytes([bytes[0]]))),
            TypedArrayKind::Uint8 | TypedArrayKind::Uint8Clamped => {
                Value::Number(i64::from(u8::from_le_bytes([bytes[0]])))
            }
            TypedArrayKind::Int16 => {
                Value::Number(i64::from(i16::from_le_bytes([bytes[0], bytes[1]])))
            }
            TypedArrayKind::Uint16 => {
                Value::Number(i64::from(u16::from_le_bytes([bytes[0], bytes[1]])))
            }
            TypedArrayKind::Int32 => Value::Number(i64::from(i32::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
            ]))),
            TypedArrayKind::Uint32 => Value::Number(i64::from(u32::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
            ]))),
            TypedArrayKind::Float16 => {
                let bits = u16::from_le_bytes([bytes[0], bytes[1]]);
                Self::number_value(Self::f16_bits_to_f32(bits) as f64)
            }
            TypedArrayKind::Float32 => Self::number_value(f64::from(f32::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
            ]))),
            TypedArrayKind::Float64 => Self::number_value(f64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ])),
            TypedArrayKind::BigInt64 => Value::BigInt(JsBigInt::from(i64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))),
            TypedArrayKind::BigUint64 => Value::BigInt(JsBigInt::from(u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))),
        };
        Ok(value)
    }

    pub(crate) fn typed_array_number_to_i128(value: f64) -> i128 {
        if !value.is_finite() {
            return 0;
        }
        let value = value.trunc();
        if value >= i128::MAX as f64 {
            i128::MAX
        } else if value <= i128::MIN as f64 {
            i128::MIN
        } else {
            value as i128
        }
    }

    pub(crate) fn typed_array_round_half_even(value: f64) -> f64 {
        let floor = value.floor();
        let frac = value - floor;
        if frac < 0.5 {
            floor
        } else if frac > 0.5 {
            floor + 1.0
        } else if (floor as i64) % 2 == 0 {
            floor
        } else {
            floor + 1.0
        }
    }

    pub(crate) fn typed_array_bytes_for_value(
        kind: TypedArrayKind,
        value: &Value,
    ) -> Result<Vec<u8>> {
        if kind.is_bigint() {
            let Value::BigInt(value) = value else {
                return Err(Error::ScriptRuntime(
                    "Cannot convert number to BigInt typed array element".into(),
                ));
            };
            let modulus = JsBigInt::one() << 64usize;
            let mut unsigned = value % &modulus;
            if unsigned.sign() == Sign::Minus {
                unsigned += &modulus;
            }
            return match kind {
                TypedArrayKind::BigInt64 => {
                    let cutoff = JsBigInt::one() << 63usize;
                    let signed = if unsigned >= cutoff {
                        unsigned - &modulus
                    } else {
                        unsigned
                    };
                    let value = signed.to_i64().unwrap_or(0);
                    Ok(value.to_le_bytes().to_vec())
                }
                TypedArrayKind::BigUint64 => {
                    let value = unsigned.to_u64().unwrap_or(0);
                    Ok(value.to_le_bytes().to_vec())
                }
                _ => unreachable!(),
            };
        }

        if matches!(value, Value::BigInt(_)) {
            return Err(Error::ScriptRuntime(
                "Cannot convert a BigInt value to a number".into(),
            ));
        }

        let number = Self::coerce_number_for_global(value);
        let bytes = match kind {
            TypedArrayKind::Int8 => {
                let modulus = 1i128 << 8;
                let mut out = Self::typed_array_number_to_i128(number).rem_euclid(modulus);
                if out >= (1i128 << 7) {
                    out -= modulus;
                }
                (out as i8).to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint8 => {
                let out = Self::typed_array_number_to_i128(number).rem_euclid(1i128 << 8);
                (out as u8).to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint8Clamped => {
                let clamped = if number.is_nan() {
                    0.0
                } else {
                    number.clamp(0.0, 255.0)
                };
                let rounded = Self::typed_array_round_half_even(clamped);
                (rounded as u8).to_le_bytes().to_vec()
            }
            TypedArrayKind::Int16 => {
                let modulus = 1i128 << 16;
                let mut out = Self::typed_array_number_to_i128(number).rem_euclid(modulus);
                if out >= (1i128 << 15) {
                    out -= modulus;
                }
                (out as i16).to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint16 => {
                let out = Self::typed_array_number_to_i128(number).rem_euclid(1i128 << 16);
                (out as u16).to_le_bytes().to_vec()
            }
            TypedArrayKind::Int32 => {
                let modulus = 1i128 << 32;
                let mut out = Self::typed_array_number_to_i128(number).rem_euclid(modulus);
                if out >= (1i128 << 31) {
                    out -= modulus;
                }
                (out as i32).to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint32 => {
                let out = Self::typed_array_number_to_i128(number).rem_euclid(1i128 << 32);
                (out as u32).to_le_bytes().to_vec()
            }
            TypedArrayKind::Float16 => {
                let rounded = Self::math_f16round(number);
                let bits = Self::f32_to_f16_bits(rounded as f32);
                bits.to_le_bytes().to_vec()
            }
            TypedArrayKind::Float32 => (number as f32).to_le_bytes().to_vec(),
            TypedArrayKind::Float64 => number.to_le_bytes().to_vec(),
            TypedArrayKind::BigInt64 | TypedArrayKind::BigUint64 => unreachable!(),
        };
        Ok(bytes)
    }

    pub(crate) fn typed_array_set_index(
        &mut self,
        array: &Rc<RefCell<TypedArrayValue>>,
        index: usize,
        value: Value,
    ) -> Result<()> {
        let (kind, buffer, byte_offset, length) = {
            let array = array.borrow();
            (
                array.kind,
                array.buffer.clone(),
                array.byte_offset,
                array.observed_length(),
            )
        };
        if index >= length {
            return Ok(());
        }
        let bytes_per_element = kind.bytes_per_element();
        let start = byte_offset.saturating_add(index.saturating_mul(bytes_per_element));
        let bytes = Self::typed_array_bytes_for_value(kind, &value)?;
        if bytes.len() != bytes_per_element {
            return Err(Error::ScriptRuntime(
                "typed array element size mismatch".into(),
            ));
        }
        let mut buffer = buffer.borrow_mut();
        if start.saturating_add(bytes_per_element) > buffer.byte_length() {
            return Ok(());
        }
        buffer.bytes[start..start + bytes_per_element].copy_from_slice(&bytes);
        Ok(())
    }

    pub(crate) fn eval_array_buffer_construct(
        &mut self,
        byte_length: &Option<Box<Expr>>,
        options: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "ArrayBuffer constructor must be called with new".into(),
            ));
        }
        let byte_length = if let Some(byte_length) = byte_length {
            let value = self.eval_expr(byte_length, env, event_param, event)?;
            Self::to_non_negative_usize(&value, "ArrayBuffer byteLength")?
        } else {
            0
        };
        let max_byte_length = if let Some(options) = options {
            let options = self.eval_expr(options, env, event_param, event)?;
            match options {
                Value::Undefined | Value::Null => None,
                Value::Object(entries) => {
                    let entries = entries.borrow();
                    if let Some(value) = Self::object_get_entry(&entries, "maxByteLength") {
                        Some(Self::to_non_negative_usize(
                            &value,
                            "ArrayBuffer maxByteLength",
                        )?)
                    } else {
                        None
                    }
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "ArrayBuffer options must be an object".into(),
                    ));
                }
            }
        } else {
            None
        };
        if max_byte_length.is_some_and(|max| byte_length > max) {
            return Err(Error::ScriptRuntime(
                "ArrayBuffer byteLength exceeds maxByteLength".into(),
            ));
        }
        Ok(Self::new_array_buffer_value(byte_length, max_byte_length))
    }

    pub(crate) fn resize_array_buffer(
        &mut self,
        buffer: &Rc<RefCell<ArrayBufferValue>>,
        new_byte_length: i64,
    ) -> Result<()> {
        Self::ensure_array_buffer_not_detached(buffer, "resize")?;
        if new_byte_length < 0 {
            return Err(Error::ScriptRuntime(
                "ArrayBuffer resize length must be non-negative".into(),
            ));
        }
        let new_byte_length = usize::try_from(new_byte_length)
            .map_err(|_| Error::ScriptRuntime("ArrayBuffer resize length is too large".into()))?;
        let max_byte_length = buffer.borrow().max_byte_length;
        let Some(max_byte_length) = max_byte_length else {
            return Err(Error::ScriptRuntime("ArrayBuffer is not resizable".into()));
        };
        if new_byte_length > max_byte_length {
            return Err(Error::ScriptRuntime(
                "ArrayBuffer resize exceeds maxByteLength".into(),
            ));
        }
        buffer.borrow_mut().bytes.resize(new_byte_length, 0);
        Ok(())
    }

    pub(crate) fn ensure_array_buffer_not_detached(
        buffer: &Rc<RefCell<ArrayBufferValue>>,
        method: &str,
    ) -> Result<()> {
        if buffer.borrow().detached {
            return Err(Error::ScriptRuntime(format!(
                "Cannot perform ArrayBuffer.prototype.{method} on a detached ArrayBuffer"
            )));
        }
        Ok(())
    }

    pub(crate) fn transfer_array_buffer(
        &mut self,
        buffer: &Rc<RefCell<ArrayBufferValue>>,
        to_fixed_length: bool,
    ) -> Result<Value> {
        Self::ensure_array_buffer_not_detached(
            buffer,
            if to_fixed_length {
                "transferToFixedLength"
            } else {
                "transfer"
            },
        )?;
        let mut source = buffer.borrow_mut();
        let bytes = source.bytes.clone();
        let max_byte_length = if to_fixed_length {
            None
        } else {
            source.max_byte_length
        };
        source.bytes.clear();
        source.max_byte_length = None;
        source.detached = true;
        drop(source);
        Ok(Value::ArrayBuffer(Rc::new(RefCell::new(
            ArrayBufferValue {
                bytes,
                max_byte_length,
                detached: false,
            },
        ))))
    }

    pub(crate) fn resize_array_buffer_in_env(
        &mut self,
        env: &HashMap<String, Value>,
        target: &str,
        new_byte_length: i64,
    ) -> Result<()> {
        let buffer = self.resolve_array_buffer_from_env(env, target)?;
        self.resize_array_buffer(&buffer, new_byte_length)
    }

    pub(crate) fn eval_typed_array_construct(
        &mut self,
        kind: TypedArrayKind,
        args: &[Expr],
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(format!(
                "{} constructor must be called with new",
                kind.name()
            )));
        }
        if args.len() > 3 {
            return Err(Error::ScriptRuntime(format!(
                "{} supports up to three arguments",
                kind.name()
            )));
        }

        if args.is_empty() {
            return self.new_typed_array_with_length(kind, 0);
        }

        let first = self.eval_expr(&args[0], env, event_param, event)?;
        match (&first, args.len()) {
            (Value::ArrayBuffer(buffer), 1) => {
                self.new_typed_array_view(kind, buffer.clone(), 0, None)
            }
            (Value::TypedArray(source), 1) => {
                let source_kind = source.borrow().kind;
                if kind.is_bigint() != source_kind.is_bigint() {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and Number typed arrays".into(),
                    ));
                }
                let values = self.typed_array_snapshot(source)?;
                self.new_typed_array_from_values(kind, &values)
            }
            (Value::Array(_), 1) | (Value::Object(_), 1) | (Value::String(_), 1) => {
                let values = self.array_like_values_from_value(&first)?;
                self.new_typed_array_from_values(kind, &values)
            }
            (Value::ArrayBuffer(buffer), _) => {
                let byte_offset = if args.len() >= 2 {
                    let offset = self.eval_expr(&args[1], env, event_param, event)?;
                    Self::to_non_negative_usize(&offset, "typed array byteOffset")?
                } else {
                    0
                };
                let length = if args.len() == 3 {
                    let length = self.eval_expr(&args[2], env, event_param, event)?;
                    if matches!(length, Value::Undefined) {
                        None
                    } else {
                        Some(Self::to_non_negative_usize(&length, "typed array length")?)
                    }
                } else {
                    None
                };
                self.new_typed_array_view(kind, buffer.clone(), byte_offset, length)
            }
            _ => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "typed array buffer view requires an ArrayBuffer first argument".into(),
                    ));
                }
                let length = Self::to_non_negative_usize(&first, "typed array length")?;
                self.new_typed_array_with_length(kind, length)
            }
        }
    }

    pub(crate) fn eval_typed_array_construct_with_callee(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "constructor must be called with new".into(),
            ));
        }
        let callee = self.eval_expr(callee, env, event_param, event)?;
        match callee {
            Value::TypedArrayConstructor(TypedArrayConstructorKind::Concrete(kind)) => {
                self.eval_typed_array_construct(kind, args, true, env, event_param, event)
            }
            Value::TypedArrayConstructor(TypedArrayConstructorKind::Abstract) => Err(
                Error::ScriptRuntime("Abstract class TypedArray not directly constructable".into()),
            ),
            _ => Err(Error::ScriptRuntime("value is not a constructor".into())),
        }
    }

    pub(crate) fn eval_typed_array_static_method(
        &mut self,
        kind: TypedArrayKind,
        method: TypedArrayStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            TypedArrayStaticMethod::From => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(format!(
                        "{}.from requires exactly one argument",
                        kind.name()
                    )));
                }
                let source = self.eval_expr(&args[0], env, event_param, event)?;
                if let Value::TypedArray(source_array) = &source {
                    if kind.is_bigint() != source_array.borrow().kind.is_bigint() {
                        return Err(Error::ScriptRuntime(
                            "cannot mix BigInt and Number typed arrays".into(),
                        ));
                    }
                }
                let values = self.array_like_values_from_value(&source)?;
                self.new_typed_array_from_values(kind, &values)
            }
            TypedArrayStaticMethod::Of => {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.eval_expr(arg, env, event_param, event)?);
                }
                self.new_typed_array_from_values(kind, &values)
            }
        }
    }

    pub(crate) fn same_value_zero(&self, left: &Value, right: &Value) -> bool {
        if let (Some(left_num), Some(right_num)) = (
            Self::number_primitive_value(left),
            Self::number_primitive_value(right),
        ) {
            if left_num.is_nan() && right_num.is_nan() {
                return true;
            }
        }
        self.strict_equal(left, right)
    }

    pub(crate) fn map_entry_index(&self, map: &MapValue, key: &Value) -> Option<usize> {
        map.entries
            .iter()
            .position(|(existing_key, _)| self.same_value_zero(existing_key, key))
    }

    pub(crate) fn map_set_entry(&self, map: &mut MapValue, key: Value, value: Value) {
        if let Some(index) = self.map_entry_index(map, &key) {
            map.entries[index].1 = value;
        } else {
            map.entries.push((key, value));
        }
    }

    pub(crate) fn map_entries_array(&self, map: &Rc<RefCell<MapValue>>) -> Vec<Value> {
        map.borrow()
            .entries
            .iter()
            .map(|(key, value)| Self::new_array_value(vec![key.clone(), value.clone()]))
            .collect::<Vec<_>>()
    }

    pub(crate) fn is_map_method_name(name: &str) -> bool {
        matches!(
            name,
            "set"
                | "get"
                | "has"
                | "delete"
                | "clear"
                | "forEach"
                | "entries"
                | "keys"
                | "values"
                | "getOrInsert"
                | "getOrInsertComputed"
        )
    }

    pub(crate) fn set_value_index(&self, set: &SetValue, value: &Value) -> Option<usize> {
        set.values
            .iter()
            .position(|existing_value| self.same_value_zero(existing_value, value))
    }

    pub(crate) fn set_add_value(&self, set: &mut SetValue, value: Value) {
        if self.set_value_index(set, &value).is_none() {
            set.values.push(value);
        }
    }

    pub(crate) fn set_values_array(&self, set: &Rc<RefCell<SetValue>>) -> Vec<Value> {
        set.borrow().values.clone()
    }

    pub(crate) fn set_entries_array(&self, set: &Rc<RefCell<SetValue>>) -> Vec<Value> {
        set.borrow()
            .values
            .iter()
            .map(|value| Self::new_array_value(vec![value.clone(), value.clone()]))
            .collect::<Vec<_>>()
    }

    pub(crate) fn set_like_keys_snapshot(&self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Set(set) => Ok(set.borrow().values.clone()),
            Value::Map(map) => Ok(map
                .borrow()
                .entries
                .iter()
                .map(|(key, _)| key.clone())
                .collect::<Vec<_>>()),
            _ => Err(Error::ScriptRuntime(
                "Set composition argument must be set-like (Set or Map)".into(),
            )),
        }
    }

    pub(crate) fn set_like_has_value(&self, value: &Value, candidate: &Value) -> Result<bool> {
        match value {
            Value::Set(set) => Ok(self.set_value_index(&set.borrow(), candidate).is_some()),
            Value::Map(map) => Ok(self.map_entry_index(&map.borrow(), candidate).is_some()),
            _ => Err(Error::ScriptRuntime(
                "Set composition argument must be set-like (Set or Map)".into(),
            )),
        }
    }

    pub(crate) fn is_url_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_URL_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_url_static_method_name(name: &str) -> bool {
        matches!(
            name,
            "canParse" | "parse" | "createObjectURL" | "revokeObjectURL"
        )
    }

    pub(crate) fn normalize_url_parts_for_serialization(parts: &mut LocationParts) {
        parts.scheme = parts.scheme.to_ascii_lowercase();
        if parts.has_authority {
            parts.hostname = parts.hostname.to_ascii_lowercase();
            let path = if parts.pathname.is_empty() {
                "/".to_string()
            } else if parts.pathname.starts_with('/') {
                parts.pathname.clone()
            } else {
                format!("/{}", parts.pathname)
            };
            parts.pathname = normalize_pathname(&path);
            parts.pathname = encode_uri_like_preserving_percent(&parts.pathname, false);
        } else {
            parts.opaque_path = encode_uri_like_preserving_percent(&parts.opaque_path, false);
        }

        if !parts.search.is_empty() {
            let body = parts
                .search
                .strip_prefix('?')
                .unwrap_or(parts.search.as_str());
            parts.search = format!("?{}", encode_uri_like_preserving_percent(body, false));
        }
        if !parts.hash.is_empty() {
            let body = parts.hash.strip_prefix('#').unwrap_or(parts.hash.as_str());
            parts.hash = format!("#{}", encode_uri_like_preserving_percent(body, false));
        }
    }

    pub(crate) fn resolve_url_against_base_parts(input: &str, base: &LocationParts) -> String {
        let input = input.trim();
        if input.is_empty() {
            return base.href();
        }

        if input.starts_with("//") {
            return LocationParts::parse(&format!("{}{}", base.protocol(), input))
                .map(|parts| parts.href())
                .unwrap_or_else(|| input.to_string());
        }

        let mut next = base.clone();
        if input.starts_with('#') {
            next.hash = ensure_hash_prefix(input);
            return next.href();
        }

        if input.starts_with('?') {
            next.search = ensure_search_prefix(input);
            next.hash.clear();
            return next.href();
        }

        if input.starts_with('/') {
            if next.has_authority {
                next.pathname = normalize_pathname(input);
            } else {
                next.opaque_path = input.to_string();
            }
            next.search.clear();
            next.hash.clear();
            return next.href();
        }

        let mut relative = input;
        let mut next_search = String::new();
        let mut next_hash = String::new();
        if let Some(hash_pos) = relative.find('#') {
            next_hash = ensure_hash_prefix(&relative[hash_pos + 1..]);
            relative = &relative[..hash_pos];
        }
        if let Some(search_pos) = relative.find('?') {
            next_search = ensure_search_prefix(&relative[search_pos + 1..]);
            relative = &relative[..search_pos];
        }

        if next.has_authority {
            let base_dir = if let Some((prefix, _)) = next.pathname.rsplit_once('/') {
                if prefix.is_empty() {
                    "/".to_string()
                } else {
                    format!("{prefix}/")
                }
            } else {
                "/".to_string()
            };
            next.pathname = normalize_pathname(&format!("{base_dir}{relative}"));
        } else {
            next.opaque_path = relative.to_string();
        }
        next.search = next_search;
        next.hash = next_hash;
        next.href()
    }

    pub(crate) fn resolve_url_string(input: &str, base: Option<&str>) -> Option<String> {
        let input = input.trim();
        if let Some(mut absolute) = LocationParts::parse(input) {
            Self::normalize_url_parts_for_serialization(&mut absolute);
            return Some(absolute.href());
        }

        let base = base?;
        let mut base_parts = LocationParts::parse(base)?;
        Self::normalize_url_parts_for_serialization(&mut base_parts);
        let resolved = Self::resolve_url_against_base_parts(input, &base_parts);
        let mut resolved_parts = LocationParts::parse(&resolved)?;
        Self::normalize_url_parts_for_serialization(&mut resolved_parts);
        Some(resolved_parts.href())
    }

    pub(crate) fn sync_url_object_entries_from_parts(
        &self,
        entries: &mut (impl ObjectEntryLookup + ObjectEntryMut),
        parts: &LocationParts,
    ) {
        let href = parts.href();
        Self::object_set_entry(
            entries,
            INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
            Value::String(href.clone()),
        );
        Self::object_set_entry(entries, "href".to_string(), Value::String(href));
        Self::object_set_entry(
            entries,
            "protocol".to_string(),
            Value::String(parts.protocol()),
        );
        Self::object_set_entry(entries, "host".to_string(), Value::String(parts.host()));
        Self::object_set_entry(
            entries,
            "hostname".to_string(),
            Value::String(parts.hostname.clone()),
        );
        Self::object_set_entry(
            entries,
            "port".to_string(),
            Value::String(parts.port.clone()),
        );
        Self::object_set_entry(
            entries,
            "pathname".to_string(),
            Value::String(if parts.has_authority {
                parts.pathname.clone()
            } else {
                parts.opaque_path.clone()
            }),
        );
        Self::object_set_entry(
            entries,
            "search".to_string(),
            Value::String(parts.search.clone()),
        );
        Self::object_set_entry(
            entries,
            "hash".to_string(),
            Value::String(parts.hash.clone()),
        );
        Self::object_set_entry(
            entries,
            "username".to_string(),
            Value::String(parts.username.clone()),
        );
        Self::object_set_entry(
            entries,
            "password".to_string(),
            Value::String(parts.password.clone()),
        );
        Self::object_set_entry(entries, "origin".to_string(), Value::String(parts.origin()));

        let owner_id = match Self::object_get_entry(entries, INTERNAL_URL_OBJECT_ID_KEY) {
            Some(Value::Number(id)) if id >= 0 => usize::try_from(id).ok(),
            _ => None,
        };
        let pairs =
            parse_url_search_params_pairs_from_query_string(&parts.search).unwrap_or_default();
        if let Some(Value::Object(search_params_object)) =
            Self::object_get_entry(entries, "searchParams")
        {
            let mut search_params_entries = search_params_object.borrow_mut();
            Self::set_url_search_params_pairs(&mut search_params_entries, &pairs);
            if let Some(owner_id) = owner_id {
                Self::object_set_entry(
                    &mut search_params_entries,
                    INTERNAL_URL_SEARCH_PARAMS_OWNER_ID_KEY.to_string(),
                    Value::Number(owner_id as i64),
                );
            }
        } else {
            Self::object_set_entry(
                entries,
                "searchParams".to_string(),
                self.new_url_search_params_value(pairs, owner_id),
            );
        }
    }

    pub(crate) fn new_url_value_from_href(&mut self, href: &str) -> Result<Value> {
        let mut parts =
            LocationParts::parse(href).ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
        Self::normalize_url_parts_for_serialization(&mut parts);
        let id = self.browser_apis.allocate_url_object_id();

        let mut entries = vec![
            (INTERNAL_URL_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_URL_OBJECT_ID_KEY.to_string(),
                Value::Number(id as i64),
            ),
        ];
        self.sync_url_object_entries_from_parts(&mut entries, &parts);
        let object = Rc::new(RefCell::new(ObjectValue::new(entries)));
        self.browser_apis.url_objects.insert(id, object.clone());
        Ok(Value::Object(object))
    }

    pub(crate) fn set_url_object_property(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        key: &str,
        value: Value,
    ) -> Result<()> {
        if matches!(key, "origin" | "searchParams") {
            return Err(Error::ScriptRuntime(format!("URL.{key} is read-only")));
        }

        let current_href = {
            let entries = object.borrow();
            Self::object_get_entry(&entries, "href")
                .map(|value| value.as_string())
                .unwrap_or_default()
        };
        let mut parts = LocationParts::parse(&current_href)
            .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
        match key {
            "href" => {
                let href = Self::resolve_url_string(&value.as_string(), None)
                    .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
                parts = LocationParts::parse(&href)
                    .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
            }
            "protocol" => {
                let protocol = value.as_string();
                let protocol = protocol.trim_end_matches(':').to_ascii_lowercase();
                if !is_valid_url_scheme(&protocol) {
                    return Err(Error::ScriptRuntime(format!(
                        "invalid URL.protocol value: {}",
                        value.as_string()
                    )));
                }
                parts.scheme = protocol;
            }
            "host" => {
                let host = value.as_string();
                let (hostname, port) = split_hostname_and_port(host.trim());
                parts.hostname = hostname;
                parts.port = port;
            }
            "hostname" => {
                parts.hostname = value.as_string();
            }
            "port" => {
                parts.port = value.as_string();
            }
            "pathname" => {
                let raw = value.as_string();
                if parts.has_authority {
                    let normalized_input = if raw.starts_with('/') {
                        raw
                    } else {
                        format!("/{raw}")
                    };
                    parts.pathname = normalize_pathname(&normalized_input);
                } else {
                    parts.opaque_path = raw;
                }
            }
            "search" => {
                parts.search = ensure_search_prefix(&value.as_string());
            }
            "hash" => {
                parts.hash = ensure_hash_prefix(&value.as_string());
            }
            "username" => {
                parts.username = value.as_string();
            }
            "password" => {
                parts.password = value.as_string();
            }
            _ => {
                Self::object_set_entry(&mut object.borrow_mut(), key.to_string(), value);
                return Ok(());
            }
        }

        Self::normalize_url_parts_for_serialization(&mut parts);
        self.sync_url_object_entries_from_parts(&mut object.borrow_mut(), &parts);
        Ok(())
    }

    pub(crate) fn sync_url_search_params_owner(&mut self, object: &Rc<RefCell<ObjectValue>>) {
        let (owner_id, pairs) = {
            let entries = object.borrow();
            let owner_id =
                match Self::object_get_entry(&entries, INTERNAL_URL_SEARCH_PARAMS_OWNER_ID_KEY) {
                    Some(Value::Number(id)) if id > 0 => usize::try_from(id).ok(),
                    _ => None,
                };
            let pairs = Self::url_search_params_pairs_from_object_entries(&entries);
            (owner_id, pairs)
        };
        let Some(owner_id) = owner_id else {
            return;
        };
        let Some(url_object) = self.browser_apis.url_objects.get(&owner_id).cloned() else {
            return;
        };

        let current_href = {
            let entries = url_object.borrow();
            Self::object_get_entry(&entries, "href")
                .map(|value| value.as_string())
                .unwrap_or_default()
        };
        let Some(mut parts) = LocationParts::parse(&current_href) else {
            return;
        };

        let serialized = serialize_url_search_params_pairs(&pairs);
        parts.search = if serialized.is_empty() {
            String::new()
        } else {
            format!("?{serialized}")
        };
        Self::normalize_url_parts_for_serialization(&mut parts);
        self.sync_url_object_entries_from_parts(&mut url_object.borrow_mut(), &parts);
    }
}
