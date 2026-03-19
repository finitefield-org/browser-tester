use super::*;

impl Harness {
    pub(crate) fn new_static_node_list_value(nodes: Vec<NodeId>) -> Value {
        Value::NodeList(Rc::new(RefCell::new(NodeListValue::static_list(nodes))))
    }

    fn document_query_selector_live_list_value(&mut self, selector: &'static str) -> Value {
        let existing = match selector {
            "form" => self.dom_runtime.live_document_forms_list.clone(),
            "img" => self.dom_runtime.live_document_images_list.clone(),
            "a[href], area[href]" => self.dom_runtime.live_document_links_list.clone(),
            "script" => self.dom_runtime.live_document_scripts_list.clone(),
            _ => None,
        };
        let list = existing.unwrap_or_else(|| {
            let nodes = self.dom.query_selector_all(selector).unwrap_or_default();
            let list = Rc::new(RefCell::new(NodeListValue::live_query_selector_all(
                self.dom.root,
                selector.to_string(),
                NodeListKind::HtmlCollection,
                nodes,
            )));
            match selector {
                "form" => self.dom_runtime.live_document_forms_list = Some(list.clone()),
                "img" => self.dom_runtime.live_document_images_list = Some(list.clone()),
                "a[href], area[href]" => {
                    self.dom_runtime.live_document_links_list = Some(list.clone())
                }
                "script" => self.dom_runtime.live_document_scripts_list = Some(list.clone()),
                _ => {}
            }
            list
        });
        self.refresh_node_list(&list);
        Value::NodeList(list)
    }

    pub(crate) fn document_forms_live_list_value(&mut self) -> Value {
        self.document_query_selector_live_list_value("form")
    }

    pub(crate) fn document_images_live_list_value(&mut self) -> Value {
        self.document_query_selector_live_list_value("img")
    }

    pub(crate) fn document_links_live_list_value(&mut self) -> Value {
        self.document_query_selector_live_list_value("a[href], area[href]")
    }

    pub(crate) fn document_scripts_live_list_value(&mut self) -> Value {
        self.document_query_selector_live_list_value("script")
    }

