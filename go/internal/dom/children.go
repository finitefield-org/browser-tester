package dom

import "fmt"

func (s *Store) Children(nodeID NodeID) (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	node := s.Node(nodeID)
	if node == nil {
		return HTMLCollection{}, fmt.Errorf("invalid node id: %d", nodeID)
	}
	switch node.Kind {
	case NodeKindDocument, NodeKindElement:
		return newHTMLCollection(s, nodeID), nil
	default:
		return HTMLCollection{}, fmt.Errorf("node %d does not support children", nodeID)
	}
}
