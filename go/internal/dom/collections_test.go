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

func TestChildNodeListTracksDocumentChildNodes(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<div id="first"></div>text<p id="second"></p>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	nodes, err := store.ChildNodes(store.DocumentID())
	if err != nil {
		t.Fatalf("ChildNodes(document) error = %v", err)
	}

	if got, want := nodes.Length(), 3; got != want {
		t.Fatalf("ChildNodes(document).Length() = %d, want %d", got, want)
	}

	firstID, ok := nodes.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("ChildNodes(document).Item(0) = (%d, %v), want first element", firstID, ok)
	}
	firstNode := store.Node(firstID)
	if firstNode == nil {
		t.Fatalf("ChildNodes(document).Item(0) node = nil")
	}
	if got, want := firstNode.TagName, "div"; got != want {
		t.Fatalf("ChildNodes(document).Item(0) tag = %q, want %q", got, want)
	}

	textID, ok := nodes.Item(1)
	if !ok || textID == 0 {
		t.Fatalf("ChildNodes(document).Item(1) = (%d, %v), want text node", textID, ok)
	}
	textNode := store.Node(textID)
	if textNode == nil {
		t.Fatalf("ChildNodes(document).Item(1) node = nil")
	}
	if textNode.Kind != NodeKindText {
		t.Fatalf("ChildNodes(document).Item(1).Kind = %v, want text", textNode.Kind)
	}

	secondID, ok := nodes.Item(2)
	if !ok || secondID == 0 {
		t.Fatalf("ChildNodes(document).Item(2) = (%d, %v), want second element", secondID, ok)
	}
	secondNode := store.Node(secondID)
	if secondNode == nil {
		t.Fatalf("ChildNodes(document).Item(2) node = nil")
	}
	if got, want := secondNode.TagName, "p"; got != want {
		t.Fatalf("ChildNodes(document).Item(2) tag = %q, want %q", got, want)
	}

	ids := nodes.IDs()
	if len(ids) != 3 {
		t.Fatalf("ChildNodes(document).IDs() len = %d, want 3", len(ids))
	}
	ids[0] = 999
	if got, ok := nodes.Item(0); !ok || got != firstID {
		t.Fatalf("ChildNodes(document) mutated via IDs() = (%d, %v), want (%d, true)", got, ok, firstID)
	}

	text2ID := store.newNode(Node{
		Kind: NodeKindText,
		Text: "more",
	})
	store.appendChild(store.DocumentID(), text2ID)

	if got, want := nodes.Length(), 4; got != want {
		t.Fatalf("ChildNodes(document).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := nodes.Item(3); !ok || got != text2ID {
		t.Fatalf("ChildNodes(document).Item(3) = (%d, %v), want (%d, true)", got, ok, text2ID)
	}
}

func TestChildNodeListTracksTemplateContentChildNodes(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<template id="tpl"><span id="first"></span>text<p id="second"></p></template>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	templateID := mustSelectSingle(t, store, "#tpl")
	nodes, err := store.TemplateContentChildNodes(templateID)
	if err != nil {
		t.Fatalf("TemplateContentChildNodes(#tpl) error = %v", err)
	}

	if got, want := nodes.Length(), 3; got != want {
		t.Fatalf("TemplateContentChildNodes(#tpl).Length() = %d, want %d", got, want)
	}

	firstID, ok := nodes.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("TemplateContentChildNodes(#tpl).Item(0) = (%d, %v), want first element", firstID, ok)
	}
	firstNode := store.Node(firstID)
	if firstNode == nil {
		t.Fatalf("TemplateContentChildNodes(#tpl).Item(0) node = nil")
	}
	if got, want := firstNode.TagName, "span"; got != want {
		t.Fatalf("TemplateContentChildNodes(#tpl).Item(0) tag = %q, want %q", got, want)
	}

	textID, ok := nodes.Item(1)
	if !ok || textID == 0 {
		t.Fatalf("TemplateContentChildNodes(#tpl).Item(1) = (%d, %v), want text node", textID, ok)
	}
	textNode := store.Node(textID)
	if textNode == nil {
		t.Fatalf("TemplateContentChildNodes(#tpl).Item(1) node = nil")
	}
	if textNode.Kind != NodeKindText {
		t.Fatalf("TemplateContentChildNodes(#tpl).Item(1).Kind = %v, want text", textNode.Kind)
	}

	secondID, ok := nodes.Item(2)
	if !ok || secondID == 0 {
		t.Fatalf("TemplateContentChildNodes(#tpl).Item(2) = (%d, %v), want second element", secondID, ok)
	}
	secondNode := store.Node(secondID)
	if secondNode == nil {
		t.Fatalf("TemplateContentChildNodes(#tpl).Item(2) node = nil")
	}
	if got, want := secondNode.TagName, "p"; got != want {
		t.Fatalf("TemplateContentChildNodes(#tpl).Item(2) tag = %q, want %q", got, want)
	}

	ids := nodes.IDs()
	if len(ids) != 3 {
		t.Fatalf("TemplateContentChildNodes(#tpl).IDs() len = %d, want 3", len(ids))
	}
	ids[0] = 999
	if got, ok := nodes.Item(0); !ok || got != firstID {
		t.Fatalf("TemplateContentChildNodes(#tpl) mutated via IDs() = (%d, %v), want (%d, true)", got, ok, firstID)
	}

	bonusID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "em",
		Attrs: []Attribute{
			{Name: "id", Value: "bonus", HasValue: true},
		},
	})
	store.appendChild(templateID, bonusID)

	if got, want := nodes.Length(), 4; got != want {
		t.Fatalf("TemplateContentChildNodes(#tpl).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := nodes.Item(3); !ok || got != bonusID {
		t.Fatalf("TemplateContentChildNodes(#tpl).Item(3) = (%d, %v), want (%d, true)", got, ok, bonusID)
	}
}

