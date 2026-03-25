package runtime

import "testing"

func TestSessionIframeCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><iframe id="first"></iframe><div id="host"></div><section><iframe name="second"></iframe></section></main>`,
	})

	if got := s.IframeCount(); got != 2 {
		t.Fatalf("IframeCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<iframe id="third"></iframe>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.IframeCount(); got != 3 {
		t.Fatalf("IframeCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.IframeCount(); got != 0 {
		t.Fatalf("nil IframeCount() = %d, want 0", got)
	}
}
