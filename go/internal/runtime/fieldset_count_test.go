package runtime

import "testing"

func TestSessionFieldsetCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><fieldset id="first"></fieldset><div id="host"></div><div><fieldset name="second"></fieldset></div></main>`,
	})

	if got := s.FieldsetCount(); got != 2 {
		t.Fatalf("FieldsetCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<fieldset id="third"></fieldset>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.FieldsetCount(); got != 3 {
		t.Fatalf("FieldsetCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.FieldsetCount(); got != 0 {
		t.Fatalf("nil FieldsetCount() = %d, want 0", got)
	}
}
