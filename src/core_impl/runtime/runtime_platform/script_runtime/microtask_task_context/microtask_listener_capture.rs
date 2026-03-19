use super::*;

impl Harness {
    pub(crate) fn push_shared_listener_capture_env_frame(
        &mut self,
        shared_env: Rc<RefCell<ScriptEnv>>,
        shared_env_owned_by_scope: bool,
    ) -> usize {
        self.push_shared_listener_capture_env_frame_with_names(
            shared_env,
            shared_env_owned_by_scope,
            None,
        )
    }

    pub(crate) fn push_shared_listener_capture_env_frame_with_names(
        &mut self,
        shared_env: Rc<RefCell<ScriptEnv>>,
        shared_env_owned_by_scope: bool,
        tracked_names: Option<HashSet<String>>,
    ) -> usize {
        let start_len = self.script_runtime.listener_capture_env_stack.len();
        self.script_runtime
            .listener_capture_env_stack
            .push(ListenerCaptureFrame {
                shared_env: Some(shared_env),
                shared_env_owned_by_scope,
                tracked_names,
                ..ListenerCaptureFrame::default()
            });
        start_len
    }

    pub(crate) fn restore_listener_capture_env_stack(&mut self, start_len: usize) {
        if start_len >= self.script_runtime.listener_capture_env_stack.len() {
            return;
        }

        let mut propagated_updates = HashMap::new();
        for frame in &mut self.script_runtime.listener_capture_env_stack[start_len..] {
            let pending_updates = std::mem::take(&mut frame.pending_env_updates);
            if let Some(shared_env) = frame.shared_env.as_ref() {
                let mut shared_env = shared_env.borrow_mut();
                for (name, value) in &pending_updates {
                    if Self::is_internal_env_key(name) {
                        continue;
                    }
                    if let Some(value) = value {
                        shared_env.insert(name.clone(), value.clone());
                    } else {
                        shared_env.remove(name);
                    }
                }
            }
            propagated_updates.extend(pending_updates);
        }

        self.script_runtime
            .listener_capture_env_stack
            .truncate(start_len);

        if propagated_updates.is_empty() {
            return;
        }

        if let Some(parent) = self.script_runtime.listener_capture_env_stack.last_mut() {
            parent.pending_env_updates.extend(propagated_updates);
        }
    }

    pub(crate) fn is_internal_env_key(name: &str) -> bool {
        name == INTERNAL_RETURN_SLOT || name.starts_with("\u{0}\u{0}bt_")
    }

    pub(crate) fn event_sync_pending_marker_key(name: &str) -> String {
        format!("\u{0}\u{0}bt_evt_sync:{name}")
    }

    pub(crate) fn event_sync_pending_marker_name(name: &str) -> Option<&str> {
        name.strip_prefix("\u{0}\u{0}bt_evt_sync:")
    }

    pub(crate) fn queue_event_sync_pending_update(
        &mut self,
        _env: &HashMap<String, Value>,
        name: &str,
        value: Option<Value>,
    ) {
        if let Some(frame) = self
            .script_runtime
            .listener_capture_env_stack
            .iter_mut()
            .rev()
            .find(|frame| frame.shared_env.is_some() && !frame.shared_env_owned_by_scope)
        {
            frame
                .pending_env_updates
                .insert(Self::event_sync_pending_marker_key(name), value);
            return;
        }
        if let Some(frame) = self
            .script_runtime
            .listener_capture_env_stack
            .iter_mut()
            .rev()
            .find(|frame| frame.shared_env.is_some())
        {
            frame
                .pending_env_updates
                .insert(Self::event_sync_pending_marker_key(name), value);
        }
    }

    pub(crate) fn env_scope_depth(env: &HashMap<String, Value>) -> i64 {
        match env.get(INTERNAL_SCOPE_DEPTH_KEY) {
            Some(Value::Number(depth)) if *depth >= 0 => *depth,
            _ => 0,
        }
    }

    pub(crate) fn env_has_local_or_lexical_binding(
        env: &HashMap<String, Value>,
        name: &str,
    ) -> bool {
        if Self::env_has_local_binding(env, name) {
            return true;
        }
        Self::env_top_level_lexical_binding_names(env).contains(name)
    }

    pub(crate) fn env_local_or_lexical_binding_names(
        env: &HashMap<String, Value>,
    ) -> HashSet<String> {
        let mut names = match env.get(INTERNAL_LOCAL_BINDINGS_KEY) {
            Some(Value::Array(local_bindings)) => local_bindings
                .borrow()
                .iter()
                .filter_map(|entry| match entry {
                    Value::String(value) => Some(value.clone()),
                    _ => None,
                })
                .collect::<HashSet<_>>(),
            _ => HashSet::new(),
        };
        names.extend(Self::env_top_level_lexical_binding_names(env));
        names
    }

