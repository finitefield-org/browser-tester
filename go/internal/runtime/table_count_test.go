package runtime

import "testing"

func TestSessionTableCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><table id="first"></table><div id="host"></div><div><table name="second"></table></div></main>`,
	})

	if got := s.TableCount(); got != 2 {
		t.Fatalf("TableCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<table id="third"></table>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.TableCount(); got != 3 {
		t.Fatalf("TableCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.TableCount(); got != 0 {
		t.Fatalf("nil TableCount() = %d, want 0", got)
	}
}
