package runtime

import "testing"

func TestSessionDfnCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><dfn id="first"></dfn><div id="host"></div><section><dfn id="second"></dfn></section></div>`,
	})

	if got := s.DfnCount(); got != 2 {
		t.Fatalf("DfnCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<dfn id="third"></dfn>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.DfnCount(); got != 3 {
		t.Fatalf("DfnCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.DfnCount(); got != 0 {
		t.Fatalf("nil DfnCount() = %d, want 0", got)
	}
}
