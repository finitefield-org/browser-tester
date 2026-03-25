package dom

import (
	"fmt"
	"strings"
)

func (s *Store) SetFormControlValue(nodeID NodeID, value string) error {
	node := s.Node(nodeID)
	if node == nil || node.Kind != NodeKindElement {
		return fmt.Errorf("node %d is not a supported form control", nodeID)
	}

	switch node.TagName {
	case "textarea":
		return s.setTextContent(nodeID, value, false)
	case "input":
		if !isTextInputType(inputType(node)) {
			return fmt.Errorf(
				"set_value is only supported on text-like inputs and textareas, not <input type=%q>",
				inputType(node),
			)
		}
		node.Attrs = setAttribute(node.Attrs, "value", value, true)
		node.Attrs = removeAttribute(node.Attrs, "autofill")
		return nil
	default:
		return fmt.Errorf("node %d is not a supported form control", nodeID)
	}
}

func (s *Store) SetFormControlChecked(nodeID NodeID, checked bool) error {
	node := s.Node(nodeID)
	if node == nil || node.Kind != NodeKindElement {
		return fmt.Errorf("node %d is not a supported form control", nodeID)
	}

	if node.TagName != "input" || !isCheckableInputType(inputType(node)) {
		return fmt.Errorf(
			"set_checked is only supported on checkbox and radio inputs, not <input type=%q>",
			inputType(node),
		)
	}

	if inputType(node) == "radio" && checked {
		currentGroupForm := nearestAncestorForm(s, nodeID)
		currentName, _ := attributeValue(node.Attrs, "name")
		if currentName != "" {
			for _, other := range s.Nodes() {
				if other == nil || other.ID == nodeID || other.Kind != NodeKindElement || other.TagName != "input" {
					continue
				}
				if inputType(other) != "radio" {
					continue
				}
				otherName, ok := attributeValue(other.Attrs, "name")
				if !ok || otherName == "" || otherName != currentName {
					continue
				}
				if nearestAncestorForm(s, other.ID) != currentGroupForm {
					continue
				}
				other.Attrs = removeAttribute(other.Attrs, "checked")
			}
		}
	}

	if checked {
		node.Attrs = setAttribute(node.Attrs, "checked", "", false)
	} else {
		node.Attrs = removeAttribute(node.Attrs, "checked")
	}
	return nil
}

func (s *Store) SetSelectValue(nodeID NodeID, value string) error {
	node := s.Node(nodeID)
	if node == nil || node.Kind != NodeKindElement {
		return fmt.Errorf("node %d is not a supported select control", nodeID)
	}
	if node.TagName != "select" {
		return fmt.Errorf("node %d is not a select control", nodeID)
	}

	options := make([]NodeID, 0, 4)
	s.walkElementPreOrder(nodeID, func(current *Node) {
		if current == nil || current.TagName != "option" {
			return
		}
		options = append(options, current.ID)
	})

	if len(options) == 0 {
		return fmt.Errorf("select node %d does not contain any options", nodeID)
	}

	matched := false
	for _, optionID := range options {
		current := s.Node(optionID)
		if current == nil {
			continue
		}
		current.Attrs = removeAttribute(current.Attrs, "selected")
		if !matched && optionValueForNode(s, current.ID) == value {
			current.Attrs = setAttribute(current.Attrs, "selected", "", false)
			matched = true
		}
	}
	return nil
}

func (s *Store) ResetFormControls(nodeID NodeID) error {
	node := s.Node(nodeID)
	if node == nil || node.Kind != NodeKindElement {
		return fmt.Errorf("node %d is not a form control container", nodeID)
	}
	if node.TagName != "form" {
		return fmt.Errorf("node %d is not a form element", nodeID)
	}

	s.walkElementPreOrder(nodeID, func(current *Node) {
		if current == nil {
			return
		}

		switch current.TagName {
		case "input":
			resetInputControl(current)
		case "select":
			current.UserValidity = false
		case "textarea":
			current.UserValidity = false
			_ = s.setTextContent(current.ID, current.DefaultText, false)
		case "option":
			if defaultHasAttribute(current, "selected") {
				current.Attrs = setAttribute(current.Attrs, "selected", "", false)
			} else {
				current.Attrs = removeAttribute(current.Attrs, "selected")
			}
		}
	})
	return nil
}

func (s *Store) syncTextareaDefaultsForSubtree(nodeID NodeID) {
	if s == nil || nodeID == 0 {
		return
	}

	current := nodeID
	for current != 0 {
		node := s.Node(current)
		if node == nil {
			return
		}
		if node.Kind == NodeKindElement && node.TagName == "textarea" {
			node.DefaultText = s.TextContentForNode(node.ID)
		}
		current = node.Parent
	}
}

