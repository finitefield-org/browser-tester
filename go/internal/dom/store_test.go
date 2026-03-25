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

func TestSerializationEscapesSpecialCharacters(t *testing.T) {
	store := NewStore()

	rootID := store.newNode(Node{
		Kind:    NodeKindElement,
		TagName: "div",
		Attrs: []Attribute{
			{Name: "data-x", Value: `a&b<"c">`, HasValue: true},
		},
	})
	textID := store.newNode(Node{
		Kind: NodeKindText,
		Text: `1 < 2 & 3 > 0`,
	})

	store.appendChild(store.DocumentID(), rootID)
	store.appendChild(rootID, textID)

	if got, want := store.DumpDOM(), `<div data-x="a&amp;b&lt;&quot;c&quot;&gt;">1 &lt; 2 &amp; 3 &gt; 0</div>`; got != want {
		t.Fatalf("DumpDOM() = %q, want %q", got, want)
	}

	outer, err := store.OuterHTMLForNode(rootID)
	if err != nil {
		t.Fatalf("OuterHTMLForNode() error = %v", err)
	}
	if got, want := outer, `<div data-x="a&amp;b&lt;&quot;c&quot;&gt;">1 &lt; 2 &amp; 3 &gt; 0</div>`; got != want {
		t.Fatalf("OuterHTMLForNode() = %q, want %q", got, want)
	}
}

func TestFormControlMutationHelpers(t *testing.T) {
	store := NewStore()
	input := `<form id="profile"><input id="name"><input id="flag" type="checkbox"><input id="radio-a" type="radio" name="size" checked><input id="radio-b" type="radio" name="size"><textarea id="bio">Base</textarea><select id="mode"><option value="a" selected>A</option><option>B</option><option value="c">C</option></select></form>`
	if err := store.BootstrapHTML(input); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	nameID := mustSelectSingle(t, store, "#name")
	flagID := mustSelectSingle(t, store, "#flag")
	radioBID := mustSelectSingle(t, store, "#radio-b")
	bioID := mustSelectSingle(t, store, "#bio")
	modeID := mustSelectSingle(t, store, "#mode")

	if err := store.SetFormControlValue(nameID, "Ada"); err != nil {
		t.Fatalf("SetFormControlValue(#name) error = %v", err)
	}
	if err := store.SetFormControlValue(bioID, "Line 1\nLine 2"); err != nil {
		t.Fatalf("SetFormControlValue(#bio) error = %v", err)
	}
	if err := store.SetFormControlChecked(flagID, true); err != nil {
		t.Fatalf("SetFormControlChecked(#flag) error = %v", err)
	}
	if err := store.SetFormControlChecked(radioBID, true); err != nil {
		t.Fatalf("SetFormControlChecked(#radio-b) error = %v", err)
	}
	if err := store.SetSelectValue(modeID, "B"); err != nil {
		t.Fatalf("SetSelectValue(#mode) error = %v", err)
	}

	if got, want := store.DumpDOM(), `<form id="profile"><input id="name" value="Ada"><input id="flag" type="checkbox" checked><input id="radio-a" type="radio" name="size"><input id="radio-b" type="radio" name="size" checked><textarea id="bio">Line 1
Line 2</textarea><select id="mode"><option value="a">A</option><option selected>B</option><option value="c">C</option></select></form>`; got != want {
		t.Fatalf("DumpDOM() = %q, want %q", got, want)
	}
}

func TestSetSelectValueClearsUnmatchedSelection(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<select id="mode"><option value="a" selected>A</option><option value="b">B</option></select>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	modeID := mustSelectSingle(t, store, "#mode")
	if err := store.SetSelectValue(modeID, "missing"); err != nil {
		t.Fatalf("SetSelectValue(#mode, missing) error = %v", err)
	}

	if got, want := store.DumpDOM(), `<select id="mode"><option value="a">A</option><option value="b">B</option></select>`; got != want {
		t.Fatalf("DumpDOM() after missing select value = %q, want %q", got, want)
	}
}

