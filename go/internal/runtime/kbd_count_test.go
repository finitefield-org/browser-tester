package runtime

import "testing"

func TestSessionKbdCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><kbd id="first"></kbd><div id="host"></div><section><kbd id="second"></kbd></section></div>`,
	})

	if got := s.KbdCount(); got != 2 {
		t.Fatalf("KbdCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<kbd id="third"></kbd>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.KbdCount(); got != 3 {
		t.Fatalf("KbdCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.KbdCount(); got != 0 {
		t.Fatalf("nil KbdCount() = %d, want 0", got)
	}
}
