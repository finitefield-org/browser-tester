use super::*;

impl Harness {
    pub(crate) fn value_to_i64(value: &Value) -> i64 {
        match value {
            Value::Number(v) => *v,
            Value::Float(v) => *v as i64,
            Value::BigInt(v) => v.to_i64().unwrap_or_else(|| {
                if v.sign() == Sign::Minus {
                    i64::MIN
                } else {
                    i64::MAX
                }
            }),
            Value::Bool(v) => {
                if *v {
                    1
                } else {
                    0
                }
            }
            Value::String(v) => v
                .parse::<i64>()
                .ok()
                .or_else(|| v.parse::<f64>().ok().map(|n| n as i64))
                .unwrap_or(0),
            Value::Array(values) => Value::Array(values.clone())
                .as_string()
                .parse::<i64>()
                .ok()
                .or_else(|| {
                    Value::Array(values.clone())
                        .as_string()
                        .parse::<f64>()
                        .ok()
                        .map(|n| n as i64)
                })
                .unwrap_or(0),
            Value::Date(value) => *value.borrow(),
            Value::Object(_) => 0,
            Value::Promise(_) => 0,
            Value::Map(_) => 0,
            Value::WeakMap(_) => 0,
            Value::Set(_) => 0,
            Value::WeakSet(_) => 0,
            Value::Blob(_) => 0,
            Value::ArrayBuffer(_) => 0,
            Value::TypedArray(_) => 0,
            Value::StringConstructor => 0,
            Value::TypedArrayConstructor(_) => 0,
            Value::BlobConstructor => 0,
            Value::UrlConstructor => 0,
            Value::ArrayBufferConstructor => 0,
            Value::PromiseConstructor => 0,
            Value::MapConstructor => 0,
            Value::WeakMapConstructor => 0,
            Value::SetConstructor => 0,
            Value::WeakSetConstructor => 0,
            Value::SymbolConstructor => 0,
            Value::RegExpConstructor => 0,
            Value::PromiseCapability(_) => 0,
            Value::Symbol(_) => 0,
            Value::RegExp(_) => 0,
            Value::Node(_) => 0,
            Value::NodeList(_) => 0,
            Value::FormData(_) => 0,
            Value::Function(_) => 0,
            Value::Null => 0,
            Value::Undefined => 0,
        }
    }

