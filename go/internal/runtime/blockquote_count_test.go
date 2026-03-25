package runtime

import "testing"

func TestSessionBlockquoteCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><blockquote id="first"></blockquote><div id="host"></div><section><blockquote id="second"></blockquote></section></div>`,
	})

	if got := s.BlockquoteCount(); got != 2 {
		t.Fatalf("BlockquoteCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<blockquote id="third"></blockquote>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.BlockquoteCount(); got != 3 {
		t.Fatalf("BlockquoteCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.BlockquoteCount(); got != 0 {
		t.Fatalf("nil BlockquoteCount() = %d, want 0", got)
	}
}
