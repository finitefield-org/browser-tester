use super::*;

impl Harness {
    pub(crate) fn is_history_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_HISTORY_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_navigation_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_NAVIGATION_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_window_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_WINDOW_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_navigator_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_NAVIGATOR_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_document_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_DOCUMENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_storage_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_STORAGE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn set_navigator_property(
        &mut self,
        navigator_object: &Rc<RefCell<ObjectValue>>,
        key: &str,
        value: Value,
    ) -> Result<()> {
        Self::object_set_entry(&mut navigator_object.borrow_mut(), key.to_string(), value);
        Ok(())
    }

    pub(crate) fn set_window_property(&mut self, key: &str, value: Value) -> Result<()> {
        match key {
            "window"
            | "self"
            | "top"
            | "parent"
            | "frames"
            | "length"
            | "closed"
            | "close"
            | "stop"
            | "focus"
            | "scroll"
            | "scrollBy"
            | "scrollTo"
            | "moveBy"
            | "moveTo"
            | "resizeBy"
            | "resizeTo"
            | "postMessage"
            | "history"
            | "navigation"
            | "navigator"
            | "clientInformation"
            | "document"
            | "origin"
            | "isSecureContext"
            | "cookieStore"
            | "caches"
            | "fetch"
            | "getComputedStyle"
            | "alert"
            | "confirm"
            | "prompt"
            | "print"
            | "reportError"
            | "atob"
            | "btoa"
            | "structuredClone"
            | "requestAnimationFrame"
            | "setTimeout"
            | "setInterval"
            | "cancelAnimationFrame"
            | "clearInterval"
            | "clearTimeout"
            | "queueMicrotask"
            | "screenX"
            | "screenY"
            | "screenLeft"
            | "screenTop"
            | "scrollX"
            | "scrollY"
            | "pageXOffset"
            | "pageYOffset"
            | "Request"
            | "Headers"
            | "URL"
            | "Object"
            | "Reflect"
            | "Element"
            | "Audio"
            | "DataTransfer"
            | "Option"
            | "HTMLElement"
            | "HTMLAnchorElement"
            | "HTMLAreaElement"
            | "HTMLBodyElement"
            | "HTMLBRElement"
            | "HTMLBaseElement"
            | "HTMLAudioElement"
            | "HTMLButtonElement"
            | "HTMLCanvasElement"
            | "HTMLDataElement"
            | "HTMLDataListElement"
            | "HTMLInputElement"
            | "HTMLOptionElement"
            | "HTMLSelectElement"
            | "DOMParser"
            | "XMLSerializer"
            | "Document"
            | "Node"
            | "NodeFilter"
            | "getSelection" => Err(Error::ScriptRuntime(format!("window.{key} is read-only"))),
            "location" => self.set_location_property("href", value),
            "localStorage" => {
                Self::object_set_entry(
                    &mut self.dom_runtime.window_object.borrow_mut(),
                    "localStorage".to_string(),
                    value.clone(),
                );
                self.script_runtime
                    .env
                    .insert("localStorage".to_string(), value);
                Ok(())
            }
            "name" => {
                Self::object_set_entry(
                    &mut self.dom_runtime.window_object.borrow_mut(),
                    "name".to_string(),
                    Value::String(value.as_string()),
                );
                Ok(())
            }
            _ => {
                Self::object_set_entry(
                    &mut self.dom_runtime.window_object.borrow_mut(),
                    key.to_string(),
                    value,
                );
                Ok(())
            }
        }
    }

    pub(crate) fn set_url_constructor_property(&mut self, key: &str, value: Value) {
        Self::object_set_entry(
            &mut self.browser_apis.url_constructor_properties.borrow_mut(),
            key.to_string(),
            value,
        );
    }

    pub(crate) fn set_storage_object_property(
        &mut self,
        storage_object: &Rc<RefCell<ObjectValue>>,
        key: &str,
        value: Value,
    ) -> Result<()> {
        match key {
            "length" => Err(Error::ScriptRuntime("Storage.length is read-only".into())),
            "getItem" | "setItem" | "removeItem" | "clear" | "key" => {
                Self::object_set_entry(&mut storage_object.borrow_mut(), key.to_string(), value);
                Ok(())
            }
            _ => {
                let mut entries = storage_object.borrow_mut();
                let mut pairs = Self::storage_pairs_from_object_entries(&entries);
                if let Some((_, stored)) = pairs.iter_mut().find(|(name, _)| name == key) {
                    *stored = value.as_string();
                } else {
                    pairs.push((key.to_string(), value.as_string()));
                }
                Self::set_storage_pairs(&mut entries, &pairs);
                Ok(())
            }
        }
    }

