package runtime

import "testing"

func TestSessionQCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><q id="first"></q><div id="host"></div><section><q id="second"></q></section></div>`,
	})

	if got := s.QCount(); got != 2 {
		t.Fatalf("QCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<q id="third"></q>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.QCount(); got != 3 {
		t.Fatalf("QCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.QCount(); got != 0 {
		t.Fatalf("nil QCount() = %d, want 0", got)
	}
}
