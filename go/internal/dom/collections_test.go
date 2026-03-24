package dom

import "testing"

func TestHTMLCollectionTracksElementChildren(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<div id="root"><p id="alpha"></p>text<span name="beta"></span></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	rootID := mustSelectSingle(t, store, "#root")
	children, err := store.Children(rootID)
	if err != nil {
		t.Fatalf("Children(#root) error = %v", err)
	}

	if got, want := children.Length(), 2; got != want {
		t.Fatalf("Children(#root).Length() = %d, want %d", got, want)
	}

	firstID, ok := children.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("Children(#root).Item(0) = (%d, %v), want first element", firstID, ok)
	}
	firstNode := store.Node(firstID)
	if firstNode == nil {
		t.Fatalf("Children(#root).Item(0) node = nil")
	}
	if got, want := firstNode.TagName, "p"; got != want {
		t.Fatalf("Children(#root).Item(0) tag = %q, want %q", got, want)
	}

	secondID, ok := children.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("Children(#root).Item(1) = (%d, %v), want second element", secondID, ok)
	}
	secondNode := store.Node(secondID)
	if secondNode == nil {
		t.Fatalf("Children(#root).Item(1) node = nil")
	}
	if got, want := secondNode.TagName, "span"; got != want {
		t.Fatalf("Children(#root).Item(1) tag = %q, want %q", got, want)
	}

	if got, ok := children.Item(2); ok || got != 0 {
		t.Fatalf("Children(#root).Item(2) = (%d, %v), want (0, false)", got, ok)
	}

	if got, ok := children.NamedItem("alpha"); !ok || got != firstID {
		t.Fatalf("Children(#root).NamedItem(alpha) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := children.NamedItem("beta"); !ok || got != secondID {
		t.Fatalf("Children(#root).NamedItem(beta) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := children.NamedItem("missing"); ok || got != 0 {
		t.Fatalf("Children(#root).NamedItem(missing) = (%d, %v), want (0, false)", got, ok)
	}

	ids := children.IDs()
	if len(ids) != 2 {
		t.Fatalf("Children(#root).IDs() len = %d, want 2", len(ids))
	}
	ids[0] = 999
	if got, ok := children.Item(0); !ok || got != firstID {
		t.Fatalf("Children(#root) mutated via IDs() = (%d, %v), want (%d, true)", got, ok, firstID)
	}

	textID := store.newNode(Node{
		Kind: NodeKindText,
		Text: "more",
	})
	store.appendChild(rootID, textID)

	buttonID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "button",
		Attrs: []Attribute{
			{Name: "id", Value: "gamma", HasValue: true},
		},
	})
	store.appendChild(rootID, buttonID)

	if got, want := children.Length(), 3; got != want {
		t.Fatalf("Children(#root).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := children.NamedItem("gamma"); !ok || got != buttonID {
		t.Fatalf("Children(#root).NamedItem(gamma) = (%d, %v), want (%d, true)", got, ok, buttonID)
	}
}

func TestHTMLCollectionTracksDocumentChildren(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<div id="first"></div>text<p id="second"></p>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	children, err := store.Children(store.DocumentID())
	if err != nil {
		t.Fatalf("Children(document) error = %v", err)
	}

	if got, want := children.Length(), 2; got != want {
		t.Fatalf("Children(document).Length() = %d, want %d", got, want)
	}

	firstID, ok := children.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("Children(document).Item(0) = (%d, %v), want first element", firstID, ok)
	}
	secondID, ok := children.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("Children(document).Item(1) = (%d, %v), want second element", secondID, ok)
	}
	firstNode := store.Node(firstID)
	if firstNode == nil {
		t.Fatalf("Children(document).Item(0) node = nil")
	}
	if got, want := firstNode.TagName, "div"; got != want {
		t.Fatalf("Children(document).Item(0) tag = %q, want %q", got, want)
	}
	secondNode := store.Node(secondID)
	if secondNode == nil {
		t.Fatalf("Children(document).Item(1) node = nil")
	}
	if got, want := secondNode.TagName, "p"; got != want {
		t.Fatalf("Children(document).Item(1) tag = %q, want %q", got, want)
	}

	sectionID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "section",
		Attrs: []Attribute{
			{Name: "id", Value: "third", HasValue: true},
		},
	})
	store.appendChild(store.DocumentID(), sectionID)

	textID := store.newNode(Node{
		Kind: NodeKindText,
		Text: "ignored",
	})
	store.appendChild(store.DocumentID(), textID)

	if got, want := children.Length(), 3; got != want {
		t.Fatalf("Children(document).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := children.NamedItem("third"); !ok || got != sectionID {
		t.Fatalf("Children(document).NamedItem(third) = (%d, %v), want (%d, true)", got, ok, sectionID)
	}
}

func TestStoreChildrenRejectsUnsupportedNodes(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><p id="one">x</p></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	var nilStore *Store
	if _, err := nilStore.Children(1); err == nil {
		t.Fatalf("nil Children() error = nil, want dom store error")
	}

	if _, err := store.Children(999); err == nil {
		t.Fatalf("Children(invalid node) error = nil, want invalid node error")
	}

	textID := store.newNode(Node{
		Kind: NodeKindText,
		Text: "text",
	})
	if _, err := store.Children(textID); err == nil {
		t.Fatalf("Children(text node) error = nil, want unsupported node error")
	}
}
