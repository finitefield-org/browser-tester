package runtime

import "testing"

func TestSessionPreCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><pre id="first"></pre><div id="host"></div><section><pre id="second"></pre></section></div>`,
	})

	if got := s.PreCount(); got != 2 {
		t.Fatalf("PreCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<pre id="third"></pre>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.PreCount(); got != 3 {
		t.Fatalf("PreCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.PreCount(); got != 0 {
		t.Fatalf("nil PreCount() = %d, want 0", got)
	}
}