    pub(crate) fn class_names_from_argument(value: &Value) -> Vec<String> {
        value
            .as_string()
            .split_whitespace()
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    pub(crate) fn class_names_live_list_value(
        &self,
        root: NodeId,
        class_names: Vec<String>,
    ) -> Value {
        let nodes = self
            .dom
            .get_elements_by_class_names_from(&root, &class_names);
        Value::NodeList(Rc::new(RefCell::new(
            NodeListValue::live_descendants_by_class_names(root, class_names, nodes),
        )))
    }

    pub(crate) fn name_live_list_value(&self, root: NodeId, name: String) -> Value {
        let nodes = self.dom.get_elements_by_name_from(&root, &name);
        Value::NodeList(Rc::new(RefCell::new(
            NodeListValue::live_descendants_by_name(root, name, nodes),
        )))
    }

    pub(crate) fn tag_name_from_argument(value: &Value) -> String {
        let raw = value.as_string();
        if raw == "*" {
            "*".to_string()
        } else {
            raw.to_ascii_lowercase()
        }
    }

    pub(crate) fn namespace_uri_from_create_element_ns_argument(value: &Value) -> Option<String> {
        if matches!(value, Value::Null) {
            return None;
        }
        let raw = value.as_string();
        if raw.is_empty() { None } else { Some(raw) }
    }

    pub(crate) fn tag_name_live_list_value(&self, root: NodeId, tag_name: String) -> Value {
        let nodes = self.dom.get_elements_by_tag_name_from(&root, &tag_name);
        Value::NodeList(Rc::new(RefCell::new(
            NodeListValue::live_descendants_by_tag_name(root, tag_name, nodes),
        )))
    }

    pub(crate) fn tag_name_ns_live_list_value(
        &self,
        root: NodeId,
        namespace_uri: Option<String>,
        local_name: String,
    ) -> Value {
        let nodes =
            self.dom
                .get_elements_by_tag_name_ns_from(&root, namespace_uri.as_deref(), &local_name);
        Value::NodeList(Rc::new(RefCell::new(
            NodeListValue::live_descendants_by_tag_name_ns(root, namespace_uri, local_name, nodes),
        )))
    }

    pub(crate) fn child_nodes_live_list_value(&mut self, parent: NodeId) -> Value {
        let existing = self
            .dom_runtime
            .live_child_nodes_lists
            .get(&parent)
            .cloned();
        let list = existing.unwrap_or_else(|| {
            let list = Rc::new(RefCell::new(NodeListValue::live_child_nodes(
                parent,
                self.dom.nodes[parent.0].children.clone(),
            )));
            self.dom_runtime
                .live_child_nodes_lists
                .insert(parent, list.clone());
            list
        });
        self.refresh_node_list(&list);
        Value::NodeList(list)
    }

    pub(crate) fn child_elements_live_list_value(&mut self, parent: NodeId) -> Value {
        let existing = self.dom_runtime.live_children_lists.get(&parent).cloned();
        let list = existing.unwrap_or_else(|| {
            let list = Rc::new(RefCell::new(NodeListValue::live_child_elements(
                parent,
                self.dom.child_elements(parent),
            )));
            self.dom_runtime
                .live_children_lists
                .insert(parent, list.clone());
            list
        });
        self.refresh_node_list(&list);
        Value::NodeList(list)
    }

    pub(crate) fn form_elements_live_list_value(&mut self, form: NodeId) -> Result<Value> {
        let existing = self
            .dom_runtime
            .live_form_elements_lists
            .get(&form)
            .cloned();
        let list = if let Some(existing) = existing {
            existing
        } else {
            let list = Rc::new(RefCell::new(NodeListValue::live_form_elements(
                form,
                self.form_elements(form)?,
            )));
            self.dom_runtime
                .live_form_elements_lists
                .insert(form, list.clone());
            list
        };
        self.refresh_node_list(&list);
        Ok(Value::NodeList(list))
    }

    pub(crate) fn form_named_group_live_list_value(
        &mut self,
        form: NodeId,
        name: &str,
    ) -> Result<Value> {
        let cache_key = (form, name.to_string());
        let existing = self
            .dom_runtime
            .live_form_named_group_lists
            .get(&cache_key)
            .cloned();
        let list = if let Some(existing) = existing {
            existing
        } else {
            let list = Rc::new(RefCell::new(NodeListValue::live_form_elements_named_group(
                form,
                name.to_string(),
                self.form_controls_named_matches(form, name)?,
            )));
            self.dom_runtime
                .live_form_named_group_lists
                .insert(cache_key, list.clone());
            list
        };
        self.refresh_node_list(&list);
        Ok(Value::NodeList(list))
    }

    pub(crate) fn select_options_live_list_value(&mut self, select: NodeId) -> Value {
        let existing = self
            .dom_runtime
            .live_select_options_lists
            .get(&select)
            .cloned();
        let list = existing.unwrap_or_else(|| {
            let list = Rc::new(RefCell::new(NodeListValue::live_select_options(
                select,
                self.select_option_nodes(select),
            )));
            self.dom_runtime
                .live_select_options_lists
                .insert(select, list.clone());
            list
        });
        self.refresh_node_list(&list);
        Value::NodeList(list)
    }

    pub(crate) fn selected_options_live_list_value(&mut self, select: NodeId) -> Value {
        let existing = self
            .dom_runtime
            .live_selected_options_lists
            .get(&select)
            .cloned();
        let list = existing.unwrap_or_else(|| {
            let list = Rc::new(RefCell::new(NodeListValue::live_selected_options(
                select,
                self.select_selected_option_nodes(select),
            )));
            self.dom_runtime
                .live_selected_options_lists
                .insert(select, list.clone());
            list
        });
        self.refresh_node_list(&list);
        Value::NodeList(list)
    }

    pub(crate) fn datalist_options_live_list_value(&mut self, datalist: NodeId) -> Value {
        let existing = self
            .dom_runtime
            .live_datalist_options_lists
            .get(&datalist)
            .cloned();
        let list = existing.unwrap_or_else(|| {
            let mut options = Vec::new();
            self.dom.collect_select_options(datalist, &mut options);
            let list = Rc::new(RefCell::new(NodeListValue::live_datalist_options(
                datalist, options,
            )));
            self.dom_runtime
                .live_datalist_options_lists
                .insert(datalist, list.clone());
            list
        });
        self.refresh_node_list(&list);
        Value::NodeList(list)
    }

    fn media_text_track_nodes(&self, media: NodeId) -> Vec<NodeId> {
        self.dom.nodes[media.0]
            .children
            .iter()
            .copied()
            .filter(|child| {
                self.dom
                    .tag_name(*child)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("track"))
            })
            .collect()
    }

