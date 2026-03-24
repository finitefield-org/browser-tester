package dom

import (
	"fmt"
	"strings"
)

func (s *Store) InnerHTMLForNode(nodeID NodeID) (string, error) {
	if s == nil {
		return "", fmt.Errorf("dom store is nil")
	}
	node, err := s.elementNode(nodeID)
	if err != nil {
		return "", err
	}

	var b strings.Builder
	for _, childID := range node.Children {
		s.serializeNode(&b, childID)
	}
	return b.String(), nil
}

func (s *Store) SetInnerHTML(nodeID NodeID, markup string) error {
	if s == nil {
		return fmt.Errorf("dom store is nil")
	}
	node, err := s.elementNode(nodeID)
	if err != nil {
		return err
	}

	fragmentIDs, err := s.parseFragmentNodes(markup)
	if err != nil {
		return err
	}

	oldChildren := append([]NodeID(nil), node.Children...)
	node.Children = node.Children[:0]
	for _, childID := range oldChildren {
		s.deleteSubtree(childID)
	}
	for _, childID := range fragmentIDs {
		s.appendChild(nodeID, childID)
	}
	return nil
}

func (s *Store) SetOuterHTML(nodeID NodeID, markup string) error {
	if s == nil {
		return fmt.Errorf("dom store is nil")
	}
	node, err := s.elementNode(nodeID)
	if err != nil {
		return err
	}

	parentID := node.Parent
	if parentID == 0 {
		return nil
	}
	parent := s.Node(parentID)
	if parent == nil {
		return fmt.Errorf("invalid parent node id: %d", parentID)
	}
	if parent.Kind == NodeKindDocument {
		return fmt.Errorf("node %d cannot be replaced within a document", nodeID)
	}

	fragmentIDs, err := s.parseFragmentNodes(markup)
	if err != nil {
		return err
	}

	index := indexOfNodeID(parent.Children, nodeID)
	if index < 0 {
		return fmt.Errorf("node %d is not attached to its parent", nodeID)
	}

	parent.Children = spliceNodeIDs(parent.Children, index, 1, fragmentIDs)
	for _, childID := range fragmentIDs {
		child := s.Node(childID)
		if child != nil {
			child.Parent = parentID
		}
	}

	s.deleteSubtree(nodeID)
	return nil
}

func (s *Store) InsertAdjacentHTML(nodeID NodeID, position, markup string) error {
	if s == nil {
		return fmt.Errorf("dom store is nil")
	}
	node, err := s.elementNode(nodeID)
	if err != nil {
		return err
	}

	normalized := strings.ToLower(strings.TrimSpace(position))
	switch normalized {
	case "beforebegin":
		parentID := node.Parent
		if parentID == 0 {
			return fmt.Errorf("node %d has no parent for beforebegin", nodeID)
		}
		parent := s.Node(parentID)
		if parent == nil {
			return fmt.Errorf("invalid parent node id: %d", parentID)
		}
		if parent.Kind == NodeKindDocument {
			return fmt.Errorf("node %d cannot insert beforebegin within a document", nodeID)
		}
		index := indexOfNodeID(parent.Children, nodeID)
		if index < 0 {
			return fmt.Errorf("node %d is not attached to its parent", nodeID)
		}
		fragmentIDs, err := s.parseFragmentNodes(markup)
		if err != nil {
			return err
		}
		parent.Children = spliceNodeIDs(parent.Children, index, 0, fragmentIDs)
		for _, childID := range fragmentIDs {
			if child := s.Node(childID); child != nil {
				child.Parent = parentID
			}
		}
	case "afterbegin":
		fragmentIDs, err := s.parseFragmentNodes(markup)
		if err != nil {
			return err
		}
		node.Children = spliceNodeIDs(node.Children, 0, 0, fragmentIDs)
		for _, childID := range fragmentIDs {
			if child := s.Node(childID); child != nil {
				child.Parent = nodeID
			}
		}
	case "beforeend":
		fragmentIDs, err := s.parseFragmentNodes(markup)
		if err != nil {
			return err
		}
		node.Children = spliceNodeIDs(node.Children, len(node.Children), 0, fragmentIDs)
		for _, childID := range fragmentIDs {
			if child := s.Node(childID); child != nil {
				child.Parent = nodeID
			}
		}
	case "afterend":
		parentID := node.Parent
		if parentID == 0 {
			return fmt.Errorf("node %d has no parent for afterend", nodeID)
		}
		parent := s.Node(parentID)
		if parent == nil {
			return fmt.Errorf("invalid parent node id: %d", parentID)
		}
		if parent.Kind == NodeKindDocument {
			return fmt.Errorf("node %d cannot insert afterend within a document", nodeID)
		}
		index := indexOfNodeID(parent.Children, nodeID)
		if index < 0 {
			return fmt.Errorf("node %d is not attached to its parent", nodeID)
		}
		fragmentIDs, err := s.parseFragmentNodes(markup)
		if err != nil {
			return err
		}
		parent.Children = spliceNodeIDs(parent.Children, index+1, 0, fragmentIDs)
		for _, childID := range fragmentIDs {
			if child := s.Node(childID); child != nil {
				child.Parent = parentID
			}
		}
	default:
		return fmt.Errorf("invalid insertAdjacentHTML position %q", position)
	}

	return nil
}

