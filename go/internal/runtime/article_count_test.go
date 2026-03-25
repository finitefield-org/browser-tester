package runtime

import "testing"

func TestSessionArticleCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><article id="first"></article><div id="host"></div><section><article id="second"></article></section></div>`,
	})

	if got := s.ArticleCount(); got != 2 {
		t.Fatalf("ArticleCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<article id="third"></article>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.ArticleCount(); got != 3 {
		t.Fatalf("ArticleCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.ArticleCount(); got != 0 {
		t.Fatalf("nil ArticleCount() = %d, want 0", got)
	}
}
