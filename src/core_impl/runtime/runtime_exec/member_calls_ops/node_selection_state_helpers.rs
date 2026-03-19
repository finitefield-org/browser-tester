use super::*;

impl Harness {
    pub(crate) fn ensure_document_selection_object(&mut self) -> Value {
        let is_selection_object = {
            let entries = self.dom_runtime.selection_object.borrow();
            Self::is_selection_object(&entries)
        };
        if !is_selection_object {
            self.dom_runtime.selection_object =
                match Self::new_selection_object_value(self.dom.root) {
                    Value::Object(selection) => selection,
                    _ => Rc::new(RefCell::new(ObjectValue::default())),
                };
        }
        Value::Object(self.dom_runtime.selection_object.clone())
    }

    pub(crate) fn selection_internal_range_object(
        &mut self,
        selection: &Rc<RefCell<ObjectValue>>,
    ) -> Rc<RefCell<ObjectValue>> {
        let existing = {
            let entries = selection.borrow();
            match Self::object_get_entry(&entries, INTERNAL_SELECTION_RANGE_KEY) {
                Some(Value::Object(range)) => Some(range),
                _ => None,
            }
        };
        if let Some(range) = existing {
            return range;
        }

        let range = match Self::new_range_object_value(self.dom.root) {
            Value::Object(range) => range,
            _ => Rc::new(RefCell::new(ObjectValue::default())),
        };
        Self::object_set_entry(
            &mut selection.borrow_mut(),
            INTERNAL_SELECTION_RANGE_KEY.to_string(),
            Value::Object(range.clone()),
        );
        range
    }

    pub(crate) fn selection_set_state(
        &mut self,
        anchor_node: Option<NodeId>,
        anchor_offset: i64,
        focus_node: Option<NodeId>,
        focus_offset: i64,
    ) -> bool {
        let selection = match self.ensure_document_selection_object() {
            Value::Object(selection) => selection,
            _ => return false,
        };

        let before = {
            let entries = selection.borrow();
            (
                Self::object_get_entry(&entries, "anchorNode"),
                Self::object_get_entry(&entries, "anchorOffset"),
                Self::object_get_entry(&entries, "focusNode"),
                Self::object_get_entry(&entries, "focusOffset"),
                Self::object_get_entry(&entries, "rangeCount"),
                Self::object_get_entry(&entries, "direction"),
            )
        };

        if let (Some(anchor_node), Some(focus_node)) = (anchor_node, focus_node) {
            let anchor_offset = self.selection_clamped_offset(anchor_node, anchor_offset);
            let focus_offset = self.selection_clamped_offset(focus_node, focus_offset);
            let ordering =
                self.selection_compare_points(anchor_node, anchor_offset, focus_node, focus_offset);
            let (start_container, start_offset, end_container, end_offset, direction) =
                match ordering {
                    std::cmp::Ordering::Greater => (
                        focus_node,
                        focus_offset,
                        anchor_node,
                        anchor_offset,
                        "backward",
                    ),
                    std::cmp::Ordering::Equal => {
                        (anchor_node, anchor_offset, focus_node, focus_offset, "none")
                    }
                    std::cmp::Ordering::Less => (
                        anchor_node,
                        anchor_offset,
                        focus_node,
                        focus_offset,
                        "forward",
                    ),
                };
            let range_object = self.selection_internal_range_object(&selection);
            {
                let mut range_entries = range_object.borrow_mut();
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_CONTAINER_KEY.to_string(),
                    Value::Node(start_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_OFFSET_KEY.to_string(),
                    Value::Number(start_offset),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_CONTAINER_KEY.to_string(),
                    Value::Node(end_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_OFFSET_KEY.to_string(),
                    Value::Number(end_offset),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startContainer".to_string(),
                    Value::Node(start_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startOffset".to_string(),
                    Value::Number(start_offset),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endContainer".to_string(),
                    Value::Node(end_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endOffset".to_string(),
                    Value::Number(end_offset),
                );
            }
            {
                let mut entries = selection.borrow_mut();
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_SELECTION_RANGE_KEY.to_string(),
                    Value::Object(range_object),
                );
                Self::object_set_entry(
                    &mut entries,
                    "anchorNode".to_string(),
                    Value::Node(anchor_node),
                );
                Self::object_set_entry(
                    &mut entries,
                    "anchorOffset".to_string(),
                    Value::Number(anchor_offset),
                );
                Self::object_set_entry(
                    &mut entries,
                    "focusNode".to_string(),
                    Value::Node(focus_node),
                );
                Self::object_set_entry(
                    &mut entries,
                    "focusOffset".to_string(),
                    Value::Number(focus_offset),
                );
                Self::object_set_entry(
                    &mut entries,
                    "isCollapsed".to_string(),
                    Value::Bool(ordering == std::cmp::Ordering::Equal),
                );
                Self::object_set_entry(&mut entries, "rangeCount".to_string(), Value::Number(1));
                Self::object_set_entry(
                    &mut entries,
                    "type".to_string(),
                    Value::String(if ordering == std::cmp::Ordering::Equal {
                        "Caret".to_string()
                    } else {
                        "Range".to_string()
                    }),
                );
                Self::object_set_entry(
                    &mut entries,
                    "direction".to_string(),
                    Value::String(direction.to_string()),
                );
            }
        } else {
            let range_object = self.selection_internal_range_object(&selection);
            {
                let mut range_entries = range_object.borrow_mut();
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_CONTAINER_KEY.to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_OFFSET_KEY.to_string(),
                    Value::Number(0),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_CONTAINER_KEY.to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_OFFSET_KEY.to_string(),
                    Value::Number(0),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startContainer".to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startOffset".to_string(),
                    Value::Number(0),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endContainer".to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endOffset".to_string(),
                    Value::Number(0),
                );
            }
            {
                let mut entries = selection.borrow_mut();
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_SELECTION_RANGE_KEY.to_string(),
                    Value::Object(range_object),
                );
                Self::object_set_entry(&mut entries, "anchorNode".to_string(), Value::Null);
                Self::object_set_entry(&mut entries, "anchorOffset".to_string(), Value::Number(0));
                Self::object_set_entry(&mut entries, "focusNode".to_string(), Value::Null);
                Self::object_set_entry(&mut entries, "focusOffset".to_string(), Value::Number(0));
                Self::object_set_entry(&mut entries, "isCollapsed".to_string(), Value::Bool(true));
                Self::object_set_entry(&mut entries, "rangeCount".to_string(), Value::Number(0));
                Self::object_set_entry(
                    &mut entries,
                    "type".to_string(),
                    Value::String("None".to_string()),
                );
                Self::object_set_entry(
                    &mut entries,
                    "direction".to_string(),
                    Value::String("none".to_string()),
                );
            }
        }

        let after = {
            let entries = selection.borrow();
            (
                Self::object_get_entry(&entries, "anchorNode"),
                Self::object_get_entry(&entries, "anchorOffset"),
                Self::object_get_entry(&entries, "focusNode"),
                Self::object_get_entry(&entries, "focusOffset"),
                Self::object_get_entry(&entries, "rangeCount"),
                Self::object_get_entry(&entries, "direction"),
            )
        };
        before != after
    }
}