    pub(crate) fn set_data_transfer_object_property(
        &mut self,
        data_transfer_object: &Rc<RefCell<ObjectValue>>,
        key: &str,
        value: Value,
    ) -> Result<()> {
        let key_lower = key.to_ascii_lowercase();
        match key_lower.as_str() {
            "dropeffect" => {
                let next = match value.as_string().to_ascii_lowercase().as_str() {
                    "none" | "copy" | "link" | "move" => value.as_string().to_ascii_lowercase(),
                    _ => "none".to_string(),
                };
                Self::object_set_entry(
                    &mut data_transfer_object.borrow_mut(),
                    "dropEffect".to_string(),
                    Value::String(next),
                );
                Ok(())
            }
            "effectallowed" => {
                let normalized = match value.as_string().to_ascii_lowercase().as_str() {
                    "none" => Some("none"),
                    "copy" => Some("copy"),
                    "copylink" => Some("copyLink"),
                    "copymove" => Some("copyMove"),
                    "link" => Some("link"),
                    "linkmove" => Some("linkMove"),
                    "move" => Some("move"),
                    "all" => Some("all"),
                    "uninitialized" => Some("uninitialized"),
                    _ => None,
                };
                if let Some(next) = normalized {
                    Self::object_set_entry(
                        &mut data_transfer_object.borrow_mut(),
                        "effectAllowed".to_string(),
                        Value::String(next.to_string()),
                    );
                }
                Ok(())
            }
            "files" | "items" | "types" => Ok(()),
            _ => {
                Self::object_set_entry(
                    &mut data_transfer_object.borrow_mut(),
                    key.to_string(),
                    value,
                );
                Ok(())
            }
        }
    }

