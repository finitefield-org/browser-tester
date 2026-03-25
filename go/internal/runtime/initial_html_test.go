package runtime

import "testing"

func TestSessionInitialHTMLInspection(t *testing.T) {
	s := NewSession(SessionConfig{HTML: `<main><script>host:setInnerHTML("#out", "ok")</script></main>`})
	if got, want := s.InitialHTML(), `<main><script>host:setInnerHTML("#out", "ok")</script></main>`; got != want {
		t.Fatalf("InitialHTML() = %q, want %q", got, want)
	}
	if got := s.DOMReady(); got {
		t.Fatalf("DOMReady() after InitialHTML() = %v, want false", got)
	}
	if got := s.DOMError(); got != "" {
		t.Fatalf("DOMError() after InitialHTML() = %q, want empty", got)
	}
}
