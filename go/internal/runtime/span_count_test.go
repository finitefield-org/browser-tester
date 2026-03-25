package runtime

import "testing"

func TestSessionSpanCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><span id="first"></span><div id="host"></div><section><span id="second"></span></section></div>`,
	})

	if got := s.SpanCount(); got != 2 {
		t.Fatalf("SpanCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<span id="third"></span>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.SpanCount(); got != 3 {
		t.Fatalf("SpanCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.SpanCount(); got != 0 {
		t.Fatalf("nil SpanCount() = %d, want 0", got)
	}
}
