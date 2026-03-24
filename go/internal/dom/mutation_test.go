package dom

import "testing"

func TestInnerHTMLForNodeAndSetInnerHTML(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<section id="wrap"><div id="target"><p>Hello</p><span>world</span></div></section>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	targetID := mustSelectSingle(t, store, "#target")
	inner, err := store.InnerHTMLForNode(targetID)
	if err != nil {
		t.Fatalf("InnerHTMLForNode(#target) error = %v", err)
	}
	if got, want := inner, `<p>Hello</p><span>world</span>`; got != want {
		t.Fatalf("InnerHTMLForNode(#target) = %q, want %q", got, want)
	}

	if err := store.SetInnerHTML(targetID, `<em id="next">updated</em>tail`); err != nil {
		t.Fatalf("SetInnerHTML(#target) error = %v", err)
	}
	if got, want := store.DumpDOM(), `<section id="wrap"><div id="target"><em id="next">updated</em>tail</div></section>`; got != want {
		t.Fatalf("DumpDOM() after SetInnerHTML = %q, want %q", got, want)
	}
	children, err := store.Children(targetID)
	if err != nil {
		t.Fatalf("Children(#target) after SetInnerHTML error = %v", err)
	}
	if got, want := children.Length(), 1; got != want {
		t.Fatalf("Children(#target).Length() after SetInnerHTML = %d, want %d", got, want)
	}
	if got, ok := children.NamedItem("next"); !ok {
		t.Fatalf("Children(#target).NamedItem(next) = (%d, %v), want inserted child", got, ok)
	}
	if ids, err := store.Select("p"); err != nil || len(ids) != 0 {
		t.Fatalf("Select(p) after SetInnerHTML = (%v, %v), want no matches", ids, err)
	}
}

