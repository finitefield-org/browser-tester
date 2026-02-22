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

    pub(crate) fn compile_and_register_script(&mut self, script: &str) -> Result<()> {
        stacker::grow(32 * 1024 * 1024, || -> Result<()> {
            let stmts = parse_block_statements(script)?;
            self.with_script_env(|this, env| {
                let mut event = EventState::new("script", this.dom.root, this.scheduler.now_ms);
                this.run_in_task_context(|inner| {
                    inner
                        .execute_stmts(&stmts, &None, &mut event, env)
                        .map(|_| ())
                })
            })?;
            Ok(())
        })
    }
}