func TestHTMLCollectionTracksDocumentScripts(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<script id="first"></script><div><script name="second"></script></div><p>text</p>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	scripts, err := store.Scripts()
	if err != nil {
		t.Fatalf("Scripts(document) error = %v", err)
	}

	if got, want := scripts.Length(), 2; got != want {
		t.Fatalf("Scripts(document).Length() = %d, want %d", got, want)
	}

	firstID, ok := scripts.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("Scripts(document).Item(0) = (%d, %v), want first script", firstID, ok)
	}
	firstNode := store.Node(firstID)
	if firstNode == nil {
		t.Fatalf("Scripts(document).Item(0) node = nil")
	}
	if got, want := firstNode.TagName, "script"; got != want {
		t.Fatalf("Scripts(document).Item(0) tag = %q, want %q", got, want)
	}

	secondID, ok := scripts.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("Scripts(document).Item(1) = (%d, %v), want second script", secondID, ok)
	}
	secondNode := store.Node(secondID)
	if secondNode == nil {
		t.Fatalf("Scripts(document).Item(1) node = nil")
	}
	if got, want := secondNode.TagName, "script"; got != want {
		t.Fatalf("Scripts(document).Item(1) tag = %q, want %q", got, want)
	}

	if got, ok := scripts.NamedItem("first"); !ok || got != firstID {
		t.Fatalf("Scripts(document).NamedItem(first) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := scripts.NamedItem("second"); !ok || got != secondID {
		t.Fatalf("Scripts(document).NamedItem(second) = (%d, %v), want (%d, true)", got, ok, secondID)
	}

	scriptsIDs := scripts.IDs()
	if len(scriptsIDs) != 2 {
		t.Fatalf("Scripts(document).IDs() len = %d, want 2", len(scriptsIDs))
	}
	scriptsIDs[0] = 999
	if got, ok := scripts.Item(0); !ok || got != firstID {
		t.Fatalf("Scripts(document) mutated via IDs() = (%d, %v), want (%d, true)", got, ok, firstID)
	}

	scriptID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "script",
		Attrs: []Attribute{
			{Name: "id", Value: "third", HasValue: true},
		},
	})
	store.appendChild(store.DocumentID(), scriptID)

	if got, want := scripts.Length(), 3; got != want {
		t.Fatalf("Scripts(document).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := scripts.NamedItem("third"); !ok || got != scriptID {
		t.Fatalf("Scripts(document).NamedItem(third) = (%d, %v), want (%d, true)", got, ok, scriptID)
	}
}

func TestHTMLCollectionTracksDocumentImages(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<img id="first" src="/a"><div><img name="second" src="/b"></div><p>text</p>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	images, err := store.Images()
	if err != nil {
		t.Fatalf("Images(document) error = %v", err)
	}

	if got, want := images.Length(), 2; got != want {
		t.Fatalf("Images(document).Length() = %d, want %d", got, want)
	}

	firstID, ok := images.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("Images(document).Item(0) = (%d, %v), want first image", firstID, ok)
	}
	firstNode := store.Node(firstID)
	if firstNode == nil {
		t.Fatalf("Images(document).Item(0) node = nil")
	}
	if got, want := firstNode.TagName, "img"; got != want {
		t.Fatalf("Images(document).Item(0) tag = %q, want %q", got, want)
	}

	secondID, ok := images.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("Images(document).Item(1) = (%d, %v), want second image", secondID, ok)
	}
	secondNode := store.Node(secondID)
	if secondNode == nil {
		t.Fatalf("Images(document).Item(1) node = nil")
	}
	if got, want := secondNode.TagName, "img"; got != want {
		t.Fatalf("Images(document).Item(1) tag = %q, want %q", got, want)
	}

	if got, ok := images.NamedItem("first"); !ok || got != firstID {
		t.Fatalf("Images(document).NamedItem(first) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := images.NamedItem("second"); !ok || got != secondID {
		t.Fatalf("Images(document).NamedItem(second) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := images.NamedItem("missing"); ok || got != 0 {
		t.Fatalf("Images(document).NamedItem(missing) = (%d, %v), want (0, false)", got, ok)
	}

	ids := images.IDs()
	if len(ids) != 2 {
		t.Fatalf("Images(document).IDs() len = %d, want 2", len(ids))
	}
	ids[0] = 999
	if got, ok := images.Item(0); !ok || got != firstID {
		t.Fatalf("Images(document) mutated via IDs() = (%d, %v), want (%d, true)", got, ok, firstID)
	}

	imageID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "img",
		Attrs: []Attribute{
			{Name: "name", Value: "third", HasValue: true},
		},
	})
	store.appendChild(store.DocumentID(), imageID)

	if got, want := images.Length(), 3; got != want {
		t.Fatalf("Images(document).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := images.NamedItem("third"); !ok || got != imageID {
		t.Fatalf("Images(document).NamedItem(third) = (%d, %v), want (%d, true)", got, ok, imageID)
	}
}

