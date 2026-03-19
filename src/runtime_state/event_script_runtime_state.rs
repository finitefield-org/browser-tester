use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Listener {
    pub(crate) capture: bool,
    pub(crate) is_event_handler_property: bool,
    pub(crate) is_arrow: bool,
    pub(crate) handler: ScriptHandler,
    pub(crate) function: Option<Rc<FunctionValue>>,
    pub(crate) captured_names: HashSet<String>,
    pub(crate) captured_env: Rc<RefCell<ScriptEnv>>,
    pub(crate) captured_pending_function_decls:
        Vec<Arc<HashMap<String, (ScriptHandler, bool, bool)>>>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ListenerStore {
    pub(crate) map: HashMap<NodeId, HashMap<String, Vec<Listener>>>,
    pub(crate) capture_name_counts: HashMap<String, usize>,
    pub(crate) capture_envs_by_name: HashMap<String, Vec<std::rc::Weak<RefCell<ScriptEnv>>>>,
}

impl ListenerStore {
    fn increment_capture_names(&mut self, listener: &Listener) {
        for name in &listener.captured_names {
            *self.capture_name_counts.entry(name.clone()).or_insert(0) += 1;
            let tracked_envs = self.capture_envs_by_name.entry(name.clone()).or_default();
            tracked_envs.retain(|weak_env| weak_env.strong_count() > 0);
            if tracked_envs
                .iter()
                .filter_map(|weak_env| weak_env.upgrade())
                .any(|existing| Rc::ptr_eq(&existing, &listener.captured_env))
            {
                continue;
            }
            tracked_envs.push(Rc::downgrade(&listener.captured_env));
        }
    }

    fn decrement_capture_names(&mut self, listener: &Listener) {
        for name in &listener.captured_names {
            let Some(count) = self.capture_name_counts.get_mut(name) else {
                continue;
            };
            if *count <= 1 {
                self.capture_name_counts.remove(name);
                self.capture_envs_by_name.remove(name);
            } else {
                *count -= 1;
            }
        }
    }

    pub(crate) fn captured_envs_for_name(&mut self, name: &str) -> Vec<Rc<RefCell<ScriptEnv>>> {
        let mut captured_envs = Vec::new();
        let mut seen = HashSet::new();
        let mut remove_name_entry = false;

        if let Some(tracked_envs) = self.capture_envs_by_name.get_mut(name) {
            tracked_envs.retain(|weak_env| {
                let Some(env) = weak_env.upgrade() else {
                    return false;
                };
                let env_id = Rc::as_ptr(&env) as usize;
                if seen.insert(env_id) {
                    captured_envs.push(env);
                }
                true
            });
            remove_name_entry = tracked_envs.is_empty();
        }

        if remove_name_entry {
            self.capture_envs_by_name.remove(name);
        }

        captured_envs
    }

    pub(crate) fn add(&mut self, node_id: NodeId, event: String, listener: Listener) {
        let should_add = {
            let listeners = self
                .map
                .entry(node_id)
                .or_default()
                .entry(event.clone())
                .or_default();

            if !listener.is_event_handler_property {
                if let Some(new_callback_ref) = listener.handler.listener_callback_reference() {
                    if listeners.iter().any(|existing| {
                        !existing.is_event_handler_property
                            && existing.capture == listener.capture
                            && existing
                                .handler
                                .listener_callback_reference()
                                .is_some_and(|existing_ref| existing_ref == new_callback_ref)
                    }) {
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            } else {
                true
            }
        };
        if !should_add {
            return;
        }
        self.increment_capture_names(&listener);
        self.map
            .entry(node_id)
            .or_default()
            .entry(event)
            .or_default()
            .push(listener);
    }

    pub(crate) fn remove(
        &mut self,
        node_id: NodeId,
        event: &str,
        capture: bool,
        handler: &ScriptHandler,
    ) -> bool {
        let Some((listener, remove_node)) = ({
            let Some(events) = self.map.get_mut(&node_id) else {
                return false;
            };

            let listener = {
                let Some(listeners) = events.get_mut(event) else {
                    return false;
                };
                let Some(pos) = listeners.iter().position(|listener| {
                    !listener.is_event_handler_property
                        && listener.capture == capture
                        && listener.handler == *handler
                }) else {
                    return false;
                };
                let listener = listeners.remove(pos);
                let remove_event = listeners.is_empty();
                if remove_event {
                    events.remove(event);
                }
                listener
            };

            Some((listener, events.is_empty()))
        }) else {
            return false;
        };

        self.decrement_capture_names(&listener);
        if remove_node {
            self.map.remove(&node_id);
        }
        true
    }

    pub(crate) fn remove_event_handler_property(
        &mut self,
        node_id: NodeId,
        event: &str,
        handler: &ScriptHandler,
    ) -> bool {
        let Some((listener, remove_node)) = ({
            let Some(events) = self.map.get_mut(&node_id) else {
                return false;
            };

            let listener = {
                let Some(listeners) = events.get_mut(event) else {
                    return false;
                };
                let Some(pos) = listeners.iter().position(|listener| {
                    listener.is_event_handler_property && listener.handler == *handler
                }) else {
                    return false;
                };
                let listener = listeners.remove(pos);
                let remove_event = listeners.is_empty();
                if remove_event {
                    events.remove(event);
                }
                listener
            };

            Some((listener, events.is_empty()))
        }) else {
            return false;
        };

        self.decrement_capture_names(&listener);
        if remove_node {
            self.map.remove(&node_id);
        }
        true
    }

    pub(crate) fn replace_event_handler_property(
        &mut self,
        node_id: NodeId,
        event: &str,
        previous_handler: &ScriptHandler,
        listener: Listener,
    ) -> bool {
        let Some(events) = self.map.get_mut(&node_id) else {
            return false;
        };
        let Some(listeners) = events.get_mut(event) else {
            return false;
        };

        if let Some(pos) = listeners.iter().position(|existing| {
            existing.is_event_handler_property && existing.handler == *previous_handler
        }) {
            listeners[pos] = listener;
            return true;
        }

        false
    }

    pub(crate) fn get(&self, node_id: NodeId, event: &str, capture: bool) -> Vec<Listener> {
        self.map
            .get(&node_id)
            .and_then(|events| events.get(event))
            .map(|listeners| {
                listeners
                    .iter()
                    .filter(|listener| listener.capture == capture)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EventState {
    pub(crate) event_type: String,
    pub(crate) target: NodeId,
    pub(crate) current_target: NodeId,
    pub(crate) target_value: Option<Value>,
    pub(crate) current_target_value: Option<Value>,
    pub(crate) event_phase: i32,
    pub(crate) time_stamp_ms: i64,
    pub(crate) default_prevented: bool,
    pub(crate) is_trusted: bool,
    pub(crate) bubbles: bool,
    pub(crate) cancelable: bool,
    pub(crate) detail: Option<Value>,
    pub(crate) submitter: Option<Value>,
    pub(crate) hash_change_interface: bool,
    pub(crate) hash_change_old_url: String,
    pub(crate) hash_change_new_url: String,
    pub(crate) error_event_interface: bool,
    pub(crate) error_event_message: String,
    pub(crate) error_event_filename: String,
    pub(crate) error_event_lineno: i64,
    pub(crate) error_event_colno: i64,
    pub(crate) error_event_error: Value,
    pub(crate) before_unload_interface: bool,
    pub(crate) before_unload_return_value: String,
    pub(crate) state: Option<Value>,
    pub(crate) old_state: Option<String>,
    pub(crate) new_state: Option<String>,
    pub(crate) key: Option<String>,
    pub(crate) code: Option<String>,
    pub(crate) location: i64,
    pub(crate) ctrl_key: bool,
    pub(crate) meta_key: bool,
    pub(crate) shift_key: bool,
    pub(crate) alt_key: bool,
    pub(crate) repeat: bool,
    pub(crate) is_composing: bool,
    pub(crate) delta_x: f64,
    pub(crate) delta_y: f64,
    pub(crate) delta_z: f64,
    pub(crate) delta_mode: i64,
    pub(crate) pointer_id: i64,
    pub(crate) pointer_width: f64,
    pub(crate) pointer_height: f64,
    pub(crate) pointer_pressure: f64,
    pub(crate) pointer_tangential_pressure: f64,
    pub(crate) pointer_tilt_x: i64,
    pub(crate) pointer_tilt_y: i64,
    pub(crate) pointer_twist: i64,
    pub(crate) pointer_type: String,
    pub(crate) pointer_is_primary: bool,
    pub(crate) pointer_altitude_angle: f64,
    pub(crate) pointer_azimuth_angle: f64,
    pub(crate) pointer_persistent_device_id: i64,
    pub(crate) navigate_can_intercept: bool,
    pub(crate) navigate_destination: Option<Value>,
    pub(crate) navigate_download_request: Option<Value>,
    pub(crate) navigate_form_data: Option<Value>,
    pub(crate) navigate_hash_change: bool,
    pub(crate) navigate_has_ua_visual_transition: bool,
    pub(crate) navigate_info: Option<Value>,
    pub(crate) navigate_navigation_type: Option<String>,
    pub(crate) navigate_signal: Option<Value>,
    pub(crate) navigate_source_element: Option<Value>,
    pub(crate) navigate_user_initiated: bool,
    pub(crate) message_data: Option<Value>,
    pub(crate) message_origin: Option<String>,
    pub(crate) message_source: Option<Value>,
    pub(crate) clipboard_data: Option<String>,
    pub(crate) clipboard_data_object: Option<Rc<RefCell<ObjectValue>>>,
    pub(crate) data_transfer_object: Option<Rc<RefCell<ObjectValue>>>,
    pub(crate) propagation_stopped: bool,
    pub(crate) immediate_propagation_stopped: bool,
}

impl EventState {
    pub(crate) fn new(event_type: &str, target: NodeId, time_stamp_ms: i64) -> Self {
        Self {
            event_type: event_type.to_string(),
            target,
            current_target: target,
            target_value: None,
            current_target_value: None,
            event_phase: 2,
            time_stamp_ms,
            default_prevented: false,
            is_trusted: true,
            bubbles: true,
            cancelable: true,
            detail: None,
            submitter: None,
            hash_change_interface: false,
            hash_change_old_url: String::new(),
            hash_change_new_url: String::new(),
            error_event_interface: false,
            error_event_message: String::new(),
            error_event_filename: String::new(),
            error_event_lineno: 0,
            error_event_colno: 0,
            error_event_error: Value::Null,
            before_unload_interface: event_type.eq_ignore_ascii_case("beforeunload"),
            before_unload_return_value: String::new(),
            state: None,
            old_state: None,
            new_state: None,
            key: None,
            code: None,
            location: 0,
            ctrl_key: false,
            meta_key: false,
            shift_key: false,
            alt_key: false,
            repeat: false,
            is_composing: false,
            delta_x: 0.0,
            delta_y: 0.0,
            delta_z: 0.0,
            delta_mode: 0,
            pointer_id: 0,
            pointer_width: 1.0,
            pointer_height: 1.0,
            pointer_pressure: 0.0,
            pointer_tangential_pressure: 0.0,
            pointer_tilt_x: 0,
            pointer_tilt_y: 0,
            pointer_twist: 0,
            pointer_type: String::new(),
            pointer_is_primary: false,
            pointer_altitude_angle: 0.0,
            pointer_azimuth_angle: 0.0,
            pointer_persistent_device_id: 0,
            navigate_can_intercept: false,
            navigate_destination: None,
            navigate_download_request: None,
            navigate_form_data: None,
            navigate_hash_change: false,
            navigate_has_ua_visual_transition: false,
            navigate_info: None,
            navigate_navigation_type: None,
            navigate_signal: None,
            navigate_source_element: None,
            navigate_user_initiated: false,
            message_data: None,
            message_origin: None,
            message_source: None,
            clipboard_data: None,
            clipboard_data_object: None,
            data_transfer_object: None,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
        }
    }

    pub(crate) fn new_untrusted(event_type: &str, target: NodeId, time_stamp_ms: i64) -> Self {
        let mut event = Self::new(event_type, target, time_stamp_ms);
        event.is_trusted = false;
        event.bubbles = false;
        event.cancelable = false;
        event
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScriptSource {
    pub(crate) code: String,
    pub(crate) is_module: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ParseOutput {
    pub(crate) dom: Dom,
    pub(crate) scripts: Vec<ScriptSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScheduledTaskKind {
    Timeout,
    Interval,
    AnimationFrame,
}

#[derive(Debug, Clone)]
pub(crate) struct ScheduledTask {
    pub(crate) id: i64,
    pub(crate) due_at: i64,
    pub(crate) order: i64,
    pub(crate) kind: ScheduledTaskKind,
    pub(crate) interval_ms: Option<i64>,
    pub(crate) callback: TimerCallback,
    pub(crate) callback_args: Vec<Value>,
    pub(crate) env: ScriptEnv,
}

#[derive(Debug, Clone)]
pub(crate) enum ScheduledMicrotask {
    Script {
        handler: ScriptHandler,
        env: ScriptEnv,
    },
    Callable {
        callback: Value,
    },
    WorkerMessage {
        worker: Rc<RefCell<ObjectValue>>,
        target: Rc<RefCell<ObjectValue>>,
        target_this: Value,
        data: Value,
    },
    Promise {
        reaction: PromiseReactionKind,
        settled: PromiseSettledValue,
    },
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ScriptEnv {
    pub(crate) inner: Arc<HashMap<String, Value>>,
}

impl ScriptEnv {
    fn clone_env_map(env: &HashMap<String, Value>) -> HashMap<String, Value> {
        let mut cloned = env.clone();
        let Some(Value::Object(bindings)) = cloned.get(INTERNAL_CONST_BINDINGS_KEY).cloned() else {
            return cloned;
        };
        cloned.insert(
            INTERNAL_CONST_BINDINGS_KEY.to_string(),
            Value::Object(Rc::new(RefCell::new(bindings.borrow().clone()))),
        );
        cloned
    }

    pub(crate) fn share(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    pub(crate) fn from_snapshot(env: &HashMap<String, Value>) -> Self {
        Self {
            inner: Arc::new(Self::clone_env_map(env)),
        }
    }

    pub(crate) fn to_map(&self) -> HashMap<String, Value> {
        Self::clone_env_map(self.inner.as_ref())
    }
}

impl std::ops::Deref for ScriptEnv {
    type Target = HashMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl std::ops::DerefMut for ScriptEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.inner)
    }
}

#[derive(Debug, Default)]
pub(crate) struct ListenerCaptureFrame {
    pub(crate) shared_env: Option<Rc<RefCell<ScriptEnv>>>,
    pub(crate) shared_env_owned_by_scope: bool,
    pub(crate) tracked_names: Option<HashSet<String>>,
    pub(crate) pending_env_updates: HashMap<String, Option<Value>>,
}

#[derive(Debug)]
pub(crate) struct PendingAsyncFunctionSuspend {
    pub(crate) awaited_promise: Rc<RefCell<PromiseValue>>,
    pub(crate) continuation: Value,
}

#[derive(Debug, Clone)]
pub(crate) enum ModuleExportBinding {
    Local(String),
    Value(Value),
}

#[derive(Debug, Default, Clone)]
pub(crate) struct TdzScopeFrame {
    pub(crate) declared: HashSet<String>,
    pub(crate) pending: HashSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrivateBindingKind {
    Field,
    Method,
    Accessor,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PrivateBindingRuntime {
    pub(crate) name: String,
    pub(crate) slot_id: usize,
    pub(crate) is_static: bool,
    pub(crate) kind: PrivateBindingKind,
    pub(crate) has_getter: bool,
    pub(crate) has_setter: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PrivateInitializerRuntime {
    pub(crate) binding: PrivateBindingRuntime,
    pub(crate) initializer: Option<Expr>,
    pub(crate) value: Option<Value>,
    pub(crate) setter_value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PublicFieldInitializerRuntime {
    pub(crate) name: String,
    pub(crate) initializer: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ConstructorInstanceInitializerRuntime {
    Private(PrivateInitializerRuntime),
    Public(PublicFieldInitializerRuntime),
}

#[derive(Debug, Default)]
pub(crate) struct ScriptRuntimeState {
    pub(crate) env: ScriptEnv,
    pub(crate) pending_function_decls: Vec<Arc<HashMap<String, (ScriptHandler, bool, bool)>>>,
    pub(crate) listener_capture_env_stack: Vec<ListenerCaptureFrame>,
    pub(crate) expression_env_overrides: HashMap<String, Option<Value>>,
    pub(crate) generator_yield_stack: Vec<Rc<RefCell<Vec<Value>>>>,
    pub(crate) pending_loop_labels: Vec<Vec<String>>,
    pub(crate) loop_label_stack: Vec<HashSet<String>>,
    pub(crate) tdz_scope_stack: Vec<TdzScopeFrame>,
    pub(crate) module_export_stack: Vec<Rc<RefCell<HashMap<String, ModuleExportBinding>>>>,
    pub(crate) module_referrer_stack: Vec<String>,
    pub(crate) module_cache: HashMap<String, HashMap<String, Value>>,
    pub(crate) module_namespace_cache: HashMap<String, Value>,
    pub(crate) loading_modules: HashSet<String>,
    pub(crate) next_function_id: usize,
    pub(crate) next_private_slot_id: usize,
    pub(crate) function_private_bindings: HashMap<usize, HashMap<String, PrivateBindingRuntime>>,
    pub(crate) function_public_properties: HashMap<usize, ObjectValue>,
    pub(crate) function_registry: HashMap<usize, Rc<FunctionValue>>,
    pub(crate) constructor_instance_initializers:
        HashMap<usize, Vec<ConstructorInstanceInitializerRuntime>>,
    pub(crate) constructor_call_stack: Vec<usize>,
    pub(crate) constructor_instance_initialized_stack: Vec<bool>,
    pub(crate) private_binding_stack: Vec<HashMap<String, PrivateBindingRuntime>>,
    pub(crate) private_instance_slots: HashMap<usize, HashMap<usize, Value>>,
    pub(crate) private_static_slots: HashMap<usize, HashMap<usize, Value>>,
    pub(crate) event_target_listener_nodes: HashMap<usize, NodeId>,
    pub(crate) next_event_target_listener_slot: usize,
    pub(crate) builtin_constructor_prototypes: HashMap<String, Rc<RefCell<ObjectValue>>>,
    pub(crate) variant_callable_public_properties: HashMap<String, ObjectValue>,
    pub(crate) string_constructor_prototype: Option<Rc<RefCell<ObjectValue>>>,
    pub(crate) symbol_constructor_prototype: Option<Rc<RefCell<ObjectValue>>>,
    pub(crate) typed_array_constructor_prototypes: HashMap<String, Rc<RefCell<ObjectValue>>>,
    pub(crate) constructor_static_methods: HashMap<String, Value>,
    pub(crate) pending_async_function_suspend: Option<PendingAsyncFunctionSuspend>,
}

impl ScriptRuntimeState {
    pub(crate) fn allocate_function_id(&mut self) -> usize {
        let id = self.next_function_id;
        self.next_function_id = self.next_function_id.saturating_add(1);
        id
    }

    pub(crate) fn allocate_private_slot_id(&mut self) -> usize {
        let id = self.next_private_slot_id;
        self.next_private_slot_id = self.next_private_slot_id.saturating_add(1);
        id
    }
}
