package runtime

import "testing"

func TestSessionMainCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><main id="first"></main><div id="host"></div><section><main id="second"></main></section></div>`,
	})

	if got := s.MainCount(); got != 2 {
		t.Fatalf("MainCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<main id="third"></main>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.MainCount(); got != 3 {
		t.Fatalf("MainCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.MainCount(); got != 0 {
		t.Fatalf("nil MainCount() = %d, want 0", got)
	}
}