func (s *Store) SetTextContent(nodeID NodeID, text string) error {
	return s.setTextContent(nodeID, text, true)
}

func (s *Store) setTextContent(nodeID NodeID, text string, updateTextareaDefault bool) error {
	node := s.Node(nodeID)
	if node == nil {
		return fmt.Errorf("invalid node id: %d", nodeID)
	}

	switch node.Kind {
	case NodeKindText:
		node.Text = text
		if updateTextareaDefault {
			s.syncTextareaDefaultsForSubtree(nodeID)
		}
		return nil
	case NodeKindElement:
		s.clearFocusedNodeIfSubtreeContains(nodeID, false)
		s.clearTargetNodeIfSubtreeContains(nodeID, false)
		oldChildren := append([]NodeID(nil), node.Children...)
		node.Children = node.Children[:0]
		for _, childID := range oldChildren {
			s.deleteSubtree(childID)
		}
		if text == "" {
			if updateTextareaDefault && node.TagName == "textarea" {
				node.DefaultText = text
			}
			return nil
		}
		textID := s.newNode(Node{
			Kind: NodeKindText,
			Text: text,
		})
		s.appendChild(nodeID, textID)
		if updateTextareaDefault && node.TagName == "textarea" {
			node.DefaultText = text
		}
		if updateTextareaDefault {
			s.syncTextareaDefaultsForSubtree(nodeID)
		}
		return nil
	default:
		return fmt.Errorf("node %d does not support text content", nodeID)
	}
}

func inputType(node *Node) string {
	if node == nil {
		return ""
	}
	value, _ := attributeValue(node.Attrs, "type")
	return strings.ToLower(strings.TrimSpace(value))
}

func isFormListedElement(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}

	switch node.TagName {
	case "button", "fieldset", "input", "select", "textarea", "output":
		return true
	default:
		return false
	}
}

func isTextInputType(typeName string) bool {
	switch strings.ToLower(strings.TrimSpace(typeName)) {
	case "", "text", "search", "url", "tel", "email", "password", "number", "date", "datetime-local", "time", "month", "week", "color":
		return true
	default:
		return false
	}
}

func isCheckableInputType(typeName string) bool {
	switch strings.ToLower(strings.TrimSpace(typeName)) {
	case "checkbox", "radio":
		return true
	default:
		return false
	}
}

func optionValueForNode(s *Store, nodeID NodeID) string {
	node := s.Node(nodeID)
	if node == nil || node.Kind != NodeKindElement || node.TagName != "option" {
		return ""
	}
	if value, ok := attributeValue(node.Attrs, "value"); ok {
		return value
	}
	return s.TextContentForNode(nodeID)
}

func resetInputControl(node *Node) {
	if node == nil || node.Kind != NodeKindElement || node.TagName != "input" {
		return
	}
	node.UserValidity = false

	switch inputType(node) {
	case "checkbox", "radio":
		if defaultHasAttribute(node, "checked") {
			node.Attrs = setAttribute(node.Attrs, "checked", "", false)
		} else {
			node.Attrs = removeAttribute(node.Attrs, "checked")
		}
	case "", "text", "search", "url", "tel", "email", "password", "number", "date", "datetime-local", "time", "month", "week", "color":
		if value, ok := defaultAttributeValue(node, "value"); ok {
			node.Attrs = setAttribute(node.Attrs, "value", value, true)
		} else {
			node.Attrs = removeAttribute(node.Attrs, "value")
		}
	}
}

func defaultAttributeValue(node *Node, name string) (string, bool) {
	if node == nil {
		return "", false
	}
	return attributeValue(node.DefaultAttrs, name)
}

func defaultHasAttribute(node *Node, name string) bool {
	if node == nil {
		return false
	}
	_, ok := attributeValue(node.DefaultAttrs, name)
	return ok
}

func nearestAncestorForm(s *Store, nodeID NodeID) NodeID {
	current := nodeID
	for current != 0 {
		node := s.Node(current)
		if node == nil {
			return 0
		}
		if node.Kind == NodeKindElement && node.TagName == "form" {
			return node.ID
		}
		current = node.Parent
	}
	return 0
}

func setAttribute(attrs []Attribute, name, value string, hasValue bool) []Attribute {
	for i := range attrs {
		if attrs[i].Name == name {
			attrs[i].Value = value
			attrs[i].HasValue = hasValue
			return attrs
		}
	}
	return append(attrs, Attribute{Name: name, Value: value, HasValue: hasValue})
}

func removeAttribute(attrs []Attribute, name string) []Attribute {
	if len(attrs) == 0 {
		return attrs
	}
	out := attrs[:0]
	for _, attr := range attrs {
		if attr.Name == name {
			continue
		}
		out = append(out, attr)
	}
	return out
}
