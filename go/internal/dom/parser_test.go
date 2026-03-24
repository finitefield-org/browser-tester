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
