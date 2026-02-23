use super::*;

impl Harness {
    pub(crate) fn replace_document_with_html(&mut self, html: &str) -> Result<()> {
        let ParseOutput { mut dom, scripts } = parse_html(html)?;
        if scripts
            .iter()
            .any(|script| script.contains("document.body"))
        {
            let _ = dom.ensure_document_body_element()?;
        }
        self.dom = dom;
        self.listeners = ListenerStore::default();
        self.dom_runtime.node_event_handler_props.clear();
        self.dom_runtime.node_expando_props.clear();
        self.script_runtime.env.clear();
        self.scheduler.task_queue.clear();
        self.scheduler.microtask_queue.clear();
        self.scheduler.running_timer_id = None;
        self.scheduler.running_timer_canceled = false;
        self.script_runtime.pending_function_decls.clear();
        self.script_runtime.listener_capture_env_stack.clear();
        self.script_runtime.pending_loop_labels.clear();
        self.script_runtime.loop_label_stack.clear();
        self.dom.set_active_element(None);
        self.dom.set_active_pseudo_element(None);
        self.initialize_global_bindings();
        for script in scripts {
            self.compile_and_register_script(&script)?;
        }
        Ok(())
    }

    pub(crate) fn set_location_property(&mut self, key: &str, value: Value) -> Result<()> {
        match key {
            "href" => self.navigate_location(&value.as_string(), LocationNavigationKind::HrefSet),
            "protocol" => {
                let mut parts = self.current_location_parts();
                let protocol = value.as_string();
                let protocol = protocol.trim_end_matches(':').to_ascii_lowercase();
                if !is_valid_url_scheme(&protocol) {
                    return Err(Error::ScriptRuntime(format!(
                        "invalid location.protocol value: {}",
                        value.as_string()
                    )));
                }
                parts.scheme = protocol;
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "host" => {
                let mut parts = self.current_location_parts();
                let host = value.as_string();
                let (hostname, port) = split_hostname_and_port(host.trim());
                parts.hostname = hostname;
                parts.port = port;
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "hostname" => {
                let mut parts = self.current_location_parts();
                parts.hostname = value.as_string();
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "port" => {
                let mut parts = self.current_location_parts();
                parts.port = value.as_string();
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "pathname" => {
                let mut parts = self.current_location_parts();
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
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "search" => {
                let mut parts = self.current_location_parts();
                parts.search = ensure_search_prefix(&value.as_string());
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "hash" => {
                let mut parts = self.current_location_parts();
                parts.hash = ensure_hash_prefix(&value.as_string());
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "origin" | "ancestorOrigins" => {
                Err(Error::ScriptRuntime(format!("location.{key} is read-only")))
            }
            _ => {
                Self::object_set_entry(
                    &mut self.dom_runtime.location_object.borrow_mut(),
                    key.to_string(),
                    value,
                );
                Ok(())
            }
        }
    }
}