func TestFormControlMutationHelpersRejectUnsupportedNodes(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><input id="name"><input id="flag" type="checkbox"><select id="mode"><option>A</option></select></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	nameID := mustSelectSingle(t, store, "#name")
	flagID := mustSelectSingle(t, store, "#flag")

	if err := store.SetFormControlValue(flagID, "Ada"); err == nil {
		t.Fatalf("SetFormControlValue(#flag) error = nil, want unsupported control error")
	}
	if err := store.SetFormControlChecked(nameID, true); err == nil {
		t.Fatalf("SetFormControlChecked(#name) error = nil, want unsupported control error")
	}
	if err := store.SetSelectValue(nameID, "A"); err == nil {
		t.Fatalf("SetSelectValue(#name) error = nil, want unsupported control error")
	}
}

func TestResetFormControlsRestoresInitialState(t *testing.T) {
	store := NewStore()
	input := `<form id="profile"><input id="name"><input id="flag" type="checkbox"><input id="radio-a" type="radio" name="size" checked><input id="radio-b" type="radio" name="size"><textarea id="bio">Base</textarea><select id="mode"><option value="a" selected>A</option><option>B</option><option value="c">C</option></select></form>`
	if err := store.BootstrapHTML(input); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	formID := mustSelectSingle(t, store, "#profile")
	nameID := mustSelectSingle(t, store, "#name")
	flagID := mustSelectSingle(t, store, "#flag")
	radioBID := mustSelectSingle(t, store, "#radio-b")
	bioID := mustSelectSingle(t, store, "#bio")
	modeID := mustSelectSingle(t, store, "#mode")

	if err := store.SetFormControlValue(nameID, "Ada"); err != nil {
		t.Fatalf("SetFormControlValue(#name) error = %v", err)
	}
	if err := store.SetFormControlChecked(flagID, true); err != nil {
		t.Fatalf("SetFormControlChecked(#flag) error = %v", err)
	}
	if err := store.SetFormControlChecked(radioBID, true); err != nil {
		t.Fatalf("SetFormControlChecked(#radio-b) error = %v", err)
	}
	if err := store.SetFormControlValue(bioID, "Line 1\nLine 2"); err != nil {
		t.Fatalf("SetFormControlValue(#bio) error = %v", err)
	}
	if err := store.SetSelectValue(modeID, "B"); err != nil {
		t.Fatalf("SetSelectValue(#mode) error = %v", err)
	}
	if err := store.SetUserValidity(nameID, true); err != nil {
		t.Fatalf("SetUserValidity(#name) error = %v", err)
	}
	if err := store.SetUserValidity(flagID, true); err != nil {
		t.Fatalf("SetUserValidity(#flag) error = %v", err)
	}
	if err := store.SetUserValidity(radioBID, true); err != nil {
		t.Fatalf("SetUserValidity(#radio-b) error = %v", err)
	}
	if err := store.SetUserValidity(bioID, true); err != nil {
		t.Fatalf("SetUserValidity(#bio) error = %v", err)
	}
	if err := store.SetUserValidity(modeID, true); err != nil {
		t.Fatalf("SetUserValidity(#mode) error = %v", err)
	}

	if err := store.ResetFormControls(formID); err != nil {
		t.Fatalf("ResetFormControls(#profile) error = %v", err)
	}

	for _, nodeID := range []NodeID{nameID, flagID, radioBID, bioID, modeID} {
		if node := store.Node(nodeID); node == nil || node.UserValidity {
			t.Fatalf("node(%d).UserValidity after reset = %v, want false", nodeID, node)
		}
	}

	if got, want := store.DumpDOM(), input; got != want {
		t.Fatalf("DumpDOM() after ResetFormControls = %q, want %q", got, want)
	}
}

