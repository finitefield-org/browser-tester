package runtime

import "testing"

func TestSessionFocusedNodeIDInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/app",
		HTML: `<main><input id="field" type="text" value="hello"><p id="other">other</p></main>`,
	})

	store, err := s.ensureDOM()
	if err != nil {
		t.Fatalf("ensureDOM() error = %v", err)
	}

	nodes, err := store.Select("#field")
	if err != nil {
		t.Fatalf("Select(#field) error = %v", err)
	}
	if len(nodes) != 1 {
		t.Fatalf("Select(#field) len = %d, want 1", len(nodes))
	}
	if err := s.Focus("#field"); err != nil {
		t.Fatalf("Focus(#field) error = %v", err)
	}
	if got := s.FocusedNodeID(); got != int64(nodes[0]) {
		t.Fatalf("FocusedNodeID() = %d, want %d", got, nodes[0])
	}

	if err := s.Blur(); err != nil {
		t.Fatalf("Blur() error = %v", err)
	}
	if got := s.FocusedNodeID(); got != 0 {
		t.Fatalf("FocusedNodeID() after blur = %d, want 0", got)
	}

	var nilSession *Session
	if got := nilSession.FocusedNodeID(); got != 0 {
		t.Fatalf("nil FocusedNodeID() = %d, want 0", got)
	}
}