    pub(crate) fn env_has_local_binding(env: &HashMap<String, Value>, name: &str) -> bool {
        match env.get(INTERNAL_LOCAL_BINDINGS_KEY) {
            Some(Value::Array(local_bindings)) => local_bindings
                .borrow()
                .iter()
                .any(|entry| matches!(entry, Value::String(value) if value == name)),
            _ => false,
        }
    }

    pub(crate) fn env_should_sync_global_name(env: &HashMap<String, Value>, name: &str) -> bool {
        if Self::env_has_local_or_lexical_binding(env, name) {
            return false;
        }
        match env.get(INTERNAL_GLOBAL_SYNC_NAMES_KEY) {
            Some(Value::Array(names)) => names
                .borrow()
                .iter()
                .any(|entry| matches!(entry, Value::String(value) if value == name)),
            _ => false,
        }
    }

    pub(crate) fn env_has_explicit_binding(env: &HashMap<String, Value>, name: &str) -> bool {
        if Self::is_internal_env_key(name) || !env.contains_key(name) {
            return false;
        }
        !Self::env_should_sync_global_name(env, name)
    }

    pub(crate) fn apply_expression_env_overrides_to_env(
        &mut self,
        env: &mut HashMap<String, Value>,
    ) {
        let overrides = std::mem::take(&mut self.script_runtime.expression_env_overrides);
        for (name, value) in overrides {
            if Self::is_internal_env_key(&name) || Self::env_has_local_binding(env, &name) {
                continue;
            }
            if let Some(value) = value {
                env.insert(name, value);
            } else {
                env.remove(&name);
            }
        }
    }

    pub(crate) fn ensure_listener_capture_env(&mut self) -> Rc<RefCell<ScriptEnv>> {
        if let Some(frame) = self.script_runtime.listener_capture_env_stack.last_mut() {
            if frame.shared_env.is_none() {
                frame.shared_env = Some(Rc::new(RefCell::new(ScriptEnv::default())));
                frame.shared_env_owned_by_scope = true;
            } else if !frame.shared_env_owned_by_scope {
                let mut seeded_env = frame
                    .shared_env
                    .as_ref()
                    .map(|shared_env| shared_env.borrow().clone())
                    .unwrap_or_default();
                for (name, value) in &frame.pending_env_updates {
                    if Self::is_internal_env_key(name) {
                        continue;
                    }
                    if let Some(value) = value {
                        seeded_env.insert(name.clone(), value.clone());
                    } else {
                        seeded_env.remove(name);
                    }
                }
                let shared_env = Rc::new(RefCell::new(seeded_env));
                self.script_runtime
                    .listener_capture_env_stack
                    .push(ListenerCaptureFrame {
                        shared_env: Some(shared_env.clone()),
                        shared_env_owned_by_scope: true,
                        ..ListenerCaptureFrame::default()
                    });
                return shared_env;
            }
            frame
                .shared_env
                .as_ref()
                .expect("shared env should exist after initialization")
                .clone()
        } else {
            Rc::new(RefCell::new(ScriptEnv::default()))
        }
    }

    pub(crate) fn queue_listener_capture_env_update_for_shared_env(
        &mut self,
        shared_env: &Rc<RefCell<ScriptEnv>>,
        name: String,
        value: Option<Value>,
    ) {
        if Self::is_internal_env_key(&name) {
            return;
        }
        for frame in self
            .script_runtime
            .listener_capture_env_stack
            .iter_mut()
            .rev()
        {
            let Some(frame_shared_env) = frame.shared_env.as_ref() else {
                continue;
            };
            if Rc::ptr_eq(frame_shared_env, shared_env) {
                frame.pending_env_updates.insert(name, value);
                return;
            }
        }
        if let Some(frame) = self.script_runtime.listener_capture_env_stack.last_mut() {
            frame.pending_env_updates.insert(name, value);
        }
    }

    pub(crate) fn pending_listener_capture_scope_start(env: &HashMap<String, Value>) -> usize {
        match env.get(INTERNAL_PENDING_SCOPE_START_KEY) {
            Some(Value::Number(start)) if *start >= 0 => *start as usize,
            _ => 0,
        }
    }

    pub(crate) fn listener_capture_pending_updates_snapshot_from(
        &self,
        start: usize,
    ) -> HashMap<String, Option<Value>> {
        let mut updates = HashMap::new();
        let start = start.min(self.script_runtime.listener_capture_env_stack.len());
        for frame in &self.script_runtime.listener_capture_env_stack[start..] {
            updates.extend(frame.pending_env_updates.clone());
        }
        updates
    }