func TestHTMLCollectionTracksDocumentForms(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<form id="first"></form><div><form name="second"></form></div><p>text</p>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	forms, err := store.Forms()
	if err != nil {
		t.Fatalf("Forms(document) error = %v", err)
	}

	if got, want := forms.Length(), 2; got != want {
		t.Fatalf("Forms(document).Length() = %d, want %d", got, want)
	}

	firstID, ok := forms.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("Forms(document).Item(0) = (%d, %v), want first form", firstID, ok)
	}
	firstNode := store.Node(firstID)
	if firstNode == nil {
		t.Fatalf("Forms(document).Item(0) node = nil")
	}
	if got, want := firstNode.TagName, "form"; got != want {
		t.Fatalf("Forms(document).Item(0) tag = %q, want %q", got, want)
	}

	secondID, ok := forms.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("Forms(document).Item(1) = (%d, %v), want second form", secondID, ok)
	}
	secondNode := store.Node(secondID)
	if secondNode == nil {
		t.Fatalf("Forms(document).Item(1) node = nil")
	}
	if got, want := secondNode.TagName, "form"; got != want {
		t.Fatalf("Forms(document).Item(1) tag = %q, want %q", got, want)
	}

	if got, ok := forms.NamedItem("first"); !ok || got != firstID {
		t.Fatalf("Forms(document).NamedItem(first) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := forms.NamedItem("second"); !ok || got != secondID {
		t.Fatalf("Forms(document).NamedItem(second) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := forms.NamedItem("missing"); ok || got != 0 {
		t.Fatalf("Forms(document).NamedItem(missing) = (%d, %v), want (0, false)", got, ok)
	}

	ids := forms.IDs()
	if len(ids) != 2 {
		t.Fatalf("Forms(document).IDs() len = %d, want 2", len(ids))
	}
	ids[0] = 999
	if got, ok := forms.Item(0); !ok || got != firstID {
		t.Fatalf("Forms(document) mutated via IDs() = (%d, %v), want (%d, true)", got, ok, firstID)
	}

	formID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "form",
		Attrs: []Attribute{
			{Name: "name", Value: "third", HasValue: true},
		},
	})
	store.appendChild(store.DocumentID(), formID)

	if got, want := forms.Length(), 3; got != want {
		t.Fatalf("Forms(document).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := forms.NamedItem("third"); !ok || got != formID {
		t.Fatalf("Forms(document).NamedItem(third) = (%d, %v), want (%d, true)", got, ok, formID)
	}
}

func TestHTMLCollectionTracksFormElements(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<form id="profile"><input id="name"><select id="mode"><option>one</option></select><textarea id="bio"></textarea><button id="save"></button><fieldset id="group"><input id="city"></fieldset></form><div><input id="ignored"></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	formID := mustSelectSingle(t, store, "#profile")
	elements, err := store.FormElements(formID)
	if err != nil {
		t.Fatalf("FormElements(#profile) error = %v", err)
	}

	if got, want := elements.Length(), 6; got != want {
		t.Fatalf("FormElements(#profile).Length() = %d, want %d", got, want)
	}

	expectedTags := []string{"input", "select", "textarea", "button", "fieldset", "input"}
	for index, wantTag := range expectedTags {
		id, ok := elements.Item(index)
		if !ok || id == 0 {
			t.Fatalf("FormElements(#profile).Item(%d) = (%d, %v), want element", index, id, ok)
		}
		node := store.Node(id)
		if node == nil {
			t.Fatalf("FormElements(#profile).Item(%d) node = nil", index)
		}
		if got := node.TagName; got != wantTag {
			t.Fatalf("FormElements(#profile).Item(%d) tag = %q, want %q", index, got, wantTag)
		}
	}

	if got, ok := elements.NamedItem("name"); !ok || got == 0 {
		t.Fatalf("FormElements(#profile).NamedItem(name) = (%d, %v), want named input", got, ok)
	}
	if got, ok := elements.NamedItem("city"); !ok || got == 0 {
		t.Fatalf("FormElements(#profile).NamedItem(city) = (%d, %v), want nested fieldset control", got, ok)
	}
	if got, ok := elements.NamedItem("ignored"); ok || got != 0 {
		t.Fatalf("FormElements(#profile).NamedItem(ignored) = (%d, %v), want (0, false)", got, ok)
	}

	ids := elements.IDs()
	if len(ids) != 6 {
		t.Fatalf("FormElements(#profile).IDs() len = %d, want 6", len(ids))
	}
	ids[0] = 999
	if got, ok := elements.Item(0); !ok || got == 999 {
		t.Fatalf("FormElements(#profile) mutated via IDs() = (%d, %v), want stable first element", got, ok)
	}

	bonusID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "input",
		Attrs: []Attribute{
			{Name: "name", Value: "bonus", HasValue: true},
		},
	})
	store.appendChild(formID, bonusID)

	if got, want := elements.Length(), 7; got != want {
		t.Fatalf("FormElements(#profile).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := elements.NamedItem("bonus"); !ok || got != bonusID {
		t.Fatalf("FormElements(#profile).NamedItem(bonus) = (%d, %v), want (%d, true)", got, ok, bonusID)
	}
}

