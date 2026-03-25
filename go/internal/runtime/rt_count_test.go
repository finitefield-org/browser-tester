package runtime

import "testing"

func TestSessionRtCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><ruby id="first"><rt id="first-rt">one</rt></ruby><div id="host"></div><section><ruby id="second"><rt id="second-rt">two</rt></ruby></section></div>`,
	})

	if got := s.RtCount(); got != 2 {
		t.Fatalf("RtCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<ruby id="third"><rt id="third-rt">three</rt></ruby>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.RtCount(); got != 3 {
		t.Fatalf("RtCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.RtCount(); got != 0 {
		t.Fatalf("nil RtCount() = %d, want 0", got)
	}
}
