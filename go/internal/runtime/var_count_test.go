package runtime

import "testing"

func TestSessionVarCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><var id="first"></var><div id="host"></div><section><var id="second"></var></section></div>`,
	})

	if got := s.VarCount(); got != 2 {
		t.Fatalf("VarCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<var id="third"></var>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.VarCount(); got != 3 {
		t.Fatalf("VarCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.VarCount(); got != 0 {
		t.Fatalf("nil VarCount() = %d, want 0", got)
	}
}
