package runtime

import (
	"fmt"
	"strings"

	"browsertester/internal/dom"
)

func (s *Session) enqueueMicrotask(source string) {
	if s == nil {
		return
	}
	normalized := strings.TrimSpace(source)
	if normalized == "" {
		return
	}
	s.microtasks = append(s.microtasks, normalized)
}

func (s *Session) discardMicrotasks() {
	if s == nil {
		return
	}
	s.microtasks = nil
}

func (s *Session) PendingMicrotasks() []string {
	if s == nil || len(s.microtasks) == 0 {
		return nil
	}
	out := make([]string, len(s.microtasks))
	copy(out, s.microtasks)
	return out
}

func (s *Session) drainMicrotasks(store *dom.Store) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if store == nil {
		store = s.domStore
	}
	if store == nil {
		return fmt.Errorf("dom store is unavailable")
	}

	for len(s.microtasks) > 0 {
		if s.domStore != nil && s.domStore != store {
			store = s.domStore
		}
		batch := append([]string(nil), s.microtasks...)
		s.microtasks = nil
		for _, source := range batch {
			if s.domStore != nil && s.domStore != store {
				store = s.domStore
			}
			if _, err := s.runScriptOnStore(store, source); err != nil {
				s.microtasks = nil
				return err
			}
		}
	}
	return nil
}