    pub(crate) fn apply_listener_capture_pending_updates_map(
        &mut self,
        env: &mut HashMap<String, Value>,
        updates: HashMap<String, Option<Value>>,
        allow_local_bindings: bool,
    ) {
        if updates.is_empty() {
            return;
        }
        let restricted_names =
            (!allow_local_bindings).then(|| Self::env_local_or_lexical_binding_names(env));
        for (name, value) in updates {
            if Self::is_internal_env_key(&name) {
                continue;
            }
            if restricted_names
                .as_ref()
                .is_some_and(|names| names.contains(&name))
            {
                continue;
            }
            if let Some(value) = value {
                env.insert(name, value);
            } else {
                env.remove(&name);
            }
        }
    }

    pub(crate) fn project_pending_listener_capture_env_updates(
        &mut self,
        env: &mut HashMap<String, Value>,
    ) {
        if self.script_runtime.listener_capture_env_stack.is_empty() {
            return;
        }
        let updates = self.listener_capture_pending_updates_snapshot_from(0);
        let allow_local_bindings = self
            .script_runtime
            .listener_capture_env_stack
            .iter()
            .rev()
            .find_map(|frame| {
                frame
                    .shared_env
                    .as_ref()
                    .map(|_| frame.shared_env_owned_by_scope)
            })
            .unwrap_or(false);
        self.apply_listener_capture_pending_updates_map(env, updates, allow_local_bindings);
    }

    pub(crate) fn apply_pending_listener_capture_env_updates(
        &mut self,
        env: &mut HashMap<String, Value>,
    ) {
        if self.script_runtime.listener_capture_env_stack.is_empty() {
            return;
        }
        let drain_start = Self::pending_listener_capture_scope_start(env)
            .min(self.script_runtime.listener_capture_env_stack.len());
        let updates = self.listener_capture_pending_updates_snapshot_from(drain_start);
        if updates.is_empty() {
            return;
        }
        self.apply_listener_capture_pending_updates_map(env, updates, true);
        for frame in &mut self.script_runtime.listener_capture_env_stack[drain_start..] {
            frame
                .pending_env_updates
                .retain(|name, _| Self::event_sync_pending_marker_name(name).is_some());
        }
    }

    pub(crate) fn push_pending_function_decl_scope(
        &mut self,
        scope: HashMap<String, (ScriptHandler, bool, bool)>,
    ) -> usize {
        let start_len = self.script_runtime.pending_function_decls.len();
        if !scope.is_empty() {
            self.script_runtime
                .pending_function_decls
                .push(Arc::new(scope));
        }
        start_len
    }

    pub(crate) fn push_pending_function_decl_scopes(
        &mut self,
        scopes: &[Arc<HashMap<String, (ScriptHandler, bool, bool)>>],
    ) -> usize {
        let start_len = self.script_runtime.pending_function_decls.len();
        self.script_runtime
            .pending_function_decls
            .extend(scopes.iter().cloned());
        start_len
    }

    pub(crate) fn restore_pending_function_decl_scopes(&mut self, start_len: usize) {
        self.script_runtime
            .pending_function_decls
            .truncate(start_len);
    }

    pub(crate) fn sync_global_binding_if_needed(
        &mut self,
        env: &HashMap<String, Value>,
        name: &str,
        value: &Value,
    ) {
        if Self::env_should_sync_global_name(env, name) {
            self.script_runtime
                .env
                .insert(name.to_string(), value.clone());
        }
    }

