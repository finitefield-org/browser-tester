package runtime

import "testing"

func TestSessionDetailsCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><details id="first"></details><div id="host"></div><section><details name="second"></details></section></main>`,
	})

	if got := s.DetailsCount(); got != 2 {
		t.Fatalf("DetailsCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<details id="third"></details>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.DetailsCount(); got != 3 {
		t.Fatalf("DetailsCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.DetailsCount(); got != 0 {
		t.Fatalf("nil DetailsCount() = %d, want 0", got)
	}
}