func TestSetTextContentUpdatesTextareaDefaultValue(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<form id="profile"><textarea id="bio">Base</textarea></form>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	formID := mustSelectSingle(t, store, "#profile")
	bioID := mustSelectSingle(t, store, "#bio")

	if err := store.SetTextContent(bioID, "Draft"); err != nil {
		t.Fatalf("SetTextContent(#bio) error = %v", err)
	}
	if got, want := store.DumpDOM(), `<form id="profile"><textarea id="bio">Draft</textarea></form>`; got != want {
		t.Fatalf("DumpDOM() after SetTextContent = %q, want %q", got, want)
	}

	if err := store.ResetFormControls(formID); err != nil {
		t.Fatalf("ResetFormControls(#profile) after SetTextContent error = %v", err)
	}
	if got, want := store.DumpDOM(), `<form id="profile"><textarea id="bio">Draft</textarea></form>`; got != want {
		t.Fatalf("DumpDOM() after reset of SetTextContent textarea = %q, want %q", got, want)
	}

	if err := store.SetFormControlValue(bioID, "User"); err != nil {
		t.Fatalf("SetFormControlValue(#bio) error = %v", err)
	}
	if got, want := store.DumpDOM(), `<form id="profile"><textarea id="bio">User</textarea></form>`; got != want {
		t.Fatalf("DumpDOM() after SetFormControlValue = %q, want %q", got, want)
	}

	if err := store.ResetFormControls(formID); err != nil {
		t.Fatalf("ResetFormControls(#profile) after SetFormControlValue error = %v", err)
	}
	if got, want := store.DumpDOM(), `<form id="profile"><textarea id="bio">Draft</textarea></form>`; got != want {
		t.Fatalf("DumpDOM() after reset of SetFormControlValue textarea = %q, want %q", got, want)
	}
}

func TestTextareaChildMutationsUpdateResetDefaultValue(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<form id="profile"><textarea id="bio">Base</textarea><button id="reset" type="reset">Reset</button></form>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	formID := mustSelectSingle(t, store, "#profile")
	bioID := mustSelectSingle(t, store, "#bio")

	if err := store.ReplaceChildren(bioID, "Draft"); err != nil {
		t.Fatalf("ReplaceChildren(#bio) error = %v", err)
	}
	if err := store.ResetFormControls(formID); err != nil {
		t.Fatalf("ResetFormControls(#profile) after ReplaceChildren error = %v", err)
	}
	if got, want := store.DumpDOM(), `<form id="profile"><textarea id="bio">Draft</textarea><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("DumpDOM() after reset of ReplaceChildren textarea = %q, want %q", got, want)
	}

	if err := store.SetInnerHTML(bioID, "Fresh"); err != nil {
		t.Fatalf("SetInnerHTML(#bio) second update error = %v", err)
	}
	if err := store.InsertAdjacentHTML(bioID, "beforeend", `<span id="bang">!</span>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(#bio,beforeend) error = %v", err)
	}
	if err := store.ResetFormControls(formID); err != nil {
		t.Fatalf("ResetFormControls(#profile) after InsertAdjacentHTML error = %v", err)
	}
	if got, want := store.DumpDOM(), `<form id="profile"><textarea id="bio">Fresh!</textarea><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("DumpDOM() after reset of InsertAdjacentHTML textarea = %q, want %q", got, want)
	}

	if err := store.SetInnerHTML(bioID, "Fresh"); err != nil {
		t.Fatalf("SetInnerHTML(#bio) third update error = %v", err)
	}
	if err := store.InsertAdjacentHTML(bioID, "beforeend", `<span id="bang">!</span>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(#bio,beforeend) second error = %v", err)
	}
	bangID := mustSelectSingle(t, store, "#bang")
	if err := store.RemoveNode(bangID); err != nil {
		t.Fatalf("RemoveNode(#bang) error = %v", err)
	}
	if got, want := store.DumpDOM(), `<form id="profile"><textarea id="bio">Fresh</textarea><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("DumpDOM() after RemoveNode(#bang) = %q, want %q", got, want)
	}
	if err := store.ResetFormControls(formID); err != nil {
		t.Fatalf("ResetFormControls(#profile) after RemoveNode error = %v", err)
	}
	if got, want := store.DumpDOM(), `<form id="profile"><textarea id="bio">Fresh</textarea><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("DumpDOM() after reset of RemoveNode textarea = %q, want %q", got, want)
	}
}

func mustSelectSingle(t *testing.T, store *Store, selector string) NodeID {
	t.Helper()
	nodes, err := store.Select(selector)
	if err != nil {
		t.Fatalf("Select(%q) error = %v", selector, err)
	}
	if len(nodes) != 1 {
		t.Fatalf("Select(%q) len = %d, want 1", selector, len(nodes))
	}
	return nodes[0]
}