    pub(crate) fn media_text_tracks_live_list_value(&mut self, media: NodeId) -> Value {
        let existing = self
            .dom_runtime
            .live_media_text_tracks_lists
            .get(&media)
            .cloned();
        let list = existing.unwrap_or_else(|| {
            let list = Rc::new(RefCell::new(NodeListValue::live_media_text_tracks(
                media,
                self.media_text_track_nodes(media),
            )));
            self.dom_runtime
                .live_media_text_tracks_lists
                .insert(media, list.clone());
            list
        });
        self.refresh_node_list(&list);
        Value::NodeList(list)
    }

    pub(crate) fn media_time_ranges_live_value(&mut self, media: NodeId, kind: &str) -> Value {
        let cache_key = (media, kind.to_string());
        let existing = self
            .dom_runtime
            .live_media_time_ranges_objects
            .get(&cache_key)
            .cloned();
        let object = existing.unwrap_or_else(|| {
            let Value::Object(object) = self.new_time_ranges_value(media, kind) else {
                unreachable!("new_time_ranges_value must return an object");
            };
            self.dom_runtime
                .live_media_time_ranges_objects
                .insert(cache_key, object.clone());
            object
        });
        Value::Object(object)
    }

    pub(crate) fn named_node_map_live_value(&mut self, owner: NodeId) -> Value {
        let existing = self.dom_runtime.live_named_node_maps.get(&owner).cloned();
        let map = existing.unwrap_or_else(|| {
            let named_node_map = self.new_named_node_map_value(owner);
            let Value::Object(object) = named_node_map else {
                unreachable!("new_named_node_map_value must return an object");
            };
            self.dom_runtime
                .live_named_node_maps
                .insert(owner, object.clone());
            object
        });
        Value::Object(map)
    }

    pub(crate) fn dom_string_map_live_value(&mut self, owner: NodeId) -> Value {
        let existing = self.dom_runtime.live_dom_string_maps.get(&owner).cloned();
        let map = existing.unwrap_or_else(|| {
            let dom_string_map = self.new_dom_string_map_value(owner);
            let Value::Object(object) = dom_string_map else {
                unreachable!("new_dom_string_map_value must return an object");
            };
            self.dom_runtime
                .live_dom_string_maps
                .insert(owner, object.clone());
            object
        });
        Value::Object(map)
    }

    pub(crate) fn class_list_live_value(&mut self, owner: NodeId) -> Value {
        let existing = self.dom_runtime.live_class_lists.get(&owner).cloned();
        let list = existing.unwrap_or_else(|| {
            let class_list = Self::new_class_list_value(owner);
            let Value::Object(object) = class_list else {
                unreachable!("new_class_list_value must return an object");
            };
            self.dom_runtime
                .live_class_lists
                .insert(owner, object.clone());
            object
        });
        Value::Object(list)
    }

