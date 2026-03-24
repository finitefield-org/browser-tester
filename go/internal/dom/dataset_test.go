package dom

import "testing"

func TestDatasetLiveViewAndSnapshotValues(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><div id="root" data-x="1" data-foo-bar="2"></div></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	rootID := mustSelectSingle(t, store, "#root")
	dataset, err := store.Dataset(rootID)
	if err != nil {
		t.Fatalf("Dataset(#root) error = %v", err)
	}

	if got := dataset.Values(); got["x"] != "1" || got["fooBar"] != "2" || len(got) != 2 {
		t.Fatalf("Dataset.Values() = %#v, want x=1 and fooBar=2", got)
	}

	values := dataset.Values()
	values["x"] = "mutated"
	if got := dataset.Values(); got["x"] != "1" {
		t.Fatalf("Dataset.Values() should return copies, got %#v", got)
	}

	if got, ok := dataset.Get("fooBar"); !ok || got != "2" {
		t.Fatalf("Dataset.Get(fooBar) = (%q, %v), want (\"2\", true)", got, ok)
	}

	if err := dataset.Set("shipId", "92432"); err != nil {
		t.Fatalf("Dataset.Set(shipId) error = %v", err)
	}
	if got, ok := dataset.Get("shipId"); !ok || got != "92432" {
		t.Fatalf("Dataset.Get(shipId) = (%q, %v), want (\"92432\", true)", got, ok)
	}
	if got, want := store.DumpDOM(), `<main><div id="root" data-x="1" data-foo-bar="2" data-ship-id="92432"></div></main>`; got != want {
		t.Fatalf("DumpDOM() after Dataset.Set = %q, want %q", got, want)
	}

	if err := dataset.Remove("fooBar"); err != nil {
		t.Fatalf("Dataset.Remove(fooBar) error = %v", err)
	}
	if got, ok := dataset.Get("fooBar"); ok || got != "" {
		t.Fatalf("Dataset.Get(fooBar) after Remove = (%q, %v), want (\"\", false)", got, ok)
	}
	if got, want := store.DumpDOM(), `<main><div id="root" data-x="1" data-ship-id="92432"></div></main>`; got != want {
		t.Fatalf("DumpDOM() after Dataset.Remove = %q, want %q", got, want)
	}

	// Live view: external mutations should be observed.
	if err := store.SetAttribute(rootID, "data-x", "5"); err != nil {
		t.Fatalf("SetAttribute(data-x) error = %v", err)
	}
	if got, ok := dataset.Get("x"); !ok || got != "5" {
		t.Fatalf("Dataset live Get(x) = (%q, %v), want (\"5\", true)", got, ok)
	}
}

func TestDatasetRejectsInvalidNames(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<div id="root"></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}
	rootID := mustSelectSingle(t, store, "#root")
	dataset, err := store.Dataset(rootID)
	if err != nil {
		t.Fatalf("Dataset(#root) error = %v", err)
	}

	if err := dataset.Set(" ", "x"); err == nil {
		t.Fatalf("Dataset.Set(empty) error = nil, want validation error")
	}
	if err := dataset.Set("foo-bar", "x"); err == nil {
		t.Fatalf("Dataset.Set(foo-bar) error = nil, want syntax error")
	}
	if err := dataset.Remove("foo-bar"); err == nil {
		t.Fatalf("Dataset.Remove(foo-bar) error = nil, want syntax error")
	}

	if got, ok := dataset.Get(" "); ok || got != "" {
		t.Fatalf("Dataset.Get(empty) = (%q, %v), want (\"\", false)", got, ok)
	}
	if got, ok := dataset.Get("foo-bar"); ok || got != "" {
		t.Fatalf("Dataset.Get(foo-bar) = (%q, %v), want (\"\", false)", got, ok)
	}
}

func TestDatasetValuesIgnoresUppercaseDataAttributes(t *testing.T) {
	store := NewStore()
	rootID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "div",
		Attrs: []Attribute{
			{Name: "id", Value: "root", HasValue: true},
			{Name: "data-Foo", Value: "bad", HasValue: true},
			{Name: "data-foo-bar", Value: "ok", HasValue: true},
		},
	})
	store.appendChild(store.DocumentID(), rootID)

	dataset, err := store.Dataset(rootID)
	if err != nil {
		t.Fatalf("Dataset(#root) error = %v", err)
	}
	if got := dataset.Values(); got["fooBar"] != "ok" || len(got) != 1 {
		t.Fatalf("Dataset.Values() = %#v, want only fooBar=ok", got)
	}
}

func TestDatasetRejectsNilStoreInvalidNodeAndNonElement(t *testing.T) {
	var nilStore *Store
	if _, err := nilStore.Dataset(1); err == nil {
		t.Fatalf("nil Store.Dataset() error = nil, want dom store error")
	}

	store := NewStore()
	if err := store.BootstrapHTML(`<div id="root"></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}
	if _, err := store.Dataset(999); err == nil {
		t.Fatalf("Store.Dataset(invalid) error = nil, want invalid node error")
	}

	textID := store.newNode(Node{
		Kind: NodeKindText,
		Text: "x",
	})
	store.appendChild(store.DocumentID(), textID)
	if _, err := store.Dataset(textID); err == nil {
		t.Fatalf("Store.Dataset(text node) error = nil, want non-element error")
	}
}
