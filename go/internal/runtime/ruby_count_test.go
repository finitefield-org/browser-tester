package runtime

import "testing"

func TestSessionRubyCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><ruby id="first"><rt>one</rt></ruby><div id="host"></div><section><ruby id="second"></ruby></section></div>`,
	})

	if got := s.RubyCount(); got != 2 {
		t.Fatalf("RubyCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<ruby id="third"></ruby>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.RubyCount(); got != 3 {
		t.Fatalf("RubyCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.RubyCount(); got != 0 {
		t.Fatalf("nil RubyCount() = %d, want 0", got)
	}
}
