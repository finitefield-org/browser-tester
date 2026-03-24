package dom

import (
	"fmt"
	"net/url"
	"strings"
)

func (s *Store) TargetNodeID() NodeID {
	if s == nil {
		return 0
	}
	return s.targetNodeID
}

func (s *Store) SetTargetNode(nodeID NodeID) error {
	if s == nil {
		return fmt.Errorf("dom store is nil")
	}
	if nodeID == 0 {
		s.targetNodeID = 0
		return nil
	}
	node, err := s.elementNode(nodeID)
	if err != nil {
		return err
	}
	s.targetNodeID = node.ID
	return nil
}

func (s *Store) ClearTargetNode() {
	if s == nil {
		return
	}
	s.targetNodeID = 0
}

func (s *Store) SyncTargetFromURL(currentURL string) {
	if s == nil {
		return
	}
	s.targetNodeID = s.resolveTargetNodeFromURL(currentURL)
}

func (s *Store) clearTargetNodeIfSubtreeContains(nodeID NodeID, includeSelf bool) {
	if s == nil || s.targetNodeID == 0 || nodeID == 0 {
		return
	}
	if includeSelf {
		if subtreeContainsNode(s, nodeID, s.targetNodeID) {
			s.targetNodeID = 0
		}
		return
	}
	if subtreeContainsDescendant(s, nodeID, s.targetNodeID) {
		s.targetNodeID = 0
	}
}

func (s *Store) resolveTargetNodeFromURL(currentURL string) NodeID {
	if s == nil {
		return 0
	}

	trimmed := strings.TrimSpace(currentURL)
	if trimmed == "" {
		return 0
	}

	parsed, err := url.Parse(trimmed)
	if err != nil {
		return 0
	}

	fragment := parsed.Fragment
	if fragment == "" {
		return 0
	}

	if nodeID := s.findPotentialTargetNode(fragment); nodeID != 0 {
		return nodeID
	}

	decodedFragment, err := url.PathUnescape(fragment)
	if err != nil {
		return 0
	}
	if nodeID := s.findPotentialTargetNode(decodedFragment); nodeID != 0 {
		return nodeID
	}
	if strings.EqualFold(decodedFragment, "top") {
		return 0
	}
	return 0
}

func (s *Store) findPotentialTargetNode(fragment string) NodeID {
	if s == nil {
		return 0
	}
	if fragment == "" {
		return 0
	}

	for _, rootID := range s.documentChildren() {
		var found NodeID
		s.walkElementPreOrder(rootID, func(node *Node) {
			if found != 0 || node == nil {
				return
			}
			if idValue, ok := attributeValue(node.Attrs, "id"); ok && idValue == fragment {
				found = node.ID
			}
		})
		if found != 0 {
			return found
		}
	}

	for _, rootID := range s.documentChildren() {
		var found NodeID
		s.walkElementPreOrder(rootID, func(node *Node) {
			if found != 0 || node == nil || node.TagName != "a" {
				return
			}
			if nameValue, ok := attributeValue(node.Attrs, "name"); ok && nameValue == fragment {
				found = node.ID
			}
		})
		if found != 0 {
			return found
		}
	}

	return 0
}
