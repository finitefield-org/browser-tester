package runtime

import "testing"

func TestSessionNavCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><nav id="first"></nav><div id="host"></div><section><nav id="second"></nav></section></div>`,
	})

	if got := s.NavCount(); got != 2 {
		t.Fatalf("NavCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<nav id="third"></nav>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.NavCount(); got != 3 {
		t.Fatalf("NavCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.NavCount(); got != 0 {
		t.Fatalf("nil NavCount() = %d, want 0", got)
	}
}
