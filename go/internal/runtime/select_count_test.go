package runtime

import "testing"

func TestSessionSelectCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><select id="first"></select><div id="host"></div><div><select name="second"></select></div></main>`,
	})

	if got := s.SelectCount(); got != 2 {
		t.Fatalf("SelectCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<select id="third"></select>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.SelectCount(); got != 3 {
		t.Fatalf("SelectCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.SelectCount(); got != 0 {
		t.Fatalf("nil SelectCount() = %d, want 0", got)
	}
}