func TestSetOuterHTMLReplacesNodeAndPreservesSiblings(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<section id="wrap"><div id="target"><b>x</b></div><p id="tail">tail</p></section>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	targetID := mustSelectSingle(t, store, "#target")
	if err := store.SetOuterHTML(targetID, `<article id="next">n</article><aside id="extra"></aside>`); err != nil {
		t.Fatalf("SetOuterHTML(#target) error = %v", err)
	}

	if got, want := store.DumpDOM(), `<section id="wrap"><article id="next">n</article><aside id="extra"></aside><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("DumpDOM() after SetOuterHTML = %q, want %q", got, want)
	}
	if ids, err := store.Select("#target"); err != nil || len(ids) != 0 {
		t.Fatalf("Select(#target) after replacement = (%v, %v), want no matches", ids, err)
	}
}

func TestInsertAdjacentHTMLPositions(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<section id="wrap"><div id="target"><span id="inside">x</span></div><p id="tail">t</p></section>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	targetID := mustSelectSingle(t, store, "#target")
	if err := store.InsertAdjacentHTML(targetID, "beforebegin", `<a id="bb"></a>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(beforebegin) error = %v", err)
	}
	if err := store.InsertAdjacentHTML(targetID, "afterbegin", `<i id="ab">a</i>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(afterbegin) error = %v", err)
	}
	if err := store.InsertAdjacentHTML(targetID, "beforeend", `<i id="be">b</i>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(beforeend) error = %v", err)
	}
	if err := store.InsertAdjacentHTML(targetID, "afterend", `<a id="ae"></a>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(afterend) error = %v", err)
	}

	if got, want := store.DumpDOM(), `<section id="wrap"><a id="bb"></a><div id="target"><i id="ab">a</i><span id="inside">x</span><i id="be">b</i></div><a id="ae"></a><p id="tail">t</p></section>`; got != want {
		t.Fatalf("DumpDOM() after InsertAdjacentHTML = %q, want %q", got, want)
	}
	wrapID := mustSelectSingle(t, store, "#wrap")
	children, err := store.Children(wrapID)
	if err != nil {
		t.Fatalf("Children(#wrap) after InsertAdjacentHTML error = %v", err)
	}
	if got, want := children.Length(), 4; got != want {
		t.Fatalf("Children(#wrap).Length() after InsertAdjacentHTML = %d, want %d", got, want)
	}
	wantIDs := []string{"bb", "target", "ae", "tail"}
	for i, wantID := range wantIDs {
		id, ok := children.Item(i)
		if !ok {
			t.Fatalf("Children(#wrap).Item(%d) = (0, false), want %q", i, wantID)
		}
		node := store.Node(id)
		if node == nil {
			t.Fatalf("Children(#wrap).Item(%d) node = nil", i)
		}
		gotID, ok := attributeValue(node.Attrs, "id")
		if !ok || gotID != wantID {
			t.Fatalf("Children(#wrap).Item(%d) id = (%q, %v), want %q", i, gotID, ok, wantID)
		}
	}
}

func TestRemoveNodeRemovesSubtree(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<section id="wrap"><div id="remove"><span id="child">x</span></div><p id="keep">k</p></section>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	removeID := mustSelectSingle(t, store, "#remove")
	if err := store.RemoveNode(removeID); err != nil {
		t.Fatalf("RemoveNode(#remove) error = %v", err)
	}

	if got, want := store.DumpDOM(), `<section id="wrap"><p id="keep">k</p></section>`; got != want {
		t.Fatalf("DumpDOM() after RemoveNode = %q, want %q", got, want)
	}
	if ids, err := store.Select("#child"); err != nil || len(ids) != 0 {
		t.Fatalf("Select(#child) after RemoveNode = (%v, %v), want no matches", ids, err)
	}
}

func TestMutationHelpersUpdateFocusedNodeState(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<section id="wrap"><div id="target"><span id="child">x</span></div><p id="keep">k</p></section>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	targetID := mustSelectSingle(t, store, "#target")
	childID := mustSelectSingle(t, store, "#child")
	keepID := mustSelectSingle(t, store, "#keep")

	if err := store.SetFocusedNode(childID); err != nil {
		t.Fatalf("SetFocusedNode(#child) error = %v", err)
	}
	if err := store.SetInnerHTML(targetID, `<em id="next">updated</em>`); err != nil {
		t.Fatalf("SetInnerHTML(#target) error = %v", err)
	}
	if got := store.FocusedNodeID(); got != 0 {
		t.Fatalf("FocusedNodeID() after removing focused descendant = %d, want 0", got)
	}

	if err := store.SetFocusedNode(targetID); err != nil {
		t.Fatalf("SetFocusedNode(#target) error = %v", err)
	}
	if err := store.SetInnerHTML(targetID, `<em id="next">updated</em>`); err != nil {
		t.Fatalf("SetInnerHTML(#target) preserve focus error = %v", err)
	}
	if got := store.FocusedNodeID(); got != targetID {
		t.Fatalf("FocusedNodeID() after SetInnerHTML on focused node = %d, want %d", got, targetID)
	}

	if err := store.SetFocusedNode(targetID); err != nil {
		t.Fatalf("SetFocusedNode(#target) before SetOuterHTML error = %v", err)
	}
	if err := store.SetOuterHTML(targetID, `<article id="next">n</article>`); err != nil {
		t.Fatalf("SetOuterHTML(#target) error = %v", err)
	}
	if got := store.FocusedNodeID(); got != 0 {
		t.Fatalf("FocusedNodeID() after SetOuterHTML = %d, want 0", got)
	}

	if err := store.SetFocusedNode(keepID); err != nil {
		t.Fatalf("SetFocusedNode(#keep) error = %v", err)
	}
	if err := store.RemoveNode(keepID); err != nil {
		t.Fatalf("RemoveNode(#keep) error = %v", err)
	}
	if got := store.FocusedNodeID(); got != 0 {
		t.Fatalf("FocusedNodeID() after RemoveNode = %d, want 0", got)
	}
}

func TestMutationHelpersUpdateTargetNodeState(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<section id="wrap"><div id="target"><span id="child">x</span></div><p id="keep">k</p></section>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	targetID := mustSelectSingle(t, store, "#target")
	childID := mustSelectSingle(t, store, "#child")
	keepID := mustSelectSingle(t, store, "#keep")

	store.SyncTargetFromURL("https://example.test/page#child")
	if got := store.TargetNodeID(); got != childID {
		t.Fatalf("TargetNodeID() after #child = %d, want %d", got, childID)
	}
	if err := store.SetInnerHTML(targetID, `<em id="next">updated</em>`); err != nil {
		t.Fatalf("SetInnerHTML(#target) error = %v", err)
	}
	if got := store.TargetNodeID(); got != 0 {
		t.Fatalf("TargetNodeID() after removing targeted descendant = %d, want 0", got)
	}

	store.SyncTargetFromURL("https://example.test/page#target")
	if got := store.TargetNodeID(); got != targetID {
		t.Fatalf("TargetNodeID() after #target = %d, want %d", got, targetID)
	}
	if err := store.SetOuterHTML(targetID, `<article id="next">n</article>`); err != nil {
		t.Fatalf("SetOuterHTML(#target) error = %v", err)
	}
	if got := store.TargetNodeID(); got != 0 {
		t.Fatalf("TargetNodeID() after SetOuterHTML = %d, want 0", got)
	}

	store.SyncTargetFromURL("https://example.test/page#keep")
	if got := store.TargetNodeID(); got != keepID {
		t.Fatalf("TargetNodeID() after #keep = %d, want %d", got, keepID)
	}
	if err := store.RemoveNode(keepID); err != nil {
		t.Fatalf("RemoveNode(#keep) error = %v", err)
	}
	if got := store.TargetNodeID(); got != 0 {
		t.Fatalf("TargetNodeID() after RemoveNode = %d, want 0", got)
	}
}

func TestCloneNodeDeepAndShallow(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<div id="root"><p id="p" class="copy"><span>text</span></p></div>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	rootID := mustSelectSingle(t, store, "#root")
	pID := mustSelectSingle(t, store, "#p")

	deepCloneID, err := store.CloneNode(pID, true)
	if err != nil {
		t.Fatalf("CloneNode(deep) error = %v", err)
	}
	if deepCloneID == pID {
		t.Fatalf("CloneNode(deep) returned source node id")
	}
	store.appendChild(rootID, deepCloneID)

	shallowCloneID, err := store.CloneNode(pID, false)
	if err != nil {
		t.Fatalf("CloneNode(shallow) error = %v", err)
	}
	store.appendChild(rootID, shallowCloneID)

	deepOuter, err := store.OuterHTMLForNode(deepCloneID)
	if err != nil {
		t.Fatalf("OuterHTMLForNode(deepCloneID) error = %v", err)
	}
	if got, want := deepOuter, `<p id="p" class="copy"><span>text</span></p>`; got != want {
		t.Fatalf("OuterHTMLForNode(deepCloneID) = %q, want %q", got, want)
	}

	shallowOuter, err := store.OuterHTMLForNode(shallowCloneID)
	if err != nil {
		t.Fatalf("OuterHTMLForNode(shallowCloneID) error = %v", err)
	}
	if got, want := shallowOuter, `<p id="p" class="copy"></p>`; got != want {
		t.Fatalf("OuterHTMLForNode(shallowCloneID) = %q, want %q", got, want)
	}
}

func TestCloneNodePreservesUserValidity(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<form id="profile"><input id="name" type="text" required value="Ada"></form>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	nameID := mustSelectSingle(t, store, "#name")
	if err := store.SetUserValidity(nameID, true); err != nil {
		t.Fatalf("SetUserValidity(#name) error = %v", err)
	}

	deepCloneID, err := store.CloneNode(nameID, true)
	if err != nil {
		t.Fatalf("CloneNode(deep) error = %v", err)
	}
	if node := store.Node(deepCloneID); node == nil || !node.UserValidity {
		t.Fatalf("CloneNode(deep) UserValidity = %v, want true", node)
	}

	shallowCloneID, err := store.CloneNode(nameID, false)
	if err != nil {
		t.Fatalf("CloneNode(shallow) error = %v", err)
	}
	if node := store.Node(shallowCloneID); node == nil || !node.UserValidity {
		t.Fatalf("CloneNode(shallow) UserValidity = %v, want true", node)
	}
}

func TestMutationHelpersRejectInvalidInputs(t *testing.T) {
	var nilStore *Store
	if _, err := nilStore.InnerHTMLForNode(1); err == nil {
		t.Fatalf("nil InnerHTMLForNode() error = nil, want dom store error")
	}
	if err := nilStore.SetInnerHTML(1, "<p>x</p>"); err == nil {
		t.Fatalf("nil SetInnerHTML() error = nil, want dom store error")
	}
	if err := nilStore.SetOuterHTML(1, "<p>x</p>"); err == nil {
		t.Fatalf("nil SetOuterHTML() error = nil, want dom store error")
	}
	if err := nilStore.InsertAdjacentHTML(1, "beforeend", "<p>x</p>"); err == nil {
		t.Fatalf("nil InsertAdjacentHTML() error = nil, want dom store error")
	}
	if err := nilStore.RemoveNode(1); err == nil {
		t.Fatalf("nil RemoveNode() error = nil, want dom store error")
	}
	if _, err := nilStore.CloneNode(1, true); err == nil {
		t.Fatalf("nil CloneNode() error = nil, want dom store error")
	}

	store := NewStore()
	if err := store.BootstrapHTML(`<div id="target">text</div><p id="sibling">tail</p>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	targetID := mustSelectSingle(t, store, "#target")
	textNode := store.Node(targetID).Children[0]

	if _, err := store.InnerHTMLForNode(999); err == nil {
		t.Fatalf("InnerHTMLForNode(invalid) error = nil, want invalid node error")
	}
	if err := store.SetInnerHTML(999, "<p>x</p>"); err == nil {
		t.Fatalf("SetInnerHTML(invalid) error = nil, want invalid node error")
	}
	if err := store.SetOuterHTML(999, "<p>x</p>"); err == nil {
		t.Fatalf("SetOuterHTML(invalid) error = nil, want invalid node error")
	}
	if err := store.InsertAdjacentHTML(999, "beforeend", "<p>x</p>"); err == nil {
		t.Fatalf("InsertAdjacentHTML(invalid) error = nil, want invalid node error")
	}
	if err := store.RemoveNode(999); err == nil {
		t.Fatalf("RemoveNode(invalid) error = nil, want invalid node error")
	}
	if err := store.RemoveNode(store.DocumentID()); err == nil {
		t.Fatalf("RemoveNode(document) error = nil, want document removal error")
	}
	if _, err := store.CloneNode(999, true); err == nil {
		t.Fatalf("CloneNode(invalid) error = nil, want invalid node error")
	}

	if _, err := store.InnerHTMLForNode(textNode); err == nil {
		t.Fatalf("InnerHTMLForNode(text) error = nil, want non-element error")
	}
	if err := store.SetInnerHTML(textNode, "<p>x</p>"); err == nil {
		t.Fatalf("SetInnerHTML(text) error = nil, want non-element error")
	}
	if err := store.SetOuterHTML(textNode, "<p>x</p>"); err == nil {
		t.Fatalf("SetOuterHTML(text) error = nil, want non-element error")
	}
	if err := store.InsertAdjacentHTML(textNode, "beforeend", "<p>x</p>"); err == nil {
		t.Fatalf("InsertAdjacentHTML(text) error = nil, want non-element error")
	}

	beforeCount := store.NodeCount()
	if err := store.InsertAdjacentHTML(targetID, "sideways", "<p>x</p>"); err == nil {
		t.Fatalf("InsertAdjacentHTML(invalid position) error = nil, want invalid position error")
	}
	if got, want := store.NodeCount(), beforeCount; got != want {
		t.Fatalf("NodeCount() after invalid InsertAdjacentHTML = %d, want %d", got, want)
	}

	if err := store.SetOuterHTML(targetID, `<section id="new"></section>`); err == nil {
		t.Fatalf("SetOuterHTML(document child) error = nil, want document-parent error")
	}
	if err := store.InsertAdjacentHTML(targetID, "beforebegin", `<a id="bb"></a>`); err == nil {
		t.Fatalf("InsertAdjacentHTML(beforebegin on document child) error = nil, want document-parent error")
	}
	if err := store.InsertAdjacentHTML(targetID, "afterend", `<a id="ae"></a>`); err == nil {
		t.Fatalf("InsertAdjacentHTML(afterend on document child) error = nil, want document-parent error")
	}
}
