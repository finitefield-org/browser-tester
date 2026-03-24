package dom

import "fmt"

func (s *Store) SetUserValidity(nodeID NodeID, valid bool) error {
	if s == nil {
		return fmt.Errorf("dom store is nil")
	}
	node, err := s.elementNode(nodeID)
	if err != nil {
		return err
	}
	if !isUserValidityControl(node) {
		return fmt.Errorf("node %d does not support user validity", nodeID)
	}
	node.UserValidity = valid
	return nil
}

func isUserValidityControl(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "input", "select", "textarea":
		return true
	default:
		return false
	}
}
