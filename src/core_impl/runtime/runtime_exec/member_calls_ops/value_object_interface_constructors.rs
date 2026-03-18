use super::*;

impl Harness {
    fn cached_collection_like_constructor_value(
        &mut self,
        name: &str,
        callable_kind: &str,
        family: &str,
        methods: &[&str],
        prototype_parent: Option<Value>,
    ) -> Value {
        let iterator_symbol = self.eval_symbol_static_property(SymbolStaticProperty::Iterator);
        let iterator_key = self.property_key_to_storage_key(&iterator_symbol);
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let prototype_parent =
            prototype_parent.unwrap_or_else(|| self.object_constructor_prototype_value());
        self.cached_constructor_static_method_value(name, || {
            let constructor =
                Self::new_receiver_builtin_constructor_object(Some(callable_kind), family, methods);
            let Value::Object(constructor_entries) = &constructor else {
                return constructor;
            };
            let prototype = {
                let entries = constructor_entries.borrow();
                Self::object_get_entry(&entries, "prototype")
            };
            let Some(Value::Object(prototype)) = prototype else {
                return constructor;
            };
            if !methods.is_empty() {
                Self::object_set_entry(
                    &mut prototype.borrow_mut(),
                    iterator_key.clone(),
                    Self::new_receiver_builtin_callable(family, "values"),
                );
            }
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                to_string_tag_key.clone(),
                Value::String(name.to_string()),
            );
            Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
            Self::set_internal_prototype(&prototype, prototype_parent.clone());
            constructor
        })
    }

    pub(crate) fn cached_node_list_constructor_value(&mut self) -> Value {
        self.cached_collection_like_constructor_value(
            "NodeList",
            "node_list_constructor",
            "node_list",
            &["item", "forEach", "entries", "keys", "values"],
            None,
        )
    }

    pub(crate) fn cached_node_list_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("NodeList")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let constructor = self.cached_node_list_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("NodeList".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_text_track_list_constructor_value(&mut self) -> Value {
        let parent = self.cached_node_list_constructor_prototype_value();
        self.cached_collection_like_constructor_value(
            "TextTrackList",
            "text_track_list_constructor",
            "node_list",
            &[],
            Some(parent),
        )
    }

    pub(crate) fn cached_text_track_list_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("TextTrackList")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let constructor = self.cached_text_track_list_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("TextTrackList".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_text_track_constructor_value(&mut self) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get("TextTrack")
            .cloned()
        {
            if let Value::Object(entries) = &constructor {
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    self.install_text_track_prototype_accessors(&prototype);
                    Self::set_internal_prototype(
                        &prototype,
                        self.object_constructor_prototype_value(),
                    );
                }
            }
            return constructor;
        }
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("text_track_constructor"),
            "text_track",
            &[],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        self.install_text_track_prototype_accessors(&prototype);
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String("TextTrack".to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime
            .constructor_static_methods
            .insert("TextTrack".to_string(), constructor.clone());
        constructor
    }

    fn install_text_track_prototype_accessors(&mut self, prototype: &Rc<RefCell<ObjectValue>>) {
        let has_mode_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "mode")
        };
        if has_mode_accessor {
            return;
        }
        for (property, getter) in [
            ("id", "id_get"),
            ("kind", "kind_get"),
            ("label", "label_get"),
            ("language", "language_get"),
            ("cues", "cues_get"),
            ("activeCues", "active_cues_get"),
            (
                "inBandMetadataTrackDispatchType",
                "in_band_metadata_track_dispatch_type_get",
            ),
        ] {
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                Self::object_getter_storage_key(property),
                Self::new_receiver_builtin_callable("text_track", getter),
            );
            Self::mark_property_non_enumerable(prototype, property);
        }
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            Self::object_getter_storage_key("mode"),
            Self::new_receiver_builtin_callable("text_track", "mode_get"),
        );
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            Self::object_setter_storage_key("mode"),
            Self::new_receiver_builtin_callable("text_track", "mode_set"),
        );
        Self::mark_property_non_enumerable(prototype, "mode");
    }

    pub(crate) fn cached_text_track_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("TextTrack")
            .cloned()
        {
            self.install_text_track_prototype_accessors(&prototype);
            Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
            return Value::Object(prototype);
        }
        let constructor = self.cached_text_track_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.install_text_track_prototype_accessors(&prototype);
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("TextTrack".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_image_bitmap_constructor_value(&mut self) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get("ImageBitmap")
            .cloned()
        {
            if let Value::Object(entries) = &constructor {
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    self.install_image_bitmap_prototype_accessors(&prototype);
                    Self::set_internal_prototype(
                        &prototype,
                        self.object_constructor_prototype_value(),
                    );
                }
            }
            return constructor;
        }
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("image_bitmap_constructor"),
            "image_bitmap",
            &["close"],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        self.install_image_bitmap_prototype_accessors(&prototype);
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String("ImageBitmap".to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime
            .constructor_static_methods
            .insert("ImageBitmap".to_string(), constructor.clone());
        constructor
    }

    fn install_image_bitmap_prototype_accessors(&mut self, prototype: &Rc<RefCell<ObjectValue>>) {
        let has_width_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "width")
        };
        if has_width_accessor {
            return;
        }
        for property in ["width", "height"] {
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                Self::object_getter_storage_key(property),
                Self::new_receiver_builtin_callable(
                    "image_bitmap",
                    if property == "width" {
                        "width_get"
                    } else {
                        "height_get"
                    },
                ),
            );
            Self::mark_property_non_enumerable(prototype, property);
        }
    }

    pub(crate) fn cached_image_bitmap_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("ImageBitmap")
            .cloned()
        {
            self.install_image_bitmap_prototype_accessors(&prototype);
            Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
            return Value::Object(prototype);
        }
        let constructor = self.cached_image_bitmap_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.install_image_bitmap_prototype_accessors(&prototype);
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("ImageBitmap".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_time_ranges_constructor_value(&mut self) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get("TimeRanges")
            .cloned()
        {
            if let Value::Object(entries) = &constructor {
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    self.install_time_ranges_prototype_accessors(&prototype);
                    Self::set_internal_prototype(
                        &prototype,
                        self.object_constructor_prototype_value(),
                    );
                }
            }
            return constructor;
        }
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("time_ranges_constructor"),
            "time_ranges",
            &["start", "end"],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        self.install_time_ranges_prototype_accessors(&prototype);
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String("TimeRanges".to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime
            .constructor_static_methods
            .insert("TimeRanges".to_string(), constructor.clone());
        constructor
    }

    fn install_time_ranges_prototype_accessors(&mut self, prototype: &Rc<RefCell<ObjectValue>>) {
        let has_length_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "length")
        };
        if has_length_accessor {
            return;
        }
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            Self::object_getter_storage_key("length"),
            Self::new_receiver_builtin_callable("time_ranges", "length_get"),
        );
        Self::mark_property_non_enumerable(prototype, "length");
    }

    pub(crate) fn cached_time_ranges_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("TimeRanges")
            .cloned()
        {
            self.install_time_ranges_prototype_accessors(&prototype);
            Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
            return Value::Object(prototype);
        }
        let constructor = self.cached_time_ranges_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.install_time_ranges_prototype_accessors(&prototype);
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("TimeRanges".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_html_collection_constructor_value(&mut self) -> Value {
        self.cached_collection_like_constructor_value(
            "HTMLCollection",
            "html_collection_constructor",
            "html_collection",
            &["item", "namedItem", "forEach", "entries", "keys", "values"],
            None,
        )
    }

    pub(crate) fn cached_html_collection_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("HTMLCollection")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let constructor = self.cached_html_collection_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("HTMLCollection".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_radio_node_list_constructor_value(&mut self) -> Value {
        let parent = self.cached_node_list_constructor_prototype_value();
        let constructor = self.cached_collection_like_constructor_value(
            "RadioNodeList",
            "radio_node_list_constructor",
            "node_list",
            &["item", "forEach", "entries", "keys", "values"],
            Some(parent),
        );
        if let Value::Object(entries) = &constructor {
            let prototype = {
                let entries = entries.borrow();
                Self::object_get_entry(&entries, "prototype")
            };
            if let Some(Value::Object(prototype)) = prototype {
                self.install_radio_node_list_prototype_accessors(&prototype);
            }
        }
        constructor
    }

    fn install_radio_node_list_prototype_accessors(
        &mut self,
        prototype: &Rc<RefCell<ObjectValue>>,
    ) {
        let has_value_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "value")
        };
        if has_value_accessor {
            return;
        }
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            Self::object_getter_storage_key("value"),
            Self::new_receiver_builtin_callable("radio_node_list", "value_get"),
        );
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            Self::object_setter_storage_key("value"),
            Self::new_receiver_builtin_callable("radio_node_list", "value_set"),
        );
        Self::mark_property_non_enumerable(prototype, "value");
    }

    pub(crate) fn cached_radio_node_list_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("RadioNodeList")
            .cloned()
        {
            self.install_radio_node_list_prototype_accessors(&prototype);
            return Value::Object(prototype);
        }
        let constructor = self.cached_radio_node_list_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.install_radio_node_list_prototype_accessors(&prototype);
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("RadioNodeList".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_html_form_controls_collection_constructor_value(&mut self) -> Value {
        let parent = self.cached_html_collection_constructor_prototype_value();
        self.cached_collection_like_constructor_value(
            "HTMLFormControlsCollection",
            "html_form_controls_collection_constructor",
            "html_collection",
            &[],
            Some(parent),
        )
    }

    pub(crate) fn cached_html_form_controls_collection_constructor_prototype_value(
        &mut self,
    ) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("HTMLFormControlsCollection")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let constructor = self.cached_html_form_controls_collection_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("HTMLFormControlsCollection".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_html_options_collection_constructor_value(&mut self) -> Value {
        let parent = self.cached_html_collection_constructor_prototype_value();
        self.cached_collection_like_constructor_value(
            "HTMLOptionsCollection",
            "html_options_collection_constructor",
            "html_collection",
            &[],
            Some(parent),
        )
    }

    pub(crate) fn cached_html_options_collection_constructor_prototype_value(
        &mut self,
    ) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("HTMLOptionsCollection")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let constructor = self.cached_html_options_collection_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("HTMLOptionsCollection".to_string(), prototype.clone());
        Value::Object(prototype)
    }
}
