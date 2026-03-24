package dom

import "testing"

func TestBootstrapHTMLRejectsInvalidMarkup(t *testing.T) {
	store := NewStore()

	if err := store.BootstrapHTML(`<div class="broken></div>`); err == nil {
		t.Fatalf("BootstrapHTML() error = nil, want malformed attribute error")
	}

	if err := store.BootstrapHTML(`</div>`); err == nil {
		t.Fatalf("BootstrapHTML() error = nil, want unexpected closing tag error")
	}
}

func TestBootstrapHTMLRollsBackOnFailure(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<div id="root"><span>ok</span></div>`); err != nil {
		t.Fatalf("BootstrapHTML(valid) error = %v", err)
	}

	before := store.DumpDOM()
	if err := store.BootstrapHTML(`<div><span></div>`); err == nil {
		t.Fatalf("BootstrapHTML(invalid) error = nil, want parse error")
	}

	if got := store.DumpDOM(); got != before {
		t.Fatalf("DumpDOM() after failed parse = %q, want %q", got, before)
	}
	if got, want := store.SourceHTML(), `<div id="root"><span>ok</span></div>`; got != want {
		t.Fatalf("SourceHTML() after failed parse = %q, want %q", got, want)
	}
}

func TestBootstrapHTMLAllowsSlashInUnquotedAttributeValue(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<img src=https://example.test/a.png>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	nodes, err := store.Select("img")
	if err != nil {
		t.Fatalf("Select(img) error = %v", err)
	}
	if len(nodes) != 1 {
		t.Fatalf("Select(img) len = %d, want 1", len(nodes))
	}

	node := store.Node(nodes[0])
	if node == nil {
		t.Fatalf("Node(img) = nil")
	}
	if got, ok := attributeValue(node.Attrs, "src"); !ok || got != "https://example.test/a.png" {
		t.Fatalf("img src = (%q, %v), want (%q, true)", got, ok, "https://example.test/a.png")
	}
}

func TestBootstrapHTMLPreservesRawScriptText(t *testing.T) {
	input := `<main><script>host:setInnerHTML("#target", "<em>updated</em>")</script></main>`
	store := NewStore()
	if err := store.BootstrapHTML(input); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	if got, want := store.DumpDOM(), input; got != want {
		t.Fatalf("DumpDOM() = %q, want %q", got, want)
	}

	scriptID := mustSelectSingle(t, store, "script")
	if got, want := store.TextContentForNode(scriptID), `host:setInnerHTML("#target", "<em>updated</em>")`; got != want {
		t.Fatalf("TextContentForNode(script) = %q, want %q", got, want)
	}
}

func TestBootstrapHTMLRejectsUnterminatedScriptElement(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><script>host:setInnerHTML("#target", "<em>updated</em>")</main>`); err == nil {
		t.Fatalf("BootstrapHTML(unterminated script) error = nil, want script parse error")
	}
}
