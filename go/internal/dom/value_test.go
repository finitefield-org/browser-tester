package dom

import "testing"

func TestValueForNodeAndCheckedForNode(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><div id="out">hello</div><input id="name" value="Ada"><input id="agree" type="checkbox" checked><input id="upload" type="file"><textarea id="bio">Hi</textarea><select id="mode"><option value="a">A</option><option value="b" selected>B</option></select></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	outID, ok, err := store.QuerySelector("#out")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#out) = (%d, %v, %v), want ok", outID, ok, err)
	}
	if got, want := store.ValueForNode(outID), "hello"; got != want {
		t.Fatalf("ValueForNode(#out) = %q, want %q", got, want)
	}

	nameID, ok, err := store.QuerySelector("#name")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#name) = (%d, %v, %v), want ok", nameID, ok, err)
	}
	if got, want := store.ValueForNode(nameID), "Ada"; got != want {
		t.Fatalf("ValueForNode(#name) = %q, want %q", got, want)
	}

	bioID, ok, err := store.QuerySelector("#bio")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#bio) = (%d, %v, %v), want ok", bioID, ok, err)
	}
	if got, want := store.ValueForNode(bioID), "Hi"; got != want {
		t.Fatalf("ValueForNode(#bio) = %q, want %q", got, want)
	}

	modeID, ok, err := store.QuerySelector("#mode")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#mode) = (%d, %v, %v), want ok", modeID, ok, err)
	}
	if got, want := store.ValueForNode(modeID), "b"; got != want {
		t.Fatalf("ValueForNode(#mode) = %q, want %q", got, want)
	}

	agreeID, ok, err := store.QuerySelector("#agree")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#agree) = (%d, %v, %v), want ok", agreeID, ok, err)
	}
	checked, ok := store.CheckedForNode(agreeID)
	if !ok || !checked {
		t.Fatalf("CheckedForNode(#agree) = (%v, %v), want (true, true)", checked, ok)
	}

	checked, ok = store.CheckedForNode(nameID)
	if ok {
		t.Fatalf("CheckedForNode(#name) ok = true, want false")
	}
	if checked {
		t.Fatalf("CheckedForNode(#name) checked = true, want false")
	}

	uploadID, ok, err := store.QuerySelector("#upload")
	if err != nil || !ok {
		t.Fatalf("QuerySelector(#upload) = (%d, %v, %v), want ok", uploadID, ok, err)
	}
	if got := store.ValueForNode(uploadID); got != "" {
		t.Fatalf("ValueForNode(#upload) = %q, want empty for bounded file input slice", got)
	}
}
