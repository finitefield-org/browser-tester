package dom

import "testing"

func TestAttributeReflectionRoundTrip(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<div id="root" data-x="1"></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	rootID := mustSelectSingle(t, store, "#root")

	if got, ok, err := store.GetAttribute(rootID, "data-x"); err != nil || !ok || got != "1" {
		t.Fatalf("GetAttribute(data-x) = (%q, %v, %v), want (\"1\", true, nil)", got, ok, err)
	}

	if ok, err := store.HasAttribute(rootID, "data-x"); err != nil || !ok {
		t.Fatalf("HasAttribute(data-x) = (%v, %v), want (true, nil)", ok, err)
	}

	if err := store.SetAttribute(rootID, "data-x", "2"); err != nil {
		t.Fatalf("SetAttribute(data-x) error = %v", err)
	}
	if got, ok, err := store.GetAttribute(rootID, "data-x"); err != nil || !ok || got != "2" {
		t.Fatalf("GetAttribute(data-x) after SetAttribute = (%q, %v, %v), want (\"2\", true, nil)", got, ok, err)
	}

	if err := store.RemoveAttribute(rootID, "data-x"); err != nil {
		t.Fatalf("RemoveAttribute(data-x) error = %v", err)
	}
	if got, ok, err := store.GetAttribute(rootID, "data-x"); err != nil || ok || got != "" {
		t.Fatalf("GetAttribute(data-x) after RemoveAttribute = (%q, %v, %v), want (\"\", false, nil)", got, ok, err)
	}
	if ok, err := store.HasAttribute(rootID, "data-x"); err != nil || ok {
		t.Fatalf("HasAttribute(data-x) after RemoveAttribute = (%v, %v), want (false, nil)", ok, err)
	}
}

func TestAttributeReflectionHandlesMissingAttributesAndNormalization(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<div id="root"></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	rootID := mustSelectSingle(t, store, "#root")

	if got, ok, err := store.GetAttribute(rootID, "missing"); err != nil || ok || got != "" {
		t.Fatalf("GetAttribute(missing) = (%q, %v, %v), want (\"\", false, nil)", got, ok, err)
	}

	// Exercise trimming + lowercase normalization.
	if err := store.SetAttribute(rootID, " DATA-X ", "1"); err != nil {
		t.Fatalf("SetAttribute(DATA-X) error = %v", err)
	}
	if got, ok, err := store.GetAttribute(rootID, "data-x"); err != nil || !ok || got != "1" {
		t.Fatalf("GetAttribute(data-x) after normalized set = (%q, %v, %v), want (\"1\", true, nil)", got, ok, err)
	}
}

func TestAttributeReflectionMutatesDOMSerializationViaSelectorLookup(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><div id="root"></div></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	rootID := mustSelectSingle(t, store, "main > div#root")

	if err := store.SetAttribute(rootID, "data-x", "1"); err != nil {
		t.Fatalf("SetAttribute(data-x) error = %v", err)
	}
	if got, want := store.DumpDOM(), `<main><div id="root" data-x="1"></div></main>`; got != want {
		t.Fatalf("DumpDOM() after SetAttribute = %q, want %q", got, want)
	}

	if err := store.RemoveAttribute(rootID, "data-x"); err != nil {
		t.Fatalf("RemoveAttribute(data-x) error = %v", err)
	}
	if got, want := store.DumpDOM(), `<main><div id="root"></div></main>`; got != want {
		t.Fatalf("DumpDOM() after RemoveAttribute = %q, want %q", got, want)
	}
}

func TestAttributeReflectionRejectsNilStoreInvalidNodeAndEmptyName(t *testing.T) {
	var nilStore *Store
	if _, _, err := nilStore.GetAttribute(1, "id"); err == nil {
		t.Fatalf("nil GetAttribute() error = nil, want dom store error")
	}
	if _, err := nilStore.HasAttribute(1, "id"); err == nil {
		t.Fatalf("nil HasAttribute() error = nil, want dom store error")
	}
	if err := nilStore.SetAttribute(1, "id", "x"); err == nil {
		t.Fatalf("nil SetAttribute() error = nil, want dom store error")
	}
	if err := nilStore.RemoveAttribute(1, "id"); err == nil {
		t.Fatalf("nil RemoveAttribute() error = nil, want dom store error")
	}

	store := NewStore()
	if err := store.BootstrapHTML(`<div id="root"></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	if _, _, err := store.GetAttribute(999, "id"); err == nil {
		t.Fatalf("GetAttribute(invalid node) error = nil, want invalid node error")
	}
	if _, err := store.HasAttribute(999, "id"); err == nil {
		t.Fatalf("HasAttribute(invalid node) error = nil, want invalid node error")
	}
	if err := store.SetAttribute(999, "id", "x"); err == nil {
		t.Fatalf("SetAttribute(invalid node) error = nil, want invalid node error")
	}
	if err := store.RemoveAttribute(999, "id"); err == nil {
		t.Fatalf("RemoveAttribute(invalid node) error = nil, want invalid node error")
	}

	rootID := mustSelectSingle(t, store, "#root")
	if _, _, err := store.GetAttribute(rootID, " "); err == nil {
		t.Fatalf("GetAttribute(empty name) error = nil, want validation error")
	}
	if _, err := store.HasAttribute(rootID, " "); err == nil {
		t.Fatalf("HasAttribute(empty name) error = nil, want validation error")
	}
	if err := store.SetAttribute(rootID, " ", "x"); err == nil {
		t.Fatalf("SetAttribute(empty name) error = nil, want validation error")
	}
	if err := store.RemoveAttribute(rootID, " "); err == nil {
		t.Fatalf("RemoveAttribute(empty name) error = nil, want validation error")
	}
}

func TestAttributeReflectionRejectsNonElementNodes(t *testing.T) {
	store := NewStore()

	textID := store.newNode(Node{
		Kind: NodeKindText,
		Text: "x",
	})
	store.appendChild(store.DocumentID(), textID)

	if _, _, err := store.GetAttribute(textID, "id"); err == nil {
		t.Fatalf("GetAttribute(text node) error = nil, want non-element error")
	}
	if _, err := store.HasAttribute(textID, "id"); err == nil {
		t.Fatalf("HasAttribute(text node) error = nil, want non-element error")
	}
	if err := store.SetAttribute(textID, "id", "x"); err == nil {
		t.Fatalf("SetAttribute(text node) error = nil, want non-element error")
	}
	if err := store.RemoveAttribute(textID, "id"); err == nil {
		t.Fatalf("RemoveAttribute(text node) error = nil, want non-element error")
	}
}
