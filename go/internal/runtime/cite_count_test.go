package runtime

import "testing"

func TestSessionCiteCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><cite id="first"></cite><div id="host"></div><section><cite id="second"></cite></section></div>`,
	})

	if got := s.CiteCount(); got != 2 {
		t.Fatalf("CiteCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<cite id="third"></cite>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.CiteCount(); got != 3 {
		t.Fatalf("CiteCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.CiteCount(); got != 0 {
		t.Fatalf("nil CiteCount() = %d, want 0", got)
	}
}
