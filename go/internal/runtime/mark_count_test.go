package runtime

import "testing"

func TestSessionMarkCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><mark id="first"></mark><div id="host"></div><section><mark id="second"></mark></section></div>`,
	})

	if got := s.MarkCount(); got != 2 {
		t.Fatalf("MarkCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<mark id="third"></mark>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.MarkCount(); got != 3 {
		t.Fatalf("MarkCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.MarkCount(); got != 0 {
		t.Fatalf("nil MarkCount() = %d, want 0", got)
	}
}
