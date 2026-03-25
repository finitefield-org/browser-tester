package runtime

import "testing"

func TestSessionTimeCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><time id="first"></time><div id="host"></div><section><time id="second"></time></section></div>`,
	})

	if got := s.TimeCount(); got != 2 {
		t.Fatalf("TimeCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<time id="third"></time>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.TimeCount(); got != 3 {
		t.Fatalf("TimeCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.TimeCount(); got != 0 {
		t.Fatalf("nil TimeCount() = %d, want 0", got)
	}
}
