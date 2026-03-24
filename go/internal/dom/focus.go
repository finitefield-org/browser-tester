package dom

import (
	"fmt"
	"strings"
)

func (s *Store) FocusedNodeID() NodeID {
	if s == nil {
		return 0
	}
	return s.focusedNodeID
}

func (s *Store) SetFocusedNode(nodeID NodeID) error {
	if s == nil {
		return fmt.Errorf("dom store is nil")
	}
	if nodeID == 0 {
		s.focusedNodeID = 0
		return nil
	}
	node, err := s.elementNode(nodeID)
	if err != nil {
		return err
	}
	s.focusedNodeID = node.ID
	return nil
}

func (s *Store) ClearFocusedNode() {
	if s == nil {
		return
	}
	s.focusedNodeID = 0
}

func (s *Store) clearFocusedNodeIfSubtreeContains(nodeID NodeID, includeSelf bool) {
	if s == nil || s.focusedNodeID == 0 || nodeID == 0 {
		return
	}
	if includeSelf {
		if subtreeContainsNode(s, nodeID, s.focusedNodeID) {
			s.focusedNodeID = 0
		}
		return
	}
	if subtreeContainsDescendant(s, nodeID, s.focusedNodeID) {
		s.focusedNodeID = 0
	}
}

func subtreeContainsNode(s *Store, ancestorID, nodeID NodeID) bool {
	if s == nil || ancestorID == 0 || nodeID == 0 {
		return false
	}
	if ancestorID == nodeID {
		return true
	}
	ancestor := s.Node(ancestorID)
	if ancestor == nil {
		return false
	}
	for _, childID := range ancestor.Children {
		if childID == nodeID || subtreeContainsNode(s, childID, nodeID) {
			return true
		}
	}
	return false
}

func subtreeContainsDescendant(s *Store, ancestorID, nodeID NodeID) bool {
	return nodeID != 0 && ancestorID != nodeID && subtreeContainsNode(s, ancestorID, nodeID)
}

func subtreeContainsAttribute(s *Store, ancestorID NodeID, attrName string) bool {
	if s == nil || ancestorID == 0 || strings.TrimSpace(attrName) == "" {
		return false
	}
	found := false
	s.walkElementPreOrder(ancestorID, func(node *Node) {
		if found || node == nil || node.Kind != NodeKindElement {
			return
		}
		if _, ok := attributeValue(node.Attrs, attrName); ok {
			found = true
		}
	})
	return found
}
