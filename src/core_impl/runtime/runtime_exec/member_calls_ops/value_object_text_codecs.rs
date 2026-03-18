use super::*;

impl Harness {
    pub(crate) fn new_text_encoder_encode_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_encode".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_encode_into_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_encode_into".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_encoding_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_get_encoding".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_decode_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_decode".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_encoding_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_get_encoding".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_fatal_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_get_fatal".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_ignore_bom_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_get_ignore_bom".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_stream_encoding_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_stream_get_encoding".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_stream_readable_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_stream_get_readable".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_stream_writable_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_stream_get_writable".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_stream_encoding_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_stream_get_encoding".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_stream_fatal_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_stream_get_fatal".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_stream_ignore_bom_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_stream_get_ignore_bom".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_stream_readable_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_stream_get_readable".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_stream_writable_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_stream_get_writable".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_instance_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_TEXT_ENCODER_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )])
    }

    pub(crate) fn new_text_decoder_instance_value(
        encoding: &str,
        fatal: bool,
        ignore_bom: bool,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_TEXT_DECODER_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_TEXT_DECODER_ENCODING_KEY.to_string(),
                Value::String(encoding.to_string()),
            ),
            (
                INTERNAL_TEXT_DECODER_FATAL_KEY.to_string(),
                Value::Bool(fatal),
            ),
            (
                INTERNAL_TEXT_DECODER_IGNORE_BOM_KEY.to_string(),
                Value::Bool(ignore_bom),
            ),
        ])
    }

    pub(crate) fn new_text_encoder_stream_instance_value(
        readable: Value,
        writable: Value,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_TEXT_ENCODER_STREAM_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_TEXT_ENCODER_STREAM_READABLE_KEY.to_string(),
                readable,
            ),
            (
                INTERNAL_TEXT_ENCODER_STREAM_WRITABLE_KEY.to_string(),
                writable,
            ),
        ])
    }

    pub(crate) fn new_text_decoder_stream_instance_value(
        encoding: &str,
        fatal: bool,
        ignore_bom: bool,
        readable: Value,
        writable: Value,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_TEXT_DECODER_STREAM_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_TEXT_DECODER_STREAM_ENCODING_KEY.to_string(),
                Value::String(encoding.to_string()),
            ),
            (
                INTERNAL_TEXT_DECODER_STREAM_FATAL_KEY.to_string(),
                Value::Bool(fatal),
            ),
            (
                INTERNAL_TEXT_DECODER_STREAM_IGNORE_BOM_KEY.to_string(),
                Value::Bool(ignore_bom),
            ),
            (
                INTERNAL_TEXT_DECODER_STREAM_READABLE_KEY.to_string(),
                readable,
            ),
            (
                INTERNAL_TEXT_DECODER_STREAM_WRITABLE_KEY.to_string(),
                writable,
            ),
        ])
    }

    fn install_text_encoder_prototype_surface(&mut self, prototype: &Rc<RefCell<ObjectValue>>) {
        let has_encoding_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "encoding")
        };
        if !has_encoding_accessor {
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                Self::object_getter_storage_key("encoding"),
                Self::new_text_encoder_encoding_getter_callable(),
            );
            Self::mark_property_non_enumerable(prototype, "encoding");
        }
        for (name, callable) in [
            ("encode", Self::new_text_encoder_encode_callable()),
            ("encodeInto", Self::new_text_encoder_encode_into_callable()),
        ] {
            Self::object_set_entry(&mut prototype.borrow_mut(), name.to_string(), callable);
            Self::mark_property_non_enumerable(prototype, name);
        }
    }

    fn install_text_decoder_prototype_surface(&mut self, prototype: &Rc<RefCell<ObjectValue>>) {
        let has_encoding_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "encoding")
        };
        if !has_encoding_accessor {
            for (property, getter) in [
                (
                    "encoding",
                    Self::new_text_decoder_encoding_getter_callable(),
                ),
                ("fatal", Self::new_text_decoder_fatal_getter_callable()),
                (
                    "ignoreBOM",
                    Self::new_text_decoder_ignore_bom_getter_callable(),
                ),
            ] {
                Self::object_set_entry(
                    &mut prototype.borrow_mut(),
                    Self::object_getter_storage_key(property),
                    getter,
                );
                Self::mark_property_non_enumerable(prototype, property);
            }
        }
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            "decode".to_string(),
            Self::new_text_decoder_decode_callable(),
        );
        Self::mark_property_non_enumerable(prototype, "decode");
    }

    fn install_text_encoder_stream_prototype_surface(
        &mut self,
        prototype: &Rc<RefCell<ObjectValue>>,
    ) {
        let has_encoding_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "encoding")
        };
        if has_encoding_accessor {
            return;
        }
        for (property, getter) in [
            (
                "encoding",
                Self::new_text_encoder_stream_encoding_getter_callable(),
            ),
            (
                "readable",
                Self::new_text_encoder_stream_readable_getter_callable(),
            ),
            (
                "writable",
                Self::new_text_encoder_stream_writable_getter_callable(),
            ),
        ] {
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                Self::object_getter_storage_key(property),
                getter,
            );
            Self::mark_property_non_enumerable(prototype, property);
        }
    }

    fn install_text_decoder_stream_prototype_surface(
        &mut self,
        prototype: &Rc<RefCell<ObjectValue>>,
    ) {
        let has_encoding_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "encoding")
        };
        if has_encoding_accessor {
            return;
        }
        for (property, getter) in [
            (
                "encoding",
                Self::new_text_decoder_stream_encoding_getter_callable(),
            ),
            (
                "fatal",
                Self::new_text_decoder_stream_fatal_getter_callable(),
            ),
            (
                "ignoreBOM",
                Self::new_text_decoder_stream_ignore_bom_getter_callable(),
            ),
            (
                "readable",
                Self::new_text_decoder_stream_readable_getter_callable(),
            ),
            (
                "writable",
                Self::new_text_decoder_stream_writable_getter_callable(),
            ),
        ] {
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                Self::object_getter_storage_key(property),
                getter,
            );
            Self::mark_property_non_enumerable(prototype, property);
        }
    }

    fn cached_text_codec_constructor_value(
        &mut self,
        name: &str,
        callable_kind: &str,
        tag: &str,
        installer: fn(&mut Self, &Rc<RefCell<ObjectValue>>),
    ) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get(name)
            .cloned()
        {
            if let Value::Object(entries) = &constructor {
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    installer(self, &prototype);
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
        let constructor = Self::new_object_backed_constructor_with_prototype(callable_kind, vec![]);
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
        installer(self, &prototype);
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String(tag.to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime
            .constructor_static_methods
            .insert(name.to_string(), constructor.clone());
        constructor
    }

    pub(crate) fn cached_text_encoder_constructor_value(&mut self) -> Value {
        self.cached_text_codec_constructor_value(
            "TextEncoder",
            "text_encoder_constructor",
            "TextEncoder",
            Self::install_text_encoder_prototype_surface,
        )
    }

    pub(crate) fn cached_text_decoder_constructor_value(&mut self) -> Value {
        self.cached_text_codec_constructor_value(
            "TextDecoder",
            "text_decoder_constructor",
            "TextDecoder",
            Self::install_text_decoder_prototype_surface,
        )
    }

    pub(crate) fn cached_text_encoder_stream_constructor_value(&mut self) -> Value {
        self.cached_text_codec_constructor_value(
            "TextEncoderStream",
            "text_encoder_stream_constructor",
            "TextEncoderStream",
            Self::install_text_encoder_stream_prototype_surface,
        )
    }

    pub(crate) fn cached_text_decoder_stream_constructor_value(&mut self) -> Value {
        self.cached_text_codec_constructor_value(
            "TextDecoderStream",
            "text_decoder_stream_constructor",
            "TextDecoderStream",
            Self::install_text_decoder_stream_prototype_surface,
        )
    }
}