func TestHTMLCollectionTracksSelectSelectedOptions(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<select id="mode" multiple><option id="first" value="one" selected></option><option name="second" value="two"></option><optgroup><option id="third" value="three" selected></option></optgroup></select><div><option id="ignored" selected></option></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	selectID := mustSelectSingle(t, store, "#mode")
	selected, err := store.SelectedOptions(selectID)
	if err != nil {
		t.Fatalf("SelectedOptions(#mode) error = %v", err)
	}

	if got, want := selected.Length(), 2; got != want {
		t.Fatalf("SelectedOptions(#mode).Length() = %d, want %d", got, want)
	}

	firstID, ok := selected.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("SelectedOptions(#mode).Item(0) = (%d, %v), want first selected option", firstID, ok)
	}
	secondID, ok := selected.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("SelectedOptions(#mode).Item(1) = (%d, %v), want second selected option", secondID, ok)
	}

	if got, ok := selected.NamedItem("first"); !ok || got != firstID {
		t.Fatalf("SelectedOptions(#mode).NamedItem(first) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := selected.NamedItem("third"); !ok || got != secondID {
		t.Fatalf("SelectedOptions(#mode).NamedItem(third) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := selected.NamedItem("ignored"); ok || got != 0 {
		t.Fatalf("SelectedOptions(#mode).NamedItem(ignored) = (%d, %v), want (0, false)", got, ok)
	}

	ids := selected.IDs()
	if len(ids) != 2 {
		t.Fatalf("SelectedOptions(#mode).IDs() len = %d, want 2", len(ids))
	}
	ids[0] = 999
	if got, ok := selected.Item(0); !ok || got != firstID {
		t.Fatalf("SelectedOptions(#mode) mutated via IDs() = (%d, %v), want stable first selected option", got, ok)
	}

	if err := store.SetSelectValue(selectID, "two"); err != nil {
		t.Fatalf("SetSelectValue(#mode, two) error = %v", err)
	}

	if got, want := selected.Length(), 1; got != want {
		t.Fatalf("SelectedOptions(#mode).Length() after SetSelectValue = %d, want %d", got, want)
	}
	if got, ok := selected.NamedItem("second"); !ok || got == 0 {
		t.Fatalf("SelectedOptions(#mode).NamedItem(second) = (%d, %v), want selected option", got, ok)
	}

	bonusID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "option",
		Attrs: []Attribute{
			{Name: "name", Value: "bonus", HasValue: true},
			{Name: "selected", HasValue: false},
		},
	})
	store.appendChild(selectID, bonusID)

	if got, want := selected.Length(), 2; got != want {
		t.Fatalf("SelectedOptions(#mode).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := selected.NamedItem("bonus"); !ok || got != bonusID {
		t.Fatalf("SelectedOptions(#mode).NamedItem(bonus) = (%d, %v), want (%d, true)", got, ok, bonusID)
	}
}

func TestHTMLCollectionTracksSelectOptions(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<select id="mode"><option id="first" value="one"></option><optgroup><option name="second" value="two"></option></optgroup><option>three</option></select><div><option id="ignored"></option></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	selectID := mustSelectSingle(t, store, "#mode")
	options, err := store.Options(selectID)
	if err != nil {
		t.Fatalf("Options(#mode) error = %v", err)
	}

	if got, want := options.Length(), 3; got != want {
		t.Fatalf("Options(#mode).Length() = %d, want %d", got, want)
	}

	firstID, ok := options.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("Options(#mode).Item(0) = (%d, %v), want first option", firstID, ok)
	}
	firstNode := store.Node(firstID)
	if firstNode == nil {
		t.Fatalf("Options(#mode).Item(0) node = nil")
	}
	if got, want := firstNode.TagName, "option"; got != want {
		t.Fatalf("Options(#mode).Item(0) tag = %q, want %q", got, want)
	}

	secondID, ok := options.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("Options(#mode).Item(1) = (%d, %v), want second option", secondID, ok)
	}
	secondNode := store.Node(secondID)
	if secondNode == nil {
		t.Fatalf("Options(#mode).Item(1) node = nil")
	}
	if got, want := secondNode.TagName, "option"; got != want {
		t.Fatalf("Options(#mode).Item(1) tag = %q, want %q", got, want)
	}

	if got, ok := options.NamedItem("first"); !ok || got != firstID {
		t.Fatalf("Options(#mode).NamedItem(first) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := options.NamedItem("second"); !ok || got != secondID {
		t.Fatalf("Options(#mode).NamedItem(second) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := options.NamedItem("ignored"); ok || got != 0 {
		t.Fatalf("Options(#mode).NamedItem(ignored) = (%d, %v), want (0, false)", got, ok)
	}

	ids := options.IDs()
	if len(ids) != 3 {
		t.Fatalf("Options(#mode).IDs() len = %d, want 3", len(ids))
	}
	ids[0] = 999
	if got, ok := options.Item(0); !ok || got != firstID {
		t.Fatalf("Options(#mode) mutated via IDs() = (%d, %v), want (%d, true)", got, ok, firstID)
	}

	optionID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "option",
		Attrs: []Attribute{
			{Name: "name", Value: "third", HasValue: true},
		},
	})
	store.appendChild(selectID, optionID)

	if got, want := options.Length(), 4; got != want {
		t.Fatalf("Options(#mode).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := options.NamedItem("third"); !ok || got != optionID {
		t.Fatalf("Options(#mode).NamedItem(third) = (%d, %v), want (%d, true)", got, ok, optionID)
	}
}

