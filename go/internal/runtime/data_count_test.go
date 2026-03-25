package runtime

import "testing"

func TestSessionDataCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><data id="first"></data><div id="host"></div><section><data id="second"></data></section></div>`,
	})

	if got := s.DataCount(); got != 2 {
		t.Fatalf("DataCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<data id="third"></data>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.DataCount(); got != 3 {
		t.Fatalf("DataCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.DataCount(); got != 0 {
		t.Fatalf("nil DataCount() = %d, want 0", got)
	}
}
