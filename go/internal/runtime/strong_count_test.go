package runtime

import "testing"

func TestSessionStrongCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><strong id="first"></strong><div id="host"></div><section><strong id="second"></strong></section></div>`,
	})

	if got := s.StrongCount(); got != 2 {
		t.Fatalf("StrongCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<strong id="third"></strong>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.StrongCount(); got != 3 {
		t.Fatalf("StrongCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.StrongCount(); got != 0 {
		t.Fatalf("nil StrongCount() = %d, want 0", got)
	}
}