func TestHTMLCollectionTracksDataListOptions(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<datalist id="choices"><option id="first" value="Cat"></option><span><option name="second" value="Dog"></option></span><div><option id="nested"><option id="inner" value="Bird"></option></option></div></datalist><div><option id="ignored"></option></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	datalistID := mustSelectSingle(t, store, "#choices")
	options, err := store.DatalistOptions(datalistID)
	if err != nil {
		t.Fatalf("DatalistOptions(#choices) error = %v", err)
	}

	if got, want := options.Length(), 4; got != want {
		t.Fatalf("DatalistOptions(#choices).Length() = %d, want %d", got, want)
	}

	firstID, ok := options.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("DatalistOptions(#choices).Item(0) = (%d, %v), want first option", firstID, ok)
	}
	secondID, ok := options.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("DatalistOptions(#choices).Item(1) = (%d, %v), want second option", secondID, ok)
	}
	nestedID, ok := options.Item(2)
	if !ok || nestedID == 0 {
		t.Fatalf("DatalistOptions(#choices).Item(2) = (%d, %v), want nested option", nestedID, ok)
	}
	innerID, ok := options.Item(3)
	if !ok || innerID == 0 {
		t.Fatalf("DatalistOptions(#choices).Item(3) = (%d, %v), want inner option", innerID, ok)
	}

	if got, ok := options.NamedItem("first"); !ok || got != firstID {
		t.Fatalf("DatalistOptions(#choices).NamedItem(first) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := options.NamedItem("second"); !ok || got != secondID {
		t.Fatalf("DatalistOptions(#choices).NamedItem(second) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := options.NamedItem("nested"); !ok || got != nestedID {
		t.Fatalf("DatalistOptions(#choices).NamedItem(nested) = (%d, %v), want (%d, true)", got, ok, nestedID)
	}
	if got, ok := options.NamedItem("inner"); !ok || got != innerID {
		t.Fatalf("DatalistOptions(#choices).NamedItem(inner) = (%d, %v), want (%d, true)", got, ok, innerID)
	}
	if got, ok := options.NamedItem("ignored"); ok || got != 0 {
		t.Fatalf("DatalistOptions(#choices).NamedItem(ignored) = (%d, %v), want (0, false)", got, ok)
	}

	ids := options.IDs()
	if len(ids) != 4 {
		t.Fatalf("DatalistOptions(#choices).IDs() len = %d, want 4", len(ids))
	}
	ids[0] = 999
	if got, ok := options.Item(0); !ok || got != firstID {
		t.Fatalf("DatalistOptions(#choices) mutated via IDs() = (%d, %v), want stable first option", got, ok)
	}

	bonusID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "option",
		Attrs: []Attribute{
			{Name: "name", Value: "bonus", HasValue: true},
		},
	})
	store.appendChild(datalistID, bonusID)

	if got, want := options.Length(), 5; got != want {
		t.Fatalf("DatalistOptions(#choices).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := options.NamedItem("bonus"); !ok || got != bonusID {
		t.Fatalf("DatalistOptions(#choices).NamedItem(bonus) = (%d, %v), want (%d, true)", got, ok, bonusID)
	}
}

func TestHTMLCollectionTracksFieldsetElements(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<fieldset id="group"><legend>Caption</legend><input id="name"><select id="mode"><option>one</option></select><fieldset id="nested"><input id="city"></fieldset></fieldset><div><input id="ignored"></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	fieldsetID := mustSelectSingle(t, store, "#group")
	elements, err := store.FieldsetElements(fieldsetID)
	if err != nil {
		t.Fatalf("FieldsetElements(#group) error = %v", err)
	}

	if got, want := elements.Length(), 4; got != want {
		t.Fatalf("FieldsetElements(#group).Length() = %d, want %d", got, want)
	}

	expectedTags := []string{"input", "select", "fieldset", "input"}
	for index, wantTag := range expectedTags {
		id, ok := elements.Item(index)
		if !ok || id == 0 {
			t.Fatalf("FieldsetElements(#group).Item(%d) = (%d, %v), want element", index, id, ok)
		}
		node := store.Node(id)
		if node == nil {
			t.Fatalf("FieldsetElements(#group).Item(%d) node = nil", index)
		}
		if got := node.TagName; got != wantTag {
			t.Fatalf("FieldsetElements(#group).Item(%d) tag = %q, want %q", index, got, wantTag)
		}
	}

	if got, ok := elements.NamedItem("name"); !ok || got == 0 {
		t.Fatalf("FieldsetElements(#group).NamedItem(name) = (%d, %v), want named input", got, ok)
	}
	if got, ok := elements.NamedItem("city"); !ok || got == 0 {
		t.Fatalf("FieldsetElements(#group).NamedItem(city) = (%d, %v), want nested fieldset control", got, ok)
	}
	if got, ok := elements.NamedItem("ignored"); ok || got != 0 {
		t.Fatalf("FieldsetElements(#group).NamedItem(ignored) = (%d, %v), want (0, false)", got, ok)
	}

	ids := elements.IDs()
	if len(ids) != 4 {
		t.Fatalf("FieldsetElements(#group).IDs() len = %d, want 4", len(ids))
	}
	ids[0] = 999
	if got, ok := elements.Item(0); !ok || got == 999 {
		t.Fatalf("FieldsetElements(#group) mutated via IDs() = (%d, %v), want stable first element", got, ok)
	}

	bonusID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "input",
		Attrs: []Attribute{
			{Name: "name", Value: "bonus", HasValue: true},
		},
	})
	store.appendChild(fieldsetID, bonusID)

	if got, want := elements.Length(), 5; got != want {
		t.Fatalf("FieldsetElements(#group).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := elements.NamedItem("bonus"); !ok || got != bonusID {
		t.Fatalf("FieldsetElements(#group).NamedItem(bonus) = (%d, %v), want (%d, true)", got, ok, bonusID)
	}
}

