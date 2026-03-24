package dom

import "testing"

func TestBootstrapHTMLRoundTripAndSerialization(t *testing.T) {
	input := `<div id="root"><p class="copy">Hello <span>world</span></p><br></div>`
	store := NewStore()
	if err := store.BootstrapHTML(input); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	if got, want := store.SourceHTML(), input; got != want {
		t.Fatalf("SourceHTML() = %q, want %q", got, want)
	}

	dump := store.DumpDOM()
	if got, want := dump, input; got != want {
		t.Fatalf("DumpDOM() = %q, want %q", got, want)
	}

	roots, err := store.Select("#root")
	if err != nil {
		t.Fatalf("Select(#root) error = %v", err)
	}
	if len(roots) != 1 {
		t.Fatalf("Select(#root) len = %d, want 1", len(roots))
	}

	outer, err := store.OuterHTMLForNode(roots[0])
	if err != nil {
		t.Fatalf("OuterHTMLForNode() error = %v", err)
	}
	if got, want := outer, `<div id="root"><p class="copy">Hello <span>world</span></p><br></div>`; got != want {
		t.Fatalf("OuterHTMLForNode() = %q, want %q", got, want)
	}

	copyStore := NewStore()
	if err := copyStore.BootstrapHTML(dump); err != nil {
		t.Fatalf("BootstrapHTML(roundtrip) error = %v", err)
	}
	if got, want := copyStore.DumpDOM(), dump; got != want {
		t.Fatalf("DumpDOM(roundtrip) = %q, want %q", got, want)
	}
}

func TestTextContentForNode(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<section id="main">Hello <strong>DOM</strong> test</section>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}
	nodes, err := store.Select("#main")
	if err != nil {
		t.Fatalf("Select() error = %v", err)
	}
	if len(nodes) != 1 {
		t.Fatalf("Select(#main) len = %d, want 1", len(nodes))
	}
	if got, want := store.TextContentForNode(nodes[0]), "Hello DOM test"; got != want {
		t.Fatalf("TextContentForNode() = %q, want %q", got, want)
	}
}