    pub(crate) fn set_canvas_2d_context_property(
        &mut self,
        context_object: &Rc<RefCell<ObjectValue>>,
        key: &str,
        value: Value,
    ) -> Result<()> {
        match key {
            "canvas" => Ok(()),
            "lineWidth" => {
                let next = Self::coerce_number_for_number_constructor(&value);
                if next.is_finite() && next > 0.0 {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "lineWidth".to_string(),
                        Self::number_value(next),
                    );
                }
                Ok(())
            }
            "miterLimit" => {
                let next = Self::coerce_number_for_number_constructor(&value);
                if next.is_finite() && next > 0.0 {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "miterLimit".to_string(),
                        Self::number_value(next),
                    );
                }
                Ok(())
            }
            "lineDashOffset" => {
                let next = Self::coerce_number_for_number_constructor(&value);
                if next.is_finite() {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "lineDashOffset".to_string(),
                        Self::number_value(next),
                    );
                }
                Ok(())
            }
            "globalAlpha" => {
                let next = Self::coerce_number_for_number_constructor(&value);
                if next.is_finite() {
                    let next = next.clamp(0.0, 1.0);
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "globalAlpha".to_string(),
                        Self::number_value(next),
                    );
                }
                Ok(())
            }
            "lineCap" => {
                let next = value.as_string().to_ascii_lowercase();
                if matches!(next.as_str(), "butt" | "round" | "square") {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "lineCap".to_string(),
                        Value::String(next),
                    );
                }
                Ok(())
            }
            "lineJoin" => {
                let next = value.as_string().to_ascii_lowercase();
                if matches!(next.as_str(), "round" | "bevel" | "miter") {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "lineJoin".to_string(),
                        Value::String(next),
                    );
                }
                Ok(())
            }
            "textAlign" => {
                let next = value.as_string().to_ascii_lowercase();
                if matches!(next.as_str(), "start" | "end" | "left" | "right" | "center") {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "textAlign".to_string(),
                        Value::String(next),
                    );
                }
                Ok(())
            }
            "textBaseline" => {
                let next = value.as_string().to_ascii_lowercase();
                if matches!(
                    next.as_str(),
                    "top" | "hanging" | "middle" | "alphabetic" | "ideographic" | "bottom"
                ) {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "textBaseline".to_string(),
                        Value::String(next),
                    );
                }
                Ok(())
            }
            "direction" => {
                let next = value.as_string().to_ascii_lowercase();
                if matches!(next.as_str(), "ltr" | "rtl" | "inherit") {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "direction".to_string(),
                        Value::String(next),
                    );
                }
                Ok(())
            }
            "fontKerning" => {
                let next = value.as_string().to_ascii_lowercase();
                if matches!(next.as_str(), "auto" | "normal" | "none") {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "fontKerning".to_string(),
                        Value::String(next),
                    );
                }
                Ok(())
            }
            "fontStretch" => {
                let next = value.as_string().to_ascii_lowercase();
                if matches!(
                    next.as_str(),
                    "ultra-condensed"
                        | "extra-condensed"
                        | "condensed"
                        | "semi-condensed"
                        | "normal"
                        | "semi-expanded"
                        | "expanded"
                        | "extra-expanded"
                        | "ultra-expanded"
                ) {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "fontStretch".to_string(),
                        Value::String(next),
                    );
                }
                Ok(())
            }
            "fontVariantCaps" => {
                let next = value.as_string().to_ascii_lowercase();
                if matches!(
                    next.as_str(),
                    "normal"
                        | "small-caps"
                        | "all-small-caps"
                        | "petite-caps"
                        | "all-petite-caps"
                        | "unicase"
                        | "titling-caps"
                ) {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "fontVariantCaps".to_string(),
                        Value::String(next),
                    );
                }
                Ok(())
            }
            "textRendering" => {
                let next = value.as_string();
                if matches!(
                    next.as_str(),
                    "auto" | "optimizeSpeed" | "optimizeLegibility" | "geometricPrecision"
                ) {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "textRendering".to_string(),
                        Value::String(next),
                    );
                }
                Ok(())
            }
            "globalCompositeOperation" => {
                let next = value.as_string();
                if matches!(
                    next.as_str(),
                    "source-over"
                        | "source-in"
                        | "source-out"
                        | "source-atop"
                        | "destination-over"
                        | "destination-in"
                        | "destination-out"
                        | "destination-atop"
                        | "lighter"
                        | "copy"
                        | "xor"
                        | "multiply"
                        | "screen"
                        | "overlay"
                        | "darken"
                        | "lighten"
                        | "color-dodge"
                        | "color-burn"
                        | "hard-light"
                        | "soft-light"
                        | "difference"
                        | "exclusion"
                        | "hue"
                        | "saturation"
                        | "color"
                        | "luminosity"
                ) {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "globalCompositeOperation".to_string(),
                        Value::String(next),
                    );
                }
                Ok(())
            }
            "imageSmoothingEnabled" => {
                Self::object_set_entry(
                    &mut context_object.borrow_mut(),
                    "imageSmoothingEnabled".to_string(),
                    Value::Bool(value.truthy()),
                );
                Ok(())
            }
            "imageSmoothingQuality" => {
                let next = value.as_string().to_ascii_lowercase();
                if matches!(next.as_str(), "low" | "medium" | "high") {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        "imageSmoothingQuality".to_string(),
                        Value::String(next),
                    );
                }
                Ok(())
            }
            "fillStyle" | "strokeStyle" | "font" | "letterSpacing" | "wordSpacing"
            | "shadowColor" | "filter" | "lang" => {
                Self::object_set_entry(&mut context_object.borrow_mut(), key.to_string(), value);
                Ok(())
            }
            "shadowBlur" | "shadowOffsetX" | "shadowOffsetY" => {
                let next = Self::coerce_number_for_number_constructor(&value);
                if next.is_finite() {
                    Self::object_set_entry(
                        &mut context_object.borrow_mut(),
                        key.to_string(),
                        Self::number_value(next),
                    );
                }
                Ok(())
            }
            _ => {
                Self::object_set_entry(&mut context_object.borrow_mut(), key.to_string(), value);
                Ok(())
            }
        }
    }

    pub(crate) fn set_history_property(&mut self, key: &str, value: Value) -> Result<()> {
        match key {
            "length" => Err(Error::ScriptRuntime("history.length is read-only".into())),
            "state" => Err(Error::ScriptRuntime("history.state is read-only".into())),
            "scrollRestoration" => {
                let mode = value.as_string();
                if mode != "auto" && mode != "manual" {
                    return Err(Error::ScriptRuntime(
                        "history.scrollRestoration must be 'auto' or 'manual'".into(),
                    ));
                }
                self.location_history.history_scroll_restoration = mode;
                self.sync_history_object();
                self.sync_window_runtime_properties();
                Ok(())
            }
            _ => {
                Self::object_set_entry(
                    &mut self.location_history.history_object.borrow_mut(),
                    key.to_string(),
                    value,
                );
                Ok(())
            }
        }
    }

    pub(crate) fn set_navigation_property(&mut self, key: &str, value: Value) -> Result<()> {
        match key {
            "activation" | "canGoBack" | "canGoForward" | "currentEntry" | "transition" => Err(
                Error::ScriptRuntime(format!("navigation.{key} is read-only")),
            ),
            _ => {
                Self::object_set_entry(
                    &mut self.location_history.navigation_object.borrow_mut(),
                    key.to_string(),
                    value,
                );
                Ok(())
            }
        }
    }

    pub(crate) fn set_document_property(
        &mut self,
        document_object: &Rc<RefCell<ObjectValue>>,
        key: &str,
        value: Value,
    ) -> Result<()> {
        if self.set_node_event_handler_property(self.dom.root, key, value.clone())? {
            return Ok(());
        }

        match key {
            "cookie" => {
                let raw = value.as_string();
                let _ = self.set_cookie_from_document_assignment(&raw);
                self.sync_document_cookie_property();
                Ok(())
            }
            "adoptedStyleSheets" => self.set_document_adopted_style_sheets_property(value),
            "head" => Ok(()),
            "textContent" => Ok(()),
            _ => {
                Self::object_set_entry(&mut document_object.borrow_mut(), key.to_string(), value);
                Ok(())
            }
        }
    }
}