func TestHTMLCollectionTracksTableRows(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<table id="grid"><thead><tr id="head"></tr></thead><tbody><tr id="body1"><td><table><tr id="nested"></tr></table></td></tr><tr id="body2"></tr></tbody><tfoot><tr id="foot"></tr></tfoot></table><div><tr id="ignored"></tr></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tableID := mustSelectSingle(t, store, "#grid")
	rows, err := store.Rows(tableID)
	if err != nil {
		t.Fatalf("Rows(#grid) error = %v", err)
	}

	if got, want := rows.Length(), 4; got != want {
		t.Fatalf("Rows(#grid).Length() = %d, want %d", got, want)
	}

	expectedIDs := []string{"head", "body1", "body2", "foot"}
	for index, wantID := range expectedIDs {
		id, ok := rows.Item(index)
		if !ok || id == 0 {
			t.Fatalf("Rows(#grid).Item(%d) = (%d, %v), want row", index, id, ok)
		}
		node := store.Node(id)
		if node == nil {
			t.Fatalf("Rows(#grid).Item(%d) node = nil", index)
		}
		if got, want := node.TagName, "tr"; got != want {
			t.Fatalf("Rows(#grid).Item(%d) tag = %q, want %q", index, got, want)
		}
		if got, ok := rows.NamedItem(wantID); !ok || got != id {
			t.Fatalf("Rows(#grid).NamedItem(%s) = (%d, %v), want (%d, true)", wantID, got, ok, id)
		}
	}

	if got, ok := rows.NamedItem("nested"); ok || got != 0 {
		t.Fatalf("Rows(#grid).NamedItem(nested) = (%d, %v), want (0, false)", got, ok)
	}
	if got, ok := rows.NamedItem("ignored"); ok || got != 0 {
		t.Fatalf("Rows(#grid).NamedItem(ignored) = (%d, %v), want (0, false)", got, ok)
	}

	ids := rows.IDs()
	if len(ids) != 4 {
		t.Fatalf("Rows(#grid).IDs() len = %d, want 4", len(ids))
	}
	ids[0] = 999
	if got, ok := rows.Item(0); !ok || got == 999 {
		t.Fatalf("Rows(#grid) mutated via IDs() = (%d, %v), want stable first row", got, ok)
	}

	bonusID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "tr",
		Attrs: []Attribute{
			{Name: "name", Value: "bonus", HasValue: true},
		},
	})
	tbodyID := mustSelectSingle(t, store, "tbody")
	store.appendChild(tbodyID, bonusID)

	if got, want := rows.Length(), 5; got != want {
		t.Fatalf("Rows(#grid).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := rows.NamedItem("bonus"); !ok || got != bonusID {
		t.Fatalf("Rows(#grid).NamedItem(bonus) = (%d, %v), want (%d, true)", got, ok, bonusID)
	}
}

func TestHTMLCollectionTracksTableSectionRows(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<table id="grid"><thead id="head"><tr id="headrow"></tr></thead><tbody id="body"><tr id="body1"><td><table><tr id="nested"></tr></table></td></tr><tr id="body2"></tr></tbody><tfoot id="foot"><tr id="footrow"></tr></tfoot></table>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tbodyID := mustSelectSingle(t, store, "#body")
	rows, err := store.Rows(tbodyID)
	if err != nil {
		t.Fatalf("Rows(#body) error = %v", err)
	}

	if got, want := rows.Length(), 2; got != want {
		t.Fatalf("Rows(#body).Length() = %d, want %d", got, want)
	}

	firstID, ok := rows.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("Rows(#body).Item(0) = (%d, %v), want first row", firstID, ok)
	}
	secondID, ok := rows.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("Rows(#body).Item(1) = (%d, %v), want second row", secondID, ok)
	}
	if got, ok := rows.NamedItem("body1"); !ok || got != firstID {
		t.Fatalf("Rows(#body).NamedItem(body1) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := rows.NamedItem("body2"); !ok || got != secondID {
		t.Fatalf("Rows(#body).NamedItem(body2) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := rows.NamedItem("nested"); ok || got != 0 {
		t.Fatalf("Rows(#body).NamedItem(nested) = (%d, %v), want (0, false)", got, ok)
	}

	ids := rows.IDs()
	if len(ids) != 2 {
		t.Fatalf("Rows(#body).IDs() len = %d, want 2", len(ids))
	}
	ids[0] = 999
	if got, ok := rows.Item(0); !ok || got != firstID {
		t.Fatalf("Rows(#body) mutated via IDs() = (%d, %v), want stable first row", got, ok)
	}

	bonusID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "tr",
		Attrs: []Attribute{
			{Name: "name", Value: "bonus", HasValue: true},
		},
	})
	store.appendChild(tbodyID, bonusID)

	if got, want := rows.Length(), 3; got != want {
		t.Fatalf("Rows(#body).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := rows.NamedItem("bonus"); !ok || got != bonusID {
		t.Fatalf("Rows(#body).NamedItem(bonus) = (%d, %v), want (%d, true)", got, ok, bonusID)
	}
}

