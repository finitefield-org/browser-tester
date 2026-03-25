package dom

import "fmt"

type ChildNodeList struct {
	store    *Store
	parentID NodeID
}

func newChildNodeList(store *Store, parentID NodeID) ChildNodeList {
	return ChildNodeList{
		store:    store,
		parentID: parentID,
	}
}

func (l ChildNodeList) Length() int {
	return len(l.nodeIDs())
}

func (l ChildNodeList) Item(index int) (NodeID, bool) {
	ids := l.nodeIDs()
	if index < 0 || index >= len(ids) {
		return 0, false
	}
	return ids[index], true
}

func (l ChildNodeList) IDs() []NodeID {
	ids := l.nodeIDs()
	if len(ids) == 0 {
		return []NodeID{}
	}
	out := make([]NodeID, len(ids))
	copy(out, ids)
	return out
}

func (l ChildNodeList) nodeIDs() []NodeID {
	if l.store == nil {
		return []NodeID{}
	}
	parent := l.store.Node(l.parentID)
	if parent == nil {
		return []NodeID{}
	}
	if parent.Kind != NodeKindDocument && parent.Kind != NodeKindElement {
		return []NodeID{}
	}
	out := make([]NodeID, len(parent.Children))
	copy(out, parent.Children)
	return out
}

func (s *Store) ChildNodes(nodeID NodeID) (ChildNodeList, error) {
	if s == nil {
		return ChildNodeList{}, fmt.Errorf("dom store is nil")
	}
	node := s.Node(nodeID)
	if node == nil {
		return ChildNodeList{}, fmt.Errorf("invalid node id: %d", nodeID)
	}
	switch node.Kind {
	case NodeKindDocument, NodeKindElement:
		return newChildNodeList(s, nodeID), nil
	default:
		return ChildNodeList{}, fmt.Errorf("node %d does not support childNodes", nodeID)
	}
}

func (s *Store) TemplateContentChildNodes(nodeID NodeID) (ChildNodeList, error) {
	if s == nil {
		return ChildNodeList{}, fmt.Errorf("dom store is nil")
	}
	node := s.Node(nodeID)
	if node == nil {
		return ChildNodeList{}, fmt.Errorf("invalid node id: %d", nodeID)
	}
	if node.Kind != NodeKindElement || node.TagName != "template" {
		return ChildNodeList{}, fmt.Errorf("node %d does not support template content childNodes", nodeID)
	}
	return newChildNodeList(s, nodeID), nil
}
