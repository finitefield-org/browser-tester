package runtime

import "testing"

func TestSessionSmallCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><small id="first"></small><div id="host"></div><section><small id="second"></small></section></div>`,
	})

	if got := s.SmallCount(); got != 2 {
		t.Fatalf("SmallCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<small id="third"></small>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.SmallCount(); got != 3 {
		t.Fatalf("SmallCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.SmallCount(); got != 0 {
		t.Fatalf("nil SmallCount() = %d, want 0", got)
	}
}
