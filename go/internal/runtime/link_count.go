package runtime

import "browsertester/internal/dom"

func (s *Session) LinkCount() int {
	return s.linkCountForNodes(func(node *dom.Node) bool {
		if node == nil || node.Kind != dom.NodeKindElement {
			return false
		}
		switch node.TagName {
		case "a", "area":
			_, ok, err := s.domStore.GetAttribute(node.ID, "href")
			return err == nil && ok
		default:
			return false
		}
	})
}

func (s *Session) AnchorCount() int {
	return s.linkCountForNodes(func(node *dom.Node) bool {
		if node == nil || node.Kind != dom.NodeKindElement || node.TagName != "a" {
			return false
		}
		_, ok, err := s.domStore.GetAttribute(node.ID, "name")
		return err == nil && ok
	})
}

func (s *Session) linkCountForNodes(predicate func(*dom.Node) bool) int {
	if s == nil {
		return 0
	}
	store, err := s.ensureDOM()
	if err != nil || store == nil {
		return 0
	}
	total := 0
	for _, node := range store.Nodes() {
		if predicate != nil && predicate(node) {
			total++
		}
	}
	return total
}
