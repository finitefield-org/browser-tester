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

func TestSessionInlineScriptsCanReadTextContent(t *testing.T) {
	s := NewSession(DefaultSessionConfig())
	if err := s.WriteHTML(`<main><div id="src">seed</div><div id="out"></div><script>host:setTextContent("#out", expr(host:textContent("#src")))</script></main>`); err != nil {
		t.Fatalf("WriteHTML() error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><div id="src">seed</div><div id="out">seed</div><script>host:setTextContent("#out", expr(host:textContent("#src")))</script></main>`; got != want {
		t.Fatalf("DumpDOM() after textContent getter = %q, want %q", got, want)
	}
}

func TestSessionLastInlineScriptHTMLTracksBootstrapScripts(t *testing.T) {
	s := NewSession(DefaultSessionConfig())
	markup := `<main><div id="out">old</div><script id="boot">host:setInnerHTML("#out", expr(host:documentCurrentScript()))</script></main>`
	if err := s.WriteHTML(markup); err != nil {
		t.Fatalf("WriteHTML() error = %v", err)
	}

	if got, want := s.LastInlineScriptHTML(), `<script id="boot">host:setInnerHTML("#out", expr(host:documentCurrentScript()))</script>`; got != want {
		t.Fatalf("LastInlineScriptHTML() = %q, want %q", got, want)
	}
}

func TestSessionInlineScriptsCanReplaceChildrenAndCloneNode(t *testing.T) {
	s := NewSession(DefaultSessionConfig())
	if err := s.WriteHTML(`<main><div id="src"><span>old</span></div><div id="out">before</div><script>host:replaceChildren("#out", "<em>fresh</em>"); host:cloneNode("#src", true)</script></main>`); err != nil {
		t.Fatalf("WriteHTML() error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><div id="src"><span>old</span></div><div id="src"><span>old</span></div><div id="out"><em>fresh</em></div><script>host:replaceChildren("#out", "<em>fresh</em>"); host:cloneNode("#src", true)</script></main>`; got != want {
		t.Fatalf("DumpDOM() after tree mutation host helpers = %q, want %q", got, want)
	}
}