    pub(crate) fn refresh_node_list(&self, list: &Rc<RefCell<NodeListValue>>) {
        let source = list.borrow().live_source.clone();
        let Some(source) = source else {
            return;
        };

        let nodes = match source {
            LiveNodeListSource::ChildNodes { parent } => {
                if self.dom.is_valid_node(parent) {
                    self.dom.nodes[parent.0].children.clone()
                } else {
                    Vec::new()
                }
            }
            LiveNodeListSource::ChildElements { parent } => {
                if self.dom.is_valid_node(parent) {
                    self.dom.child_elements(parent)
                } else {
                    Vec::new()
                }
            }
            LiveNodeListSource::FormElements { form } => {
                if self.dom.is_valid_node(form) {
                    self.form_elements(form).unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
            LiveNodeListSource::FormElementsNamedGroup { form, name } => {
                if self.dom.is_valid_node(form) {
                    self.form_controls_named_matches(form, name.as_str())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
            LiveNodeListSource::SelectOptions { select } => {
                if self.dom.is_valid_node(select) {
                    self.select_option_nodes(select)
                } else {
                    Vec::new()
                }
            }
            LiveNodeListSource::SelectedOptions { select } => {
                if self.dom.is_valid_node(select) {
                    self.select_selected_option_nodes(select)
                } else {
                    Vec::new()
                }
            }
            LiveNodeListSource::DataListOptions { datalist } => {
                if self.dom.is_valid_node(datalist) {
                    let mut options = Vec::new();
                    self.dom.collect_select_options(datalist, &mut options);
                    options
                } else {
                    Vec::new()
                }
            }
            LiveNodeListSource::MediaTextTracks { media } => {
                if self.dom.is_valid_node(media) {
                    self.media_text_track_nodes(media)
                } else {
                    Vec::new()
                }
            }
            LiveNodeListSource::DescendantsByClassNames { root, class_names } => {
                if !self.dom.is_valid_node(root) || class_names.is_empty() {
                    Vec::new()
                } else {
                    self.dom
                        .get_elements_by_class_names_from(&root, &class_names)
                }
            }
            LiveNodeListSource::DescendantsByName { root, name } => {
                if !self.dom.is_valid_node(root) {
                    Vec::new()
                } else {
                    self.dom.get_elements_by_name_from(&root, &name)
                }
            }
            LiveNodeListSource::DescendantsByTagName { root, tag_name } => {
                if !self.dom.is_valid_node(root) {
                    Vec::new()
                } else {
                    self.dom.get_elements_by_tag_name_from(&root, &tag_name)
                }
            }
            LiveNodeListSource::DescendantsByTagNameNs {
                root,
                namespace_uri,
                local_name,
            } => {
                if !self.dom.is_valid_node(root) {
                    Vec::new()
                } else {
                    self.dom.get_elements_by_tag_name_ns_from(
                        &root,
                        namespace_uri.as_deref(),
                        &local_name,
                    )
                }
            }
            LiveNodeListSource::QuerySelectorAll { root, selector } => {
                if !self.dom.is_valid_node(root) {
                    Vec::new()
                } else {
                    self.dom
                        .query_selector_all_from(&root, &selector)
                        .unwrap_or_default()
                }
            }
        };
        list.borrow_mut().nodes = nodes;
    }

    pub(crate) fn node_list_snapshot(&self, list: &Rc<RefCell<NodeListValue>>) -> Vec<NodeId> {
        self.refresh_node_list(list);
        list.borrow().nodes.clone()
    }

    pub(crate) fn node_list_len(&self, list: &Rc<RefCell<NodeListValue>>) -> usize {
        self.refresh_node_list(list);
        list.borrow().nodes.len()
    }

    pub(crate) fn node_list_get(
        &self,
        list: &Rc<RefCell<NodeListValue>>,
        index: usize,
    ) -> Option<NodeId> {
        self.refresh_node_list(list);
        list.borrow().nodes.get(index).copied()
    }
}
