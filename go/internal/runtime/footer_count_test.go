package runtime

import "testing"

func TestSessionFooterCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><footer id="first"></footer><div id="host"></div><section><footer id="second"></footer></section></div>`,
	})

	if got := s.FooterCount(); got != 2 {
		t.Fatalf("FooterCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<footer id="third"></footer>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.FooterCount(); got != 3 {
		t.Fatalf("FooterCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.FooterCount(); got != 0 {
		t.Fatalf("nil FooterCount() = %d, want 0", got)
	}
}