func TestHTMLCollectionTracksTableRowCells(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<table><tbody><tr id="row"><td id="first"></td><th name="second"></th><td><table><tr><td id="nested"></td></tr></table></td></tr></tbody></table>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	rowID := mustSelectSingle(t, store, "#row")
	cells, err := store.Cells(rowID)
	if err != nil {
		t.Fatalf("Cells(#row) error = %v", err)
	}

	if got, want := cells.Length(), 3; got != want {
		t.Fatalf("Cells(#row).Length() = %d, want %d", got, want)
	}

	firstID, ok := cells.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("Cells(#row).Item(0) = (%d, %v), want first cell", firstID, ok)
	}
	secondID, ok := cells.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("Cells(#row).Item(1) = (%d, %v), want second cell", secondID, ok)
	}
	if got, ok := cells.NamedItem("first"); !ok || got != firstID {
		t.Fatalf("Cells(#row).NamedItem(first) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := cells.NamedItem("second"); !ok || got != secondID {
		t.Fatalf("Cells(#row).NamedItem(second) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := cells.NamedItem("nested"); ok || got != 0 {
		t.Fatalf("Cells(#row).NamedItem(nested) = (%d, %v), want (0, false)", got, ok)
	}

	ids := cells.IDs()
	if len(ids) != 3 {
		t.Fatalf("Cells(#row).IDs() len = %d, want 3", len(ids))
	}
	ids[0] = 999
	if got, ok := cells.Item(0); !ok || got != firstID {
		t.Fatalf("Cells(#row) mutated via IDs() = (%d, %v), want stable first cell", got, ok)
	}

	bonusID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "td",
		Attrs: []Attribute{
			{Name: "name", Value: "bonus", HasValue: true},
		},
	})
	store.appendChild(rowID, bonusID)

	if got, want := cells.Length(), 4; got != want {
		t.Fatalf("Cells(#row).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := cells.NamedItem("bonus"); !ok || got != bonusID {
		t.Fatalf("Cells(#row).NamedItem(bonus) = (%d, %v), want (%d, true)", got, ok, bonusID)
	}
}

func TestHTMLCollectionTracksTableBodies(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<table id="grid"><caption>title</caption><tbody id="body1"><tr></tr></tbody><thead><tr></tr></thead><tbody id="body2"><tr></tr></tbody><tfoot><tr></tr></tfoot><tbody id="body3"><tr></tr></tbody><div><tbody id="ignored"></tbody></div></table>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tableID := mustSelectSingle(t, store, "#grid")
	bodies, err := store.TBodies(tableID)
	if err != nil {
		t.Fatalf("TBodies(#grid) error = %v", err)
	}

	if got, want := bodies.Length(), 3; got != want {
		t.Fatalf("TBodies(#grid).Length() = %d, want %d", got, want)
	}

	expectedIDs := []string{"body1", "body2", "body3"}
	for index, wantID := range expectedIDs {
		id, ok := bodies.Item(index)
		if !ok || id == 0 {
			t.Fatalf("TBodies(#grid).Item(%d) = (%d, %v), want tbody", index, id, ok)
		}
		node := store.Node(id)
		if node == nil {
			t.Fatalf("TBodies(#grid).Item(%d) node = nil", index)
		}
		if got, want := node.TagName, "tbody"; got != want {
			t.Fatalf("TBodies(#grid).Item(%d) tag = %q, want %q", index, got, want)
		}
		if got, ok := bodies.NamedItem(wantID); !ok || got != id {
			t.Fatalf("TBodies(#grid).NamedItem(%s) = (%d, %v), want (%d, true)", wantID, got, ok, id)
		}
	}

	if got, ok := bodies.NamedItem("ignored"); ok || got != 0 {
		t.Fatalf("TBodies(#grid).NamedItem(ignored) = (%d, %v), want (0, false)", got, ok)
	}

	ids := bodies.IDs()
	if len(ids) != 3 {
		t.Fatalf("TBodies(#grid).IDs() len = %d, want 3", len(ids))
	}
	ids[0] = 999
	if got, ok := bodies.Item(0); !ok || got == 999 {
		t.Fatalf("TBodies(#grid) mutated via IDs() = (%d, %v), want stable first tbody", got, ok)
	}

	bonusID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "tbody",
		Attrs: []Attribute{
			{Name: "name", Value: "bonus", HasValue: true},
		},
	})
	store.appendChild(tableID, bonusID)

	if got, want := bodies.Length(), 4; got != want {
		t.Fatalf("TBodies(#grid).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := bodies.NamedItem("bonus"); !ok || got != bonusID {
		t.Fatalf("TBodies(#grid).NamedItem(bonus) = (%d, %v), want (%d, true)", got, ok, bonusID)
	}
}

func TestHTMLCollectionTracksDocumentLinks(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<a id="first" href="/a">A</a><div><area name="second" href="/b"></div><a name="ignored">Ignored</a>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	links, err := store.Links()
	if err != nil {
		t.Fatalf("Links(document) error = %v", err)
	}

	if got, want := links.Length(), 2; got != want {
		t.Fatalf("Links(document).Length() = %d, want %d", got, want)
	}

	firstID, ok := links.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("Links(document).Item(0) = (%d, %v), want first link", firstID, ok)
	}
	firstNode := store.Node(firstID)
	if firstNode == nil {
		t.Fatalf("Links(document).Item(0) node = nil")
	}
	if got, want := firstNode.TagName, "a"; got != want {
		t.Fatalf("Links(document).Item(0) tag = %q, want %q", got, want)
	}

	secondID, ok := links.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("Links(document).Item(1) = (%d, %v), want second link", secondID, ok)
	}
	secondNode := store.Node(secondID)
	if secondNode == nil {
		t.Fatalf("Links(document).Item(1) node = nil")
	}
	if got, want := secondNode.TagName, "area"; got != want {
		t.Fatalf("Links(document).Item(1) tag = %q, want %q", got, want)
	}

	if got, ok := links.NamedItem("first"); !ok || got != firstID {
		t.Fatalf("Links(document).NamedItem(first) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := links.NamedItem("second"); !ok || got != secondID {
		t.Fatalf("Links(document).NamedItem(second) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := links.NamedItem("ignored"); ok || got != 0 {
		t.Fatalf("Links(document).NamedItem(ignored) = (%d, %v), want (0, false)", got, ok)
	}

	linksIDs := links.IDs()
	if len(linksIDs) != 2 {
		t.Fatalf("Links(document).IDs() len = %d, want 2", len(linksIDs))
	}
	linksIDs[0] = 999
	if got, ok := links.Item(0); !ok || got != firstID {
		t.Fatalf("Links(document) mutated via IDs() = (%d, %v), want (%d, true)", got, ok, firstID)
	}

	linkID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "a",
		Attrs: []Attribute{
			{Name: "name", Value: "third", HasValue: true},
			{Name: "href", Value: "/c", HasValue: true},
		},
	})
	store.appendChild(store.DocumentID(), linkID)

	if got, want := links.Length(), 3; got != want {
		t.Fatalf("Links(document).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := links.NamedItem("third"); !ok || got != linkID {
		t.Fatalf("Links(document).NamedItem(third) = (%d, %v), want (%d, true)", got, ok, linkID)
	}
}

