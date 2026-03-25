package runtime

import "testing"

func TestSessionTemplateCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><template id="first"></template><div id="host"></div><div><template name="second"></template></div></main>`,
	})

	if got := s.TemplateCount(); got != 2 {
		t.Fatalf("TemplateCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<template id="third"></template>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.TemplateCount(); got != 3 {
		t.Fatalf("TemplateCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.TemplateCount(); got != 0 {
		t.Fatalf("nil TemplateCount() = %d, want 0", got)
	}
}
