impl Harness {
    fn eval_expr_regex_numbers_and_builtins(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
            Expr::RegexLiteral { pattern, flags } => {
                Self::new_regex_value(pattern.clone(), flags.clone())
            }
            Expr::RegexNew { pattern, flags } => {
                let pattern = self.eval_expr(pattern, env, event_param, event)?;
                let flags = flags
                    .as_ref()
                    .map(|flags| self.eval_expr(flags, env, event_param, event))
                    .transpose()?;
                Self::new_regex_from_values(&pattern, flags.as_ref())
            }
            Expr::RegExpConstructor => Ok(Value::RegExpConstructor),
            Expr::RegExpStaticMethod { method, args } => {
                self.eval_regexp_static_method(*method, args, env, event_param, event)
            }
            Expr::RegexTest { regex, input } => {
                let regex = self.eval_expr(regex, env, event_param, event)?;
                let input = self.eval_expr(input, env, event_param, event)?.as_string();
                let regex = Self::resolve_regex_from_value(&regex)?;
                Ok(Value::Bool(Self::regex_test(&regex, &input)?))
            }
            Expr::RegexExec { regex, input } => {
                let regex = self.eval_expr(regex, env, event_param, event)?;
                let input = self.eval_expr(input, env, event_param, event)?.as_string();
                let regex = Self::resolve_regex_from_value(&regex)?;
                let Some(captures) = Self::regex_exec(&regex, &input)? else {
                    return Ok(Value::Null);
                };
                Ok(Self::new_array_value(
                    captures.into_iter().map(Value::String).collect::<Vec<_>>(),
                ))
            }
            Expr::RegexToString { regex } => {
                let value = self.eval_expr(regex, env, event_param, event)?;
                if let Ok(regex) = Self::resolve_regex_from_value(&value) {
                    let regex = regex.borrow();
                    Ok(Value::String(format!("/{}/{}", regex.source, regex.flags)))
                } else if let Ok(locale_data) = self.resolve_intl_locale_data(&value) {
                    Ok(Value::String(Self::intl_locale_data_to_string(
                        &locale_data,
                    )))
                } else {
                    Ok(Value::String(value.as_string()))
                }
            }
            Expr::MathConst(constant) => match constant {
                MathConst::E => Ok(Value::Float(std::f64::consts::E)),
                MathConst::Ln10 => Ok(Value::Float(std::f64::consts::LN_10)),
                MathConst::Ln2 => Ok(Value::Float(std::f64::consts::LN_2)),
                MathConst::Log10E => Ok(Value::Float(std::f64::consts::LOG10_E)),
                MathConst::Log2E => Ok(Value::Float(std::f64::consts::LOG2_E)),
                MathConst::Pi => Ok(Value::Float(std::f64::consts::PI)),
                MathConst::Sqrt1_2 => Ok(Value::Float(std::f64::consts::FRAC_1_SQRT_2)),
                MathConst::Sqrt2 => Ok(Value::Float(std::f64::consts::SQRT_2)),
                MathConst::ToStringTag => Ok(Value::String("Math".to_string())),
            },
            Expr::MathMethod { method, args } => {
                self.eval_math_method(*method, args, env, event_param, event)
            }
            Expr::StringConstruct {
                value,
                called_with_new,
            } => {
                let value = value
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .unwrap_or(Value::Undefined);
                let coerced = value.as_string();
                if *called_with_new {
                    Ok(Self::new_string_wrapper_value(coerced))
                } else {
                    Ok(Value::String(coerced))
                }
            }
            Expr::StringStaticMethod { method, args } => {
                self.eval_string_static_method(*method, args, env, event_param, event)
            }
            Expr::StringConstructor => Ok(Value::StringConstructor),
            Expr::NumberConstruct { value } => {
                let value = value
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .unwrap_or(Value::Number(0));
                Ok(Self::number_value(
                    Self::coerce_number_for_number_constructor(&value),
                ))
            }
            Expr::NumberConst(constant) => match constant {
                NumberConst::Epsilon => Ok(Value::Float(f64::EPSILON)),
                NumberConst::MaxSafeInteger => Ok(Value::Number(9_007_199_254_740_991)),
                NumberConst::MaxValue => Ok(Value::Float(f64::MAX)),
                NumberConst::MinSafeInteger => Ok(Value::Number(-9_007_199_254_740_991)),
                NumberConst::MinValue => Ok(Value::Float(f64::from_bits(1))),
                NumberConst::NaN => Ok(Value::Float(f64::NAN)),
                NumberConst::NegativeInfinity => Ok(Value::Float(f64::NEG_INFINITY)),
                NumberConst::PositiveInfinity => Ok(Value::Float(f64::INFINITY)),
            },
            Expr::NumberMethod { method, args } => {
                self.eval_number_method(*method, args, env, event_param, event)
            }
            Expr::NumberInstanceMethod {
                value,
                method,
                args,
            } => self.eval_number_instance_method(*method, value, args, env, event_param, event),
            Expr::BigIntConstruct {
                value,
                called_with_new,
            } => {
                if *called_with_new {
                    return Err(Error::ScriptRuntime("BigInt is not a constructor".into()));
                }
                let value = value
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .unwrap_or(Value::Undefined);
                Ok(Value::BigInt(Self::coerce_bigint_for_constructor(&value)?))
            }
            Expr::BigIntMethod { method, args } => {
                self.eval_bigint_method(*method, args, env, event_param, event)
            }
            Expr::BigIntInstanceMethod {
                value,
                method,
                args,
            } => self.eval_bigint_instance_method(*method, value, args, env, event_param, event),
            Expr::BlobConstruct {
                parts,
                options,
                called_with_new,
            } => {
                self.eval_blob_construct(parts, options, *called_with_new, env, event_param, event)
            }
            Expr::BlobConstructor => Ok(Value::BlobConstructor),
            Expr::UrlConstruct {
                input,
                base,
                called_with_new,
            } => self.eval_url_construct(input, base, *called_with_new, env, event_param, event),
            Expr::UrlConstructor => Ok(Value::UrlConstructor),
            Expr::UrlStaticMethod { method, args } => {
                self.eval_url_static_method(*method, args, env, event_param, event)
            }
            Expr::ArrayBufferConstruct {
                byte_length,
                options,
                called_with_new,
            } => self.eval_array_buffer_construct(
                byte_length,
                options,
                *called_with_new,
                env,
                event_param,
                event,
            ),
            Expr::ArrayBufferConstructor => Ok(Value::ArrayBufferConstructor),
            Expr::ArrayBufferIsView(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Bool(matches!(value, Value::TypedArray(_))))
            }
            Expr::ArrayBufferDetached(target) => {
                let buffer = self.resolve_array_buffer_from_env(env, target)?;
                Ok(Value::Bool(buffer.borrow().detached))
            }
            Expr::ArrayBufferMaxByteLength(target) => {
                let buffer = self.resolve_array_buffer_from_env(env, target)?;
                Ok(Value::Number(buffer.borrow().max_byte_length() as i64))
            }
            Expr::ArrayBufferResizable(target) => {
                let buffer = self.resolve_array_buffer_from_env(env, target)?;
                Ok(Value::Bool(buffer.borrow().resizable()))
            }
            Expr::ArrayBufferResize {
                target,
                new_byte_length,
            } => {
                let new_byte_length = self.eval_expr(new_byte_length, env, event_param, event)?;
                let new_byte_length = Self::value_to_i64(&new_byte_length);
                self.resize_array_buffer_in_env(env, target, new_byte_length)?;
                Ok(Value::Undefined)
            }
            Expr::ArrayBufferSlice { target, start, end } => {
                let buffer = self.resolve_array_buffer_from_env(env, target)?;
                Self::ensure_array_buffer_not_detached(&buffer, "slice")?;
                let source = buffer.borrow();
                let len = source.bytes.len();
                let start = if let Some(start) = start {
                    let start = self.eval_expr(start, env, event_param, event)?;
                    Self::normalize_slice_index(len, Self::value_to_i64(&start))
                } else {
                    0
                };
                let end = if let Some(end) = end {
                    let end = self.eval_expr(end, env, event_param, event)?;
                    Self::normalize_slice_index(len, Self::value_to_i64(&end))
                } else {
                    len
                };
                let end = end.max(start);
                let bytes = source.bytes[start..end].to_vec();
                Ok(Value::ArrayBuffer(Rc::new(RefCell::new(
                    ArrayBufferValue {
                        bytes,
                        max_byte_length: None,
                        detached: false,
                    },
                ))))
            }
            Expr::ArrayBufferTransfer {
                target,
                to_fixed_length,
            } => {
                let buffer = self.resolve_array_buffer_from_env(env, target)?;
                self.transfer_array_buffer(&buffer, *to_fixed_length)
            }
            Expr::TypedArrayConstructorRef(kind) => Ok(Value::TypedArrayConstructor(kind.clone())),
            Expr::TypedArrayConstruct {
                kind,
                args,
                called_with_new,
            } => self.eval_typed_array_construct(
                *kind,
                args,
                *called_with_new,
                env,
                event_param,
                event,
            ),
            Expr::TypedArrayConstructWithCallee {
                callee,
                args,
                called_with_new,
            } => self.eval_typed_array_construct_with_callee(
                callee,
                args,
                *called_with_new,
                env,
                event_param,
                event,
            ),
            Expr::PromiseConstruct {
                executor,
                called_with_new,
            } => self.eval_promise_construct(executor, *called_with_new, env, event_param, event),
            Expr::PromiseConstructor => Ok(Value::PromiseConstructor),
            Expr::PromiseStaticMethod { method, args } => {
                self.eval_promise_static_method(*method, args, env, event_param, event)
            }
            Expr::PromiseMethod {
                target,
                method,
                args,
            } => self.eval_promise_method(target, *method, args, env, event_param, event),
            Expr::MapConstruct {
                iterable,
                called_with_new,
            } => self.eval_map_construct(iterable, *called_with_new, env, event_param, event),
            Expr::MapConstructor => Ok(Value::MapConstructor),
            Expr::MapStaticMethod { method, args } => {
                self.eval_map_static_method(*method, args, env, event_param, event)
            }
            Expr::MapMethod {
                target,
                method,
                args,
            } => self.eval_map_method(target, *method, args, env, event_param, event),
            Expr::UrlSearchParamsConstruct {
                init,
                called_with_new,
            } => self.eval_url_search_params_construct(
                init,
                *called_with_new,
                env,
                event_param,
                event,
            ),
            Expr::UrlSearchParamsMethod {
                target,
                method,
                args,
            } => self.eval_url_search_params_method(target, *method, args, env, event_param, event),
            Expr::SetConstruct {
                iterable,
                called_with_new,
            } => self.eval_set_construct(iterable, *called_with_new, env, event_param, event),
            Expr::SetConstructor => Ok(Value::SetConstructor),
            Expr::SetMethod {
                target,
                method,
                args,
            } => self.eval_set_method(target, *method, args, env, event_param, event),
            Expr::SymbolConstruct {
                description,
                called_with_new,
            } => self.eval_symbol_construct(description, *called_with_new, env, event_param, event),
            Expr::SymbolConstructor => Ok(Value::SymbolConstructor),
            Expr::SymbolStaticMethod { method, args } => {
                self.eval_symbol_static_method(*method, args, env, event_param, event)
            }
            Expr::SymbolStaticProperty(property) => Ok(self.eval_symbol_static_property(*property)),
            Expr::TypedArrayStaticBytesPerElement(kind) => {
                Ok(Value::Number(kind.bytes_per_element() as i64))
            }
            Expr::TypedArrayStaticMethod { kind, method, args } => {
                self.eval_typed_array_static_method(*kind, *method, args, env, event_param, event)
            }
            Expr::TypedArrayByteLength(target) => match env.get(target) {
                Some(Value::TypedArray(array)) => {
                    Ok(Value::Number(array.borrow().observed_byte_length() as i64))
                }
                Some(Value::ArrayBuffer(buffer)) => {
                    Ok(Value::Number(buffer.borrow().byte_length() as i64))
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a TypedArray",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::TypedArrayByteOffset(target) => {
                let array = self.resolve_typed_array_from_env(env, target)?;
                let byte_offset = if array.borrow().observed_length() == 0
                    && array.borrow().byte_offset >= array.borrow().buffer.borrow().byte_length()
                {
                    0
                } else {
                    array.borrow().byte_offset
                };
                Ok(Value::Number(byte_offset as i64))
            }
            Expr::TypedArrayBuffer(target) => {
                let array = self.resolve_typed_array_from_env(env, target)?;
                Ok(Value::ArrayBuffer(array.borrow().buffer.clone()))
            }
            Expr::TypedArrayBytesPerElement(target) => {
                let array = self.resolve_typed_array_from_env(env, target)?;
                Ok(Value::Number(array.borrow().kind.bytes_per_element() as i64))
            }
            Expr::TypedArrayMethod {
                target,
                method,
                args,
            } => self.eval_typed_array_method(target, *method, args, env, event_param, event),
            Expr::EncodeUri(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(encode_uri_like(&value.as_string(), false)))
            }
            Expr::EncodeUriComponent(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(encode_uri_like(&value.as_string(), true)))
            }
            Expr::DecodeUri(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(decode_uri_like(&value.as_string(), false)?))
            }
            Expr::DecodeUriComponent(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(decode_uri_like(&value.as_string(), true)?))
            }
            Expr::Escape(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(js_escape(&value.as_string())))
            }
            Expr::Unescape(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(js_unescape(&value.as_string())))
            }
            Expr::IsNaN(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Bool(Self::coerce_number_for_global(&value).is_nan()))
            }
            Expr::IsFinite(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Bool(
                    Self::coerce_number_for_global(&value).is_finite(),
                ))
            }
            Expr::Atob(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(decode_base64_to_binary_string(
                    &value.as_string(),
                )?))
            }
            Expr::Btoa(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(encode_binary_string_to_base64(
                    &value.as_string(),
                )?))
            }
            Expr::ParseInt { value, radix } => {
                let value = self.eval_expr(value, env, event_param, event)?;
                let radix = radix
                    .as_ref()
                    .map(|expr| self.eval_expr(expr, env, event_param, event))
                    .transpose()?
                    .map(|radix| Self::value_to_i64(&radix));
                Ok(Value::Float(parse_js_parse_int(&value.as_string(), radix)))
            }
            Expr::ParseFloat(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Float(parse_js_parse_float(&value.as_string())))
            }
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