    pub(crate) fn sync_scheduled_task_captures_for_binding_if_escaping(
        &mut self,
        env: &HashMap<String, Value>,
        name: &str,
        value: &Value,
    ) {
        if Self::is_internal_env_key(name) {
            return;
        }

        let local_binding = Self::env_has_local_binding(env, name);
        let lexical_binding = Self::env_top_level_lexical_binding_names(env).contains(name);
        let local_scope_start = Self::pending_listener_capture_scope_start(env)
            .min(self.script_runtime.listener_capture_env_stack.len());
        let captured_in_current_function_scope = self
            .script_runtime
            .listener_capture_env_stack
            .iter()
            .enumerate()
            .rev()
            .any(|(frame_index, frame)| {
                if frame_index < local_scope_start || frame.shared_env_owned_by_scope {
                    return false;
                }
                frame.pending_env_updates.contains_key(name)
                    || frame
                        .tracked_names
                        .as_ref()
                        .is_some_and(|tracked_names| tracked_names.contains(name))
                    || frame
                        .shared_env
                        .as_ref()
                        .is_some_and(|shared_env| shared_env.borrow().contains_key(name))
            });
        let scope_local_binding = local_binding
            || (lexical_binding
                && Self::env_scope_depth(env) > 0
                && !captured_in_current_function_scope);
        let active_shared_env = if scope_local_binding {
            self.script_runtime
                .listener_capture_env_stack
                .iter()
                .enumerate()
                .rev()
                .find_map(|(frame_index, frame)| {
                    if frame_index < local_scope_start || !frame.shared_env_owned_by_scope {
                        return None;
                    }
                    frame
                        .shared_env
                        .as_ref()
                        .map(|shared_env| (shared_env.clone(), true))
                })
        } else {
            self.script_runtime
                .listener_capture_env_stack
                .iter()
                .enumerate()
                .rev()
                .find_map(|(frame_index, frame)| {
                    let shared_env = frame.shared_env.as_ref()?;
                    let tracks_name_in_frame = frame.pending_env_updates.contains_key(name)
                        || frame
                            .tracked_names
                            .as_ref()
                            .is_some_and(|tracked_names| tracked_names.contains(name))
                        || shared_env.borrow().contains_key(name);
                    let owned_frame_tracks_name = frame.shared_env_owned_by_scope
                        && (frame_index >= local_scope_start || tracks_name_in_frame);
                    (owned_frame_tracks_name || tracks_name_in_frame)
                        .then(|| (shared_env.clone(), frame.shared_env_owned_by_scope))
                })
        };
        let shared_frame_tracks_name = active_shared_env.is_some();
        if local_binding && !shared_frame_tracks_name {
            self.sync_active_function_captures_in_env_for_binding(env, name, value);
            return;
        }

        if !shared_frame_tracks_name
            && !Self::env_should_sync_global_name(env, name)
            && !lexical_binding
            && !self.listeners.capture_name_counts.contains_key(name)
            && self.scheduler.task_queue.is_empty()
        {
            return;
        }

        if let Some((shared_env, shared_env_owned_by_scope)) = active_shared_env {
            if shared_env_owned_by_scope {
                shared_env
                    .borrow_mut()
                    .insert(name.to_string(), value.clone());
            }
            self.queue_listener_capture_env_update_for_shared_env(
                &shared_env,
                name.to_string(),
                Some(value.clone()),
            );
        }
        self.sync_function_captures_in_env_for_binding(env, name, value);
        if lexical_binding && !local_binding {
            self.sync_runtime_function_captures_for_binding(name, value);
        }
        if !scope_local_binding {
            self.sync_scheduled_task_captures_for_binding(name, value);
        }
    }

    fn sync_function_captures_in_env_for_binding(
        &mut self,
        env: &HashMap<String, Value>,
        name: &str,
        value: &Value,
    ) {
        let mut seen_function_ids = HashSet::new();
        for entry in env.values() {
            let Value::Function(function) = entry else {
                continue;
            };
            if !seen_function_ids.insert(function.function_id) {
                continue;
            }
            if function.global_scope || function.local_bindings.contains(name) {
                continue;
            }
            if !function.captured_names.contains(name) {
                continue;
            }
            function
                .captured_env
                .borrow_mut()
                .insert(name.to_string(), value.clone());
            self.queue_listener_capture_env_update_for_shared_env(
                &function.captured_env,
                name.to_string(),
                Some(value.clone()),
            );
        }
    }

    fn sync_active_function_captures_in_env_for_binding(
        &mut self,
        env: &HashMap<String, Value>,
        name: &str,
        value: &Value,
    ) {
        let mut seen_function_ids = HashSet::new();
        for entry in env.values() {
            let Value::Function(function) = entry else {
                continue;
            };
            if !seen_function_ids.insert(function.function_id) {
                continue;
            }
            if function.global_scope || function.local_bindings.contains(name) {
                continue;
            }
            if !function.captured_names.contains(name) {
                continue;
            }
            let has_active_shared_env =
                self.script_runtime
                    .listener_capture_env_stack
                    .iter()
                    .any(|frame| {
                        frame.shared_env.as_ref().is_some_and(|shared_env| {
                            Rc::ptr_eq(shared_env, &function.captured_env)
                        })
                    });
            if !has_active_shared_env {
                continue;
            }
            function
                .captured_env
                .borrow_mut()
                .insert(name.to_string(), value.clone());
            self.queue_listener_capture_env_update_for_shared_env(
                &function.captured_env,
                name.to_string(),
                Some(value.clone()),
            );
        }
    }

    fn sync_runtime_function_captures_for_binding(&mut self, name: &str, value: &Value) {
        let runtime_values = self
            .script_runtime
            .env
            .to_map()
            .into_values()
            .collect::<Vec<_>>();
        let mut seen_function_ids = HashSet::new();
        for entry in runtime_values {
            let Value::Function(function) = entry else {
                continue;
            };
            if !seen_function_ids.insert(function.function_id) {
                continue;
            }
            if function.global_scope || function.local_bindings.contains(name) {
                continue;
            }
            if !function.captured_names.contains(name) {
                continue;
            }
            function
                .captured_env
                .borrow_mut()
                .insert(name.to_string(), value.clone());
        }
    }
}
