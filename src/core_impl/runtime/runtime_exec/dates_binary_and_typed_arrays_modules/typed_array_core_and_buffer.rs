impl Harness {
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

}
