package dom

import "testing"

func TestOptionLabelForNode(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<select><option id="labeled" label="Display">Fallback</option><option id="text">Text</option><option id="empty" label="">Used text</option></select>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	labeledID, ok, err := store.QuerySelector("#labeled")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#labeled) = (%d, %v, %v), want ok", labeledID, ok, err)
	}
	if got, want := store.OptionLabelForNode(labeledID), "Display"; got != want {
		t.Fatalf("OptionLabelForNode(#labeled) = %q, want %q", got, want)
	}

	textID, ok, err := store.QuerySelector("#text")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#text) = (%d, %v, %v), want ok", textID, ok, err)
	}
	if got, want := store.OptionLabelForNode(textID), "Text"; got != want {
		t.Fatalf("OptionLabelForNode(#text) = %q, want %q", got, want)
	}

	emptyID, ok, err := store.QuerySelector("#empty")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#empty) = (%d, %v, %v), want ok", emptyID, ok, err)
	}
	if got, want := store.OptionLabelForNode(emptyID), "Used text"; got != want {
		t.Fatalf("OptionLabelForNode(#empty) = %q, want %q", got, want)
	}

	if got := store.OptionLabelForNode(999); got != "" {
		t.Fatalf("OptionLabelForNode(invalid) = %q, want empty", got)
	}

	var nilStore *Store
	if got := nilStore.OptionLabelForNode(1); got != "" {
		t.Fatalf("nil OptionLabelForNode() = %q, want empty", got)
	}
}
