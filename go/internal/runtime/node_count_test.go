package runtime

import "testing"

func TestSessionNodeCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/app",
		HTML: `<main><section><p id="one">one</p><p id="two">two</p></section></main>`,
	})

	if got := s.NodeCount(); got == 0 {
		t.Fatalf("NodeCount() = %d, want non-zero for bootstrapped DOM", got)
	}

	after := s.NodeCount()
	if after != s.NodeCount() {
		t.Fatalf("NodeCount() reread = %d, want stable result", s.NodeCount())
	}

	var nilSession *Session
	if got := nilSession.NodeCount(); got != 0 {
		t.Fatalf("nil NodeCount() = %d, want 0", got)
	}
}