func (s *Store) RemoveNode(nodeID NodeID) error {
	if s == nil {
		return fmt.Errorf("dom store is nil")
	}
	node := s.Node(nodeID)
	if node == nil {
		return fmt.Errorf("invalid node id: %d", nodeID)
	}
	if node.Kind == NodeKindDocument {
		return fmt.Errorf("document node cannot be removed")
	}
	if node.Parent == 0 {
		return nil
	}

	parent := s.Node(node.Parent)
	if parent != nil {
		parent.Children = removeNodeID(parent.Children, nodeID)
	}
	s.deleteSubtree(nodeID)
	return nil
}

func (s *Store) CloneNode(nodeID NodeID, deep bool) (NodeID, error) {
	if s == nil {
		return 0, fmt.Errorf("dom store is nil")
	}
	if s.Node(nodeID) == nil {
		return 0, fmt.Errorf("invalid node id: %d", nodeID)
	}
	return s.cloneNodeRecursive(nodeID, deep), nil
}

func (s *Store) parseFragmentNodes(markup string) ([]NodeID, error) {
	temp := NewStore()
	if err := temp.BootstrapHTML(markup); err != nil {
		return nil, err
	}

	rootChildren := temp.documentChildren()
	if len(rootChildren) == 0 {
		return []NodeID{}, nil
	}

	out := make([]NodeID, 0, len(rootChildren))
	for _, childID := range rootChildren {
		cloned := s.cloneNodeFrom(temp, childID, true)
		if cloned != 0 {
			out = append(out, cloned)
		}
	}
	return out, nil
}

func (s *Store) cloneNodeRecursive(nodeID NodeID, deep bool) NodeID {
	node := s.Node(nodeID)
	if node == nil {
		return 0
	}

	clonedID := s.newNode(Node{
		Kind:         node.Kind,
		TagName:      node.TagName,
		Attrs:        cloneAttributes(node.Attrs),
		Text:         node.Text,
		DefaultAttrs: cloneAttributes(node.DefaultAttrs),
		DefaultText:  node.DefaultText,
	})
	if !deep {
		return clonedID
	}

	for _, childID := range node.Children {
		clonedChildID := s.cloneNodeRecursive(childID, true)
		if clonedChildID != 0 {
			s.appendChild(clonedID, clonedChildID)
		}
	}
	return clonedID
}

func (s *Store) cloneNodeFrom(src *Store, nodeID NodeID, deep bool) NodeID {
	if s == nil || src == nil {
		return 0
	}
	node := src.Node(nodeID)
	if node == nil {
		return 0
	}

	clonedID := s.newNode(Node{
		Kind:         node.Kind,
		TagName:      node.TagName,
		Attrs:        cloneAttributes(node.Attrs),
		Text:         node.Text,
		DefaultAttrs: cloneAttributes(node.DefaultAttrs),
		DefaultText:  node.DefaultText,
	})
	if !deep {
		return clonedID
	}

	for _, childID := range node.Children {
		clonedChildID := s.cloneNodeFrom(src, childID, true)
		if clonedChildID != 0 {
			s.appendChild(clonedID, clonedChildID)
		}
	}
	return clonedID
}

func cloneAttributes(attrs []Attribute) []Attribute {
	if len(attrs) == 0 {
		return []Attribute{}
	}
	out := make([]Attribute, len(attrs))
	copy(out, attrs)
	return out
}

func deleteSubtree(s *Store, nodeID NodeID) {
	if s == nil {
		return
	}
	node := s.nodes[nodeID]
	if node == nil {
		return
	}

	children := append([]NodeID(nil), node.Children...)
	for _, childID := range children {
		deleteSubtree(s, childID)
	}

	if parent := s.nodes[node.Parent]; parent != nil {
		parent.Children = removeNodeID(parent.Children, nodeID)
	}
	delete(s.nodes, nodeID)
}

func (s *Store) deleteSubtree(nodeID NodeID) {
	deleteSubtree(s, nodeID)
}

func indexOfNodeID(ids []NodeID, target NodeID) int {
	for i, id := range ids {
		if id == target {
			return i
		}
	}
	return -1
}

func spliceNodeIDs(ids []NodeID, index, deleteCount int, insert []NodeID) []NodeID {
	if index < 0 {
		index = 0
	}
	if index > len(ids) {
		index = len(ids)
	}
	if deleteCount < 0 {
		deleteCount = 0
	}
	end := index + deleteCount
	if end > len(ids) {
		end = len(ids)
	}

	out := make([]NodeID, 0, len(ids)-(end-index)+len(insert))
	out = append(out, ids[:index]...)
	out = append(out, insert...)
	out = append(out, ids[end:]...)
	return out
}

func removeNodeID(ids []NodeID, target NodeID) []NodeID {
	index := indexOfNodeID(ids, target)
	if index < 0 {
		return ids
	}
	return append(ids[:index], ids[index+1:]...)
}