func TestHTMLCollectionTracksDocumentAnchors(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<a id="first" name="alpha"></a><div><a name="beta"></a></div><a href="/ignored">Ignored</a>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	anchors, err := store.Anchors()
	if err != nil {
		t.Fatalf("Anchors(document) error = %v", err)
	}

	if got, want := anchors.Length(), 2; got != want {
		t.Fatalf("Anchors(document).Length() = %d, want %d", got, want)
	}

	firstID, ok := anchors.Item(0)
	if !ok || firstID == 0 {
		t.Fatalf("Anchors(document).Item(0) = (%d, %v), want first anchor", firstID, ok)
	}
	secondID, ok := anchors.Item(1)
	if !ok || secondID == 0 {
		t.Fatalf("Anchors(document).Item(1) = (%d, %v), want second anchor", secondID, ok)
	}

	if got, ok := anchors.NamedItem("alpha"); !ok || got != firstID {
		t.Fatalf("Anchors(document).NamedItem(alpha) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := anchors.NamedItem("beta"); !ok || got != secondID {
		t.Fatalf("Anchors(document).NamedItem(beta) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := anchors.NamedItem("missing"); ok || got != 0 {
		t.Fatalf("Anchors(document).NamedItem(missing) = (%d, %v), want (0, false)", got, ok)
	}

	anchorsIDs := anchors.IDs()
	if len(anchorsIDs) != 2 {
		t.Fatalf("Anchors(document).IDs() len = %d, want 2", len(anchorsIDs))
	}
	anchorsIDs[0] = 999
	if got, ok := anchors.Item(0); !ok || got != firstID {
		t.Fatalf("Anchors(document) mutated via IDs() = (%d, %v), want (%d, true)", got, ok, firstID)
	}

	anchorID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "a",
		Attrs: []Attribute{
			{Name: "name", Value: "gamma", HasValue: true},
		},
	})
	store.appendChild(store.DocumentID(), anchorID)

	if got, want := anchors.Length(), 3; got != want {
		t.Fatalf("Anchors(document).Length() after mutation = %d, want %d", got, want)
	}
	if got, ok := anchors.NamedItem("gamma"); !ok || got != anchorID {
		t.Fatalf("Anchors(document).NamedItem(gamma) = (%d, %v), want (%d, true)", got, ok, anchorID)
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
	if _, err := nilStore.Scripts(); err == nil {
		t.Fatalf("nil Scripts() error = nil, want dom store error")
	}
	if _, err := nilStore.Images(); err == nil {
		t.Fatalf("nil Images() error = nil, want dom store error")
	}
	if _, err := nilStore.Forms(); err == nil {
		t.Fatalf("nil Forms() error = nil, want dom store error")
	}
	if _, err := nilStore.FormElements(1); err == nil {
		t.Fatalf("nil FormElements() error = nil, want dom store error")
	}
	if _, err := nilStore.SelectedOptions(1); err == nil {
		t.Fatalf("nil SelectedOptions() error = nil, want dom store error")
	}
	if _, err := nilStore.Options(1); err == nil {
		t.Fatalf("nil Options() error = nil, want dom store error")
	}
	if _, err := nilStore.DatalistOptions(1); err == nil {
		t.Fatalf("nil DatalistOptions() error = nil, want dom store error")
	}
	if _, err := nilStore.FieldsetElements(1); err == nil {
		t.Fatalf("nil FieldsetElements() error = nil, want dom store error")
	}
	if _, err := nilStore.Cells(1); err == nil {
		t.Fatalf("nil Cells() error = nil, want dom store error")
	}
	if _, err := nilStore.Rows(1); err == nil {
		t.Fatalf("nil Rows() error = nil, want dom store error")
	}
	if _, err := nilStore.TBodies(1); err == nil {
		t.Fatalf("nil TBodies() error = nil, want dom store error")
	}
	if _, err := nilStore.Links(); err == nil {
		t.Fatalf("nil Links() error = nil, want dom store error")
	}
	if _, err := nilStore.Anchors(); err == nil {
		t.Fatalf("nil Anchors() error = nil, want dom store error")
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

	if _, err := store.ChildNodes(textID); err == nil {
		t.Fatalf("ChildNodes(text node) error = nil, want unsupported node error")
	}
	if _, err := store.TemplateContentChildNodes(textID); err == nil {
		t.Fatalf("TemplateContentChildNodes(text node) error = nil, want unsupported node error")
	}

	if _, err := store.Scripts(); err != nil {
		t.Fatalf("Scripts(document) error = %v, want nil", err)
	}
}
