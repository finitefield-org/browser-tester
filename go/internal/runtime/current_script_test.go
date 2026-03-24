package runtime

import "testing"

func TestSessionDocumentCurrentScriptTracksBootstrapScripts(t *testing.T) {
	s := NewSession(DefaultSessionConfig())
	if err := s.WriteHTML(`<main><div id="out">old</div><script id="boot">host:setInnerHTML("#out", expr(host:documentCurrentScript()))</script></main>`); err != nil {
		t.Fatalf("WriteHTML() error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><div id="out"><script id="boot">host:setInnerHTML("#out", expr(host:documentCurrentScript()))</script></div><script id="boot">host:setInnerHTML("#out", expr(host:documentCurrentScript()))</script></main>`; got != want {
		t.Fatalf("DumpDOM() after bootstrap currentScript = %q, want %q", got, want)
	}
	if got := s.documentCurrentScript(); got != "" {
		t.Fatalf("documentCurrentScript() after bootstrap = %q, want empty", got)
	}
}

func TestSessionDocumentCurrentScriptIsEmptyForEventHandlers(t *testing.T) {
	s := NewSession(DefaultSessionConfig())
	if err := s.WriteHTML(`<main><button id="btn">Go</button><div id="out">old</div><script>host:addEventListener("#btn", "click", 'host:setInnerHTML("#out", expr(host:documentCurrentScript()))')</script></main>`); err != nil {
		t.Fatalf("WriteHTML() error = %v", err)
	}

	if err := s.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><button id="btn">Go</button><div id="out"></div><script>host:addEventListener("#btn", "click", 'host:setInnerHTML("#out", expr(host:documentCurrentScript()))')</script></main>`; got != want {
		t.Fatalf("DumpDOM() after click currentScript = %q, want %q", got, want)
	}
	if got := s.documentCurrentScript(); got != "" {
		t.Fatalf("documentCurrentScript() after click = %q, want empty", got)
	}
}
