use super::*;

impl Harness {
    fn navigate_location_parts_if_changed(&mut self, parts: &LocationParts) -> Result<()> {
        let mut next_parts = parts.clone();
        Self::normalize_url_parts_for_serialization(&mut next_parts);
        let next_href = next_parts.href();
        if next_href == self.current_location_parts().href() {
            return Ok(());
        }
        self.navigate_location(&next_href, LocationNavigationKind::HrefSet)
    }

    pub(crate) fn replace_document_with_html(&mut self, html: &str) -> Result<()> {
        let ParseOutput { mut dom, scripts } = parse_html(html)?;
        if scripts
            .iter()
            .any(|script| script.code.contains("document.body"))
        {
            let _ = dom.ensure_document_body_element()?;
        }
        self.dom = dom;
        self.listeners = ListenerStore::default();
        self.dom_runtime.node_event_handler_props.clear();
        self.dom_runtime.node_expando_props.clear();
        self.dom_runtime.live_child_nodes_lists.clear();
        self.dom_runtime.live_children_lists.clear();
        self.dom_runtime.live_named_node_maps.clear();
        self.script_runtime.env.clear();
        self.scheduler.task_queue.clear();
        self.scheduler.microtask_queue.clear();
        self.scheduler.running_timer_id = None;
        self.scheduler.running_timer_canceled = false;
        self.script_runtime.pending_function_decls.clear();
        self.script_runtime.listener_capture_env_stack.clear();
        self.script_runtime.pending_loop_labels.clear();
        self.script_runtime.loop_label_stack.clear();
        self.script_runtime.module_export_stack.clear();
        self.script_runtime.module_referrer_stack.clear();
        self.script_runtime.module_cache.clear();
        self.script_runtime.module_namespace_cache.clear();
        self.script_runtime.loading_modules.clear();
        self.script_runtime.event_target_listener_nodes.clear();
        self.script_runtime.next_event_target_listener_slot = 0;
        self.dom.set_active_element(None);
        self.dom.set_active_pseudo_element(None);
        self.dom_runtime.document_ready_state = "loading".to_string();
        self.dom_runtime.document_visibility_state = "visible".to_string();
        self.dom_runtime.document_scroll_x = 0;
        self.dom_runtime.document_scroll_y = 0;
        self.initialize_global_bindings();
        for script in scripts {
            self.compile_and_register_script(&script.code, script.is_module)?;
        }
        self.finalize_document_ready_state_with_dom_content_loaded()?;
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
                if !parts.apply_protocol_setter(&protocol) {
                    return Ok(());
                }
                self.navigate_location_parts_if_changed(&parts)
            }
            "host" => {
                let mut parts = self.current_location_parts();
                if !parts.apply_host_setter(&value.as_string()) {
                    return Ok(());
                }
                self.navigate_location_parts_if_changed(&parts)
            }
            "hostname" => {
                let mut parts = self.current_location_parts();
                if !parts.apply_hostname_setter(&value.as_string()) {
                    return Ok(());
                }
                self.navigate_location_parts_if_changed(&parts)
            }
            "port" => {
                let mut parts = self.current_location_parts();
                if !parts.apply_port_setter(&value.as_string()) {
                    return Ok(());
                }
                self.navigate_location_parts_if_changed(&parts)
            }
            "pathname" => {
                let mut parts = self.current_location_parts();
                if !parts.has_authority {
                    return Ok(());
                }
                let raw = value.as_string();
                let normalized_input = if raw.starts_with('/') {
                    raw
                } else {
                    format!("/{raw}")
                };
                parts.pathname = normalize_pathname(&normalized_input);
                self.navigate_location_parts_if_changed(&parts)
            }
            "search" => {
                let mut parts = self.current_location_parts();
                parts.search = ensure_search_prefix(&value.as_string());
                self.navigate_location_parts_if_changed(&parts)
            }
            "hash" => {
                let mut parts = self.current_location_parts();
                parts.hash = ensure_hash_prefix(&value.as_string());
                self.navigate_location_parts_if_changed(&parts)
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
