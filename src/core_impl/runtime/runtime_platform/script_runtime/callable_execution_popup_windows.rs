use super::*;

impl Harness {
    pub(crate) fn window_open_target_url(&self, args: &[Value]) -> String {
        let requested = args.first().map(Value::as_string).unwrap_or_default();
        if requested.trim().is_empty() {
            "about:blank".to_string()
        } else {
            self.resolve_document_target_url(&requested)
        }
    }

    fn window_open_disables_opener(features: &str) -> bool {
        features.split(',').any(|raw_feature| {
            let feature = raw_feature.trim();
            if feature.is_empty() {
                return false;
            }
            let mut parts = feature.splitn(2, '=');
            let name = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
            let value = parts.next().map(|value| value.trim().to_ascii_lowercase());
            match name.as_str() {
                "noopener" | "noreferrer" => {
                    !matches!(value.as_deref(), Some("0") | Some("false") | Some("no"))
                }
                _ => false,
            }
        })
    }

    pub(crate) fn new_popup_window_value(&self, url: &str, target: &str, features: &str) -> Value {
        let popup_window = Rc::new(RefCell::new(ObjectValue::default()));
        let popup_document = Rc::new(RefCell::new(ObjectValue::default()));
        let popup_window_value = Value::Object(popup_window.clone());
        let popup_document_value = Value::Object(popup_document.clone());
        let opener = if Self::window_open_disables_opener(features) {
            Value::Null
        } else {
            Value::Object(self.dom_runtime.window_object.clone())
        };
        let popup_location =
            Self::new_object_value(vec![("href".to_string(), Value::String(url.to_string()))]);

        {
            let mut document_entries = popup_document.borrow_mut();
            Self::object_set_entry(
                &mut document_entries,
                INTERNAL_POPUP_DOCUMENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            );
            Self::object_set_entry(
                &mut document_entries,
                INTERNAL_POPUP_DOCUMENT_HTML_KEY.to_string(),
                Value::String(String::new()),
            );
            Self::object_set_entry(
                &mut document_entries,
                "defaultView".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut document_entries,
                "URL".to_string(),
                Value::String(url.to_string()),
            );
            Self::object_set_entry(
                &mut document_entries,
                "baseURI".to_string(),
                Value::String(url.to_string()),
            );
            Self::object_set_entry(
                &mut document_entries,
                "readyState".to_string(),
                Value::String("complete".to_string()),
            );
            Self::object_set_entry(
                &mut document_entries,
                "open".to_string(),
                Self::new_popup_document_open_callable_value(),
            );
            Self::object_set_entry(
                &mut document_entries,
                "write".to_string(),
                Self::new_popup_document_write_callable_value(),
            );
            Self::object_set_entry(
                &mut document_entries,
                "close".to_string(),
                Self::new_popup_document_close_callable_value(),
            );
        }

        {
            let mut window_entries = popup_window.borrow_mut();
            Self::object_set_entry(
                &mut window_entries,
                INTERNAL_POPUP_WINDOW_OBJECT_KEY.to_string(),
                Value::Bool(true),
            );
            Self::object_set_entry(
                &mut window_entries,
                "window".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "globalThis".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "self".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "top".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "parent".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "frames".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "closed".to_string(),
                Value::Bool(false),
            );
            Self::object_set_entry(
                &mut window_entries,
                "name".to_string(),
                Value::String(target.to_string()),
            );
            Self::object_set_entry(&mut window_entries, "opener".to_string(), opener);
            Self::object_set_entry(&mut window_entries, "location".to_string(), popup_location);
            Self::object_set_entry(
                &mut window_entries,
                "document".to_string(),
                popup_document_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "close".to_string(),
                Self::new_popup_window_close_callable_value(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "focus".to_string(),
                Self::new_popup_window_focus_callable_value(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "print".to_string(),
                Self::new_popup_window_print_callable_value(),
            );
        }

        popup_window_value
    }
}
