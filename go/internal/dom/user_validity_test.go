package dom

import "testing"

func TestSetUserValidityTracksSupportedControls(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><input id="name" type="text"><select id="mode"><option>A</option></select><textarea id="bio"></textarea></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	nameID := mustSelectSingle(t, store, "#name")
	modeID := mustSelectSingle(t, store, "#mode")
	bioID := mustSelectSingle(t, store, "#bio")

	if err := store.SetUserValidity(nameID, true); err != nil {
		t.Fatalf("SetUserValidity(#name) error = %v", err)
	}
	if err := store.SetUserValidity(modeID, true); err != nil {
		t.Fatalf("SetUserValidity(#mode) error = %v", err)
	}
	if err := store.SetUserValidity(bioID, true); err != nil {
		t.Fatalf("SetUserValidity(#bio) error = %v", err)
	}

	for _, nodeID := range []NodeID{nameID, modeID, bioID} {
		if node := store.Node(nodeID); node == nil || !node.UserValidity {
			t.Fatalf("node(%d).UserValidity = %v, want true", nodeID, node)
		}
	}
}

func TestSetUserValidityRejectsUnsupportedTargets(t *testing.T) {
	var nilStore *Store
	if err := nilStore.SetUserValidity(1, true); err == nil {
		t.Fatalf("nil SetUserValidity() error = nil, want dom store error")
	}

	store := NewStore()
	if err := store.BootstrapHTML(`<main><div id="box"></div><input id="name"></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	if err := store.SetUserValidity(999, true); err == nil {
		t.Fatalf("SetUserValidity(invalid) error = nil, want invalid node error")
	}

	boxID := mustSelectSingle(t, store, "#box")
	if err := store.SetUserValidity(boxID, true); err == nil {
		t.Fatalf("SetUserValidity(non-control) error = nil, want unsupported control error")
	}
}
