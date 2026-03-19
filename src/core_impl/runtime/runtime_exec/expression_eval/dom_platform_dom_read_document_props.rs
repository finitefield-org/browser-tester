use super::*;

impl Harness {
    pub(crate) fn try_eval_dom_read_document_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
    ) -> Result<Option<Value>> {
        let value = match prop {
            DomProp::ShadowRoot => self.shadow_root_property_value(node),
            DomProp::ActiveElement => self.document_active_element_property_value(),
            DomProp::ActiveViewTransition => Value::Null,
            DomProp::AdoptedStyleSheets => self.ensure_document_adopted_style_sheets_property(),
            DomProp::AdoptedStyleSheetsLength => {
                let adopted = self.ensure_document_adopted_style_sheets_property();
                let len = match adopted {
                    Value::Array(values) => values.borrow().len() as i64,
                    _ => 0,
                };
                Value::Number(len)
            }
            DomProp::CharacterSet => Value::String("UTF-8".to_string()),
            DomProp::CompatMode => Value::String("CSS1Compat".to_string()),
            DomProp::ContentType => Value::String("text/html".to_string()),
            DomProp::ReadyState => Value::String(self.dom_runtime.document_ready_state.clone()),
            DomProp::Referrer => Value::String(String::new()),
            DomProp::Title => Value::String(self.dom.document_title()),
            DomProp::Url | DomProp::DocumentUri => Value::String(self.document_url.clone()),
            DomProp::BaseUri => Value::String(self.document_base_url()),
            DomProp::Location => Value::Object(self.dom_runtime.location_object.clone()),
            DomProp::LocationHref => Value::String(self.document_url.clone()),
            DomProp::LocationProtocol => Value::String(self.current_location_parts().protocol()),
            DomProp::LocationHost => Value::String(self.current_location_parts().host()),
            DomProp::LocationHostname => Value::String(self.current_location_parts().hostname),
            DomProp::LocationPort => Value::String(self.current_location_parts().effective_port()),
            DomProp::LocationPathname => {
                let parts = self.current_location_parts();
                Value::String(if parts.has_authority {
                    parts.pathname
                } else {
                    parts.opaque_path
                })
            }
            DomProp::LocationSearch => Value::String(self.current_location_parts().search),
            DomProp::LocationHash => Value::String(self.current_location_parts().hash),
            DomProp::LocationOrigin => Value::String(self.current_location_parts().origin()),
            DomProp::LocationAncestorOrigins => Self::new_array_value(Vec::new()),
            DomProp::History => Value::Object(self.location_history.history_object.clone()),
            DomProp::HistoryLength => {
                Value::Number(self.location_history.history_entries.len() as i64)
            }
            DomProp::HistoryState => self.current_history_state(),
            DomProp::HistoryScrollRestoration => {
                Value::String(self.location_history.history_scroll_restoration.clone())
            }
            DomProp::DefaultView => Value::Object(self.dom_runtime.window_object.clone()),
            DomProp::Hidden => {
                if node == self.dom.root {
                    Value::Bool(self.dom_runtime.document_visibility_state == "hidden")
                } else if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["hidden"])?
                {
                    value
                } else {
                    Value::Bool(self.dom.attr(node, "hidden").is_some())
                }
            }
            DomProp::VisibilityState => {
                Value::String(self.dom_runtime.document_visibility_state.clone())
            }
            DomProp::Forms => self.document_forms_live_list_value(),
            DomProp::Images => self.document_images_live_list_value(),
            DomProp::Links => self.document_links_live_list_value(),
            DomProp::Scripts => self.document_scripts_live_list_value(),
            DomProp::Children => self.child_elements_live_list_value(node),
            DomProp::ChildElementCount => Value::Number(self.dom.child_element_count(node) as i64),
            DomProp::FirstElementChild => self
                .dom
                .first_element_child(node)
                .map(Value::Node)
                .unwrap_or(Value::Null),
            DomProp::LastElementChild => self
                .dom
                .last_element_child(node)
                .map(Value::Node)
                .unwrap_or(Value::Null),
            DomProp::CurrentScript => Value::Null,
            DomProp::FormsLength => {
                let list = self.document_forms_live_list_value();
                return self
                    .object_property_from_value_with_receiver(&list, "length", &list)
                    .map(Some);
            }
            DomProp::ImagesLength => {
                let list = self.document_images_live_list_value();
                return self
                    .object_property_from_value_with_receiver(&list, "length", &list)
                    .map(Some);
            }
            DomProp::LinksLength => {
                let list = self.document_links_live_list_value();
                return self
                    .object_property_from_value_with_receiver(&list, "length", &list)
                    .map(Some);
            }
            DomProp::ScriptsLength => {
                let list = self.document_scripts_live_list_value();
                return self
                    .object_property_from_value_with_receiver(&list, "length", &list)
                    .map(Some);
            }
            DomProp::ChildrenLength => {
                let list = self.child_elements_live_list_value(node);
                return self
                    .object_property_from_value_with_receiver(&list, "length", &list)
                    .map(Some);
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
