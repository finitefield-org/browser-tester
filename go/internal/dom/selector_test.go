package dom

import "testing"

func TestSelectSimpleSelectors(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<div id="main">` +
			`<p class="item primary">first</p>` +
			`<p class="item">second</p>` +
			`<span class="item auxiliary">third</span>` +
			`</div>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "div", wantLen: 1},
		{selector: "#main", wantLen: 1},
		{selector: ".item", wantLen: 3},
		{selector: "p.item", wantLen: 2},
		{selector: "p.primary", wantLen: 1},
		{selector: "*.auxiliary", wantLen: 1},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectRejectsUnsupportedSelectorSyntax(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<div><p class="item">x</p></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	if _, err := store.Select("div > p"); err == nil {
		t.Fatalf("Select(\"div > p\") error = nil, want unsupported selector error")
	}
	if _, err := store.Select("p[item]"); err == nil {
		t.Fatalf("Select(\"p[item]\") error = nil, want unsupported selector error")
	}
}
