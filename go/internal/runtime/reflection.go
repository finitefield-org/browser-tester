package runtime

import (
	"fmt"

	"browsertester/internal/dom"
)

func (s *Session) ClassList(selector string) (dom.ClassList, error) {
	if s == nil {
		return dom.ClassList{}, fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return dom.ClassList{}, err
	}
	return store.ClassList(nodeID)
}

func (s *Session) Dataset(selector string) (dom.Dataset, error) {
	if s == nil {
		return dom.Dataset{}, fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return dom.Dataset{}, err
	}
	return store.Dataset(nodeID)
}
