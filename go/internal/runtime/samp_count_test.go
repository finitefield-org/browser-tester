package runtime

import "testing"

func TestSessionSampCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><samp id="first"><kbd>alpha</kbd></samp><div id="host"></div><section><samp id="second"></samp></section></div>`,
	})

	if got := s.SampCount(); got != 2 {
		t.Fatalf("SampCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<samp id="third"></samp>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.SampCount(); got != 3 {
		t.Fatalf("SampCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.SampCount(); got != 0 {
		t.Fatalf("nil SampCount() = %d, want 0", got)
	}
}
