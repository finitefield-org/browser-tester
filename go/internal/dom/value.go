package dom

// ValueForNode returns a bounded "value" view similar to HTML form-control value reflection.
// It is intentionally small and only covers the currently implemented DOM slices.
func (s *Store) ValueForNode(nodeID NodeID) string {
	if s == nil {
		return ""
	}
	node := s.Node(nodeID)
	if node == nil {
		return ""
	}

	switch node.Kind {
	case NodeKindDocument:
		return s.TextContentForNode(nodeID)
	case NodeKindText:
		return node.Text
	case NodeKindElement:
		// Continue below.
	default:
		return ""
	}

	switch node.TagName {
	case "select":
		return s.selectValueForNode(nodeID)
	case "option":
		return optionValueForNode(s, nodeID)
	case "textarea":
		// The DOM store models textarea value as its text content.
		return s.TextContentForNode(nodeID)
	case "input":
		// File inputs are not wired into the DOM store yet (they are captured via mocks).
		if inputType(node) == "file" {
			return ""
		}
		if value, ok := attributeValue(node.Attrs, "value"); ok {
			return value
		}
		return s.TextContentForNode(nodeID)
	default:
		if value, ok := attributeValue(node.Attrs, "value"); ok {
			return value
		}
		return s.TextContentForNode(nodeID)
	}
}

// CheckedForNode reports the bounded checkedness of a node.
// The second return value is false when the node is not checkable.
func (s *Store) CheckedForNode(nodeID NodeID) (bool, bool) {
	if s == nil {
		return false, false
	}
	node := s.Node(nodeID)
	if node == nil || node.Kind != NodeKindElement || node.TagName != "input" {
		return false, false
	}
	if !isCheckableInputType(inputType(node)) {
		return false, false
	}
	_, ok := attributeValue(node.Attrs, "checked")
	return ok, true
}

func (s *Store) selectValueForNode(nodeID NodeID) string {
	node := s.Node(nodeID)
	if node == nil || node.Kind != NodeKindElement || node.TagName != "select" {
		return ""
	}

	var selectedOption NodeID
	s.walkElementPreOrder(nodeID, func(current *Node) {
		if selectedOption != 0 || current == nil || current.Kind != NodeKindElement || current.TagName != "option" {
			return
		}
		if _, ok := attributeValue(current.Attrs, "selected"); ok {
			selectedOption = current.ID
		}
	})
	if selectedOption == 0 {
		return ""
	}
	return optionValueForNode(s, selectedOption)
}
