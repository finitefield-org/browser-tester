package runtime

import "testing"

func TestSessionScriptCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><script></script><div id="host"></div><script>host:setTextContent(#host, changed)</script></main>`,
	})

	if got := s.ScriptCount(); got != 2 {
		t.Fatalf("ScriptCount() = %d, want 2", got)
	}

	after := s.ScriptCount()
	if after != s.ScriptCount() {
		t.Fatalf("ScriptCount() reread = %d, want stable result", s.ScriptCount())
	}

	if err := s.SetInnerHTML("#host", "<script></script>"); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.ScriptCount(); got != 3 {
		t.Fatalf("ScriptCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.ScriptCount(); got != 0 {
		t.Fatalf("nil ScriptCount() = %d, want 0", got)
	}
}
