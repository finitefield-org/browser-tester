package dom

import "testing"

func TestClassListLiveViewAndSnapshotValues(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><div id="root" class="a  b	c"></div></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	rootID := mustSelectSingle(t, store, "#root")
	classList, err := store.ClassList(rootID)
	if err != nil {
		t.Fatalf("ClassList(#root) error = %v", err)
	}

	if got := classList.Values(); len(got) != 3 || got[0] != "a" || got[1] != "b" || got[2] != "c" {
		t.Fatalf("ClassList.Values() = %#v, want [a b c]", got)
	}
	if !classList.Contains("b") {
		t.Fatalf("ClassList.Contains(b) = false, want true")
	}
	if classList.Contains("missing") {
		t.Fatalf("ClassList.Contains(missing) = true, want false")
	}

	values := classList.Values()
	values[0] = "mutated"
	if got := classList.Values(); got[0] != "a" {
		t.Fatalf("ClassList.Values() should return copies, got %#v", got)
	}

	// Add should ignore duplicates and append new tokens.
	if err := classList.Add("b", "d"); err != nil {
		t.Fatalf("ClassList.Add() error = %v", err)
	}
	if got, want := store.DumpDOM(), `<main><div id="root" class="a b c d"></div></main>`; got != want {
		t.Fatalf("DumpDOM() after Add = %q, want %q", got, want)
	}

	// Remove should remove tokens and keep the attribute present if it existed.
	if err := classList.Remove("a", "c"); err != nil {
		t.Fatalf("ClassList.Remove() error = %v", err)
	}
	if got, want := store.DumpDOM(), `<main><div id="root" class="b d"></div></main>`; got != want {
		t.Fatalf("DumpDOM() after Remove = %q, want %q", got, want)
	}

	// Live view: external mutations should be observed.
	if err := store.SetAttribute(rootID, "class", "x y"); err != nil {
		t.Fatalf("SetAttribute(class) error = %v", err)
	}
	if got := classList.Values(); len(got) != 2 || got[0] != "x" || got[1] != "y" {
		t.Fatalf("ClassList live Values() = %#v, want [x y]", got)
	}
}

func TestClassListRejectsInvalidTokens(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<div id="root" class="a"></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}
	rootID := mustSelectSingle(t, store, "#root")
	classList, err := store.ClassList(rootID)
	if err != nil {
		t.Fatalf("ClassList(#root) error = %v", err)
	}

	if err := classList.Add(" "); err == nil {
		t.Fatalf("ClassList.Add(empty) error = nil, want validation error")
	}
	if err := classList.Add("a b"); err == nil {
		t.Fatalf("ClassList.Add(with space) error = nil, want validation error")
	}
	if err := classList.Remove("a b"); err == nil {
		t.Fatalf("ClassList.Remove(with space) error = nil, want validation error")
	}
}

func TestClassListRejectsNilStoreInvalidNodeAndNonElement(t *testing.T) {
	var nilStore *Store
	if _, err := nilStore.ClassList(1); err == nil {
		t.Fatalf("nil Store.ClassList() error = nil, want dom store error")
	}

	store := NewStore()
	if err := store.BootstrapHTML(`<div id="root"></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}
	if _, err := store.ClassList(999); err == nil {
		t.Fatalf("Store.ClassList(invalid) error = nil, want invalid node error")
	}

	textID := store.newNode(Node{
		Kind: NodeKindText,
		Text: "x",
	})
	store.appendChild(store.DocumentID(), textID)
	if _, err := store.ClassList(textID); err == nil {
		t.Fatalf("Store.ClassList(text node) error = nil, want non-element error")
	}
}