    pub(crate) fn next_random_f64(&mut self) -> f64 {
        // xorshift64*: simple deterministic PRNG for test runtime.
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = if x == 0 { 0xA5A5_A5A5_A5A5_A5A5 } else { x };
        let out = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        // Convert top 53 bits to [0.0, 1.0).
        let mantissa = out >> 11;
        (mantissa as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    pub(crate) fn schedule_timeout(
        &mut self,
        callback: TimerCallback,
        delay_ms: i64,
        callback_args: Vec<Value>,
        env: &HashMap<String, Value>,
    ) -> i64 {
        let delay_ms = delay_ms.max(0);
        let due_at = self.scheduler.now_ms.saturating_add(delay_ms);
        let id = self.scheduler.allocate_timer_id();
        let order = self.scheduler.allocate_task_order();
        self.scheduler.task_queue.push(ScheduledTask {
            id,
            due_at,
            order,
            interval_ms: None,
            callback,
            callback_args,
            env: ScriptEnv::from_snapshot(env),
        });
        self.trace_timer_line(format!(
            "[timer] schedule timeout id={} due_at={} delay_ms={}",
            id, due_at, delay_ms
        ));
        id
    }

    pub(crate) fn schedule_interval(
        &mut self,
        callback: TimerCallback,
        interval_ms: i64,
        callback_args: Vec<Value>,
        env: &HashMap<String, Value>,
    ) -> i64 {
        let interval_ms = interval_ms.max(0);
        let due_at = self.scheduler.now_ms.saturating_add(interval_ms);
        let id = self.scheduler.allocate_timer_id();
        let order = self.scheduler.allocate_task_order();
        self.scheduler.task_queue.push(ScheduledTask {
            id,
            due_at,
            order,
            interval_ms: Some(interval_ms),
            callback,
            callback_args,
            env: ScriptEnv::from_snapshot(env),
        });
        self.trace_timer_line(format!(
            "[timer] schedule interval id={} due_at={} interval_ms={}",
            id, due_at, interval_ms
        ));
        id
    }

    pub(crate) fn clear_timeout(&mut self, id: i64) {
        let before = self.scheduler.task_queue.len();
        self.scheduler.task_queue.retain(|task| task.id != id);
        let removed = before.saturating_sub(self.scheduler.task_queue.len());
        let mut running_canceled = false;
        if self.scheduler.running_timer_id == Some(id) {
            self.scheduler.running_timer_canceled = true;
            running_canceled = true;
        }
        self.trace_timer_line(format!(
            "[timer] clear id={} removed={} running_canceled={}",
            id, removed, running_canceled
        ));
    }

    pub(crate) fn compile_and_register_script(
        &mut self,
        script: &str,
        is_module: bool,
    ) -> Result<()> {
        stacker::grow(32 * 1024 * 1024, || -> Result<()> {
            let stmts = if is_module {
                parse_module_block_statements(script)?
            } else {
                parse_block_statements(script)?
            };
            if is_module {
                self.script_runtime
                    .module_referrer_stack
                    .push(self.document_url.clone());
            }
            let result = self.with_script_env(|this, env| {
                let mut event = EventState::new("script", this.dom.root, this.scheduler.now_ms);
                this.run_in_task_context(|inner| {
                    inner
                        .execute_stmts(&stmts, &None, &mut event, env)
                        .map(|_| ())
                })
            });
            if is_module {
                let _ = self.script_runtime.module_referrer_stack.pop();
            }
            result?;
            Ok(())
        })
    }

    pub(crate) fn resolve_module_specifier_key(&self, specifier: &str, referrer: &str) -> String {
        let specifier = specifier.trim();
        if specifier.starts_with("data:") {
            return specifier.to_string();
        }
        Self::resolve_url_string(specifier, Some(referrer)).unwrap_or_else(|| specifier.to_string())
    }

    pub(crate) fn parse_data_module_source(specifier: &str) -> Result<(String, String)> {
        let Some(rest) = specifier.strip_prefix("data:") else {
            return Err(Error::ScriptRuntime(format!(
                "invalid data module specifier: {specifier}"
            )));
        };
        let Some((meta, payload)) = rest.split_once(',') else {
            return Err(Error::ScriptRuntime(format!(
                "invalid data module specifier: {specifier}"
            )));
        };
        let media_type = meta
            .split(';')
            .next()
            .map(str::trim)
            .unwrap_or_default()
            .to_ascii_lowercase();
        if meta
            .split(';')
            .skip(1)
            .any(|part| part.trim().eq_ignore_ascii_case("base64"))
        {
            return Err(Error::ScriptRuntime(
                "base64 data URL modules are not supported yet".into(),
            ));
        }
        let source = decode_uri_like(payload, true)?;
        Ok((media_type, source))
    }

    pub(crate) fn load_module_exports(
        &mut self,
        specifier: &str,
        attribute_type: Option<&str>,
        referrer: &str,
    ) -> Result<HashMap<String, Value>> {
        let cache_key = self.resolve_module_specifier_key(specifier, referrer);
        if let Some(cached) = self.script_runtime.module_cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        if !self.script_runtime.loading_modules.insert(cache_key.clone()) {
            return Err(Error::ScriptRuntime(format!(
                "circular module import is not supported: {cache_key}"
            )));
        }

        let result = (|| -> Result<HashMap<String, Value>> {
            let (media_type, source) = if cache_key.starts_with("data:") {
                Self::parse_data_module_source(&cache_key)?
            } else {
                let source = self
                    .platform_mocks
                    .fetch_mocks
                    .get(&cache_key)
                    .cloned()
                    .or_else(|| self.platform_mocks.fetch_mocks.get(specifier).cloned())
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!(
                            "module source mock not found for import: {specifier}"
                        ))
                    })?;
                ("text/javascript".to_string(), source)
            };

            let is_json_module = attribute_type.is_some_and(|ty| ty == "json")
                || media_type.contains("application/json");
            if is_json_module {
                let value = Self::parse_json_text(&source)?;
                return Ok(HashMap::from([("default".to_string(), value)]));
            }

            let stmts = parse_module_block_statements(&source)?;
            let export_collector = Rc::new(RefCell::new(HashMap::new()));
            self.script_runtime
                .module_export_stack
                .push(export_collector.clone());
            self.script_runtime
                .module_referrer_stack
                .push(cache_key.clone());
            let exec_result = self.with_script_env(|this, env| {
                let mut event = EventState::new("script", this.dom.root, this.scheduler.now_ms);
                this.run_in_task_context(|inner| {
                    inner
                        .execute_stmts(&stmts, &None, &mut event, env)
                        .map(|_| ())
                })
            });
            let _ = self.script_runtime.module_referrer_stack.pop();
            let _ = self.script_runtime.module_export_stack.pop();
            exec_result?;

            let env_snapshot = self.script_runtime.env.to_map();
            let mut exports = HashMap::new();
            for (exported, binding) in export_collector.borrow().iter() {
                let value = match binding {
                    ModuleExportBinding::Local(local) => env_snapshot
                        .get(local)
                        .cloned()
                        .or_else(|| self.resolve_pending_function_decl(local, &env_snapshot))
                        .unwrap_or(Value::Undefined),
                    ModuleExportBinding::Value(value) => value.clone(),
                };
                exports.insert(exported.clone(), value);
            }
            Ok(exports)
        })();

        self.script_runtime.loading_modules.remove(&cache_key);
        let exports = result?;
        self.script_runtime
            .module_cache
            .insert(cache_key.clone(), exports.clone());
        Ok(exports)
    }
}
