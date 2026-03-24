package dom

import "testing"

func TestQueryHelpersReuseSelectorEngine(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(
		`<div id="root">` +
			`<section class="pane"><p id="first" class="item primary">one</p></section>` +
			`<p id="second" class="item">two</p>` +
			`<span id="third" class="item auxiliary">three</span>` +
			`</div>`,
	); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	firstID := mustSelectSingle(t, store, "#first")
	secondID := mustSelectSingle(t, store, "#second")
	thirdID := mustSelectSingle(t, store, "#third")
	sectionID := mustSelectSingle(t, store, "section")
	rootID := mustSelectSingle(t, store, "#root")

	gotID, ok, err := store.QuerySelector("div > section > p.primary")
	if err != nil {
		t.Fatalf("QuerySelector() error = %v", err)
	}
	if !ok || gotID != firstID {
		t.Fatalf("QuerySelector() = (%d, %v), want (%d, true)", gotID, ok, firstID)
	}

	nodes, err := store.QuerySelectorAll("div .item")
	if err != nil {
		t.Fatalf("QuerySelectorAll() error = %v", err)
	}
	if got, want := nodes.Length(), 3; got != want {
		t.Fatalf("QuerySelectorAll() len = %d, want %d", got, want)
	}
	if got, ok := nodes.Item(0); !ok || got != firstID {
		t.Fatalf("QuerySelectorAll().Item(0) = (%d, %v), want (%d, true)", got, ok, firstID)
	}
	if got, ok := nodes.Item(1); !ok || got != secondID {
		t.Fatalf("QuerySelectorAll().Item(1) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := nodes.Item(2); !ok || got != thirdID {
		t.Fatalf("QuerySelectorAll().Item(2) = (%d, %v), want (%d, true)", got, ok, thirdID)
	}
	if got, ok := nodes.Item(3); ok || got != 0 {
		t.Fatalf("QuerySelectorAll().Item(3) = (%d, %v), want (0, false)", got, ok)
	}

	ids := nodes.IDs()
	ids[0] = 999
	if got, ok := nodes.Item(0); !ok || got != firstID {
		t.Fatalf("QuerySelectorAll() snapshot mutated via IDs() = (%d, %v), want (%d, true)", got, ok, firstID)
	}

	matched, err := store.Matches(firstID, "div > section > p.primary")
	if err != nil {
		t.Fatalf("Matches() error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches() = false, want true")
	}

	matched, err = store.Matches(secondID, "div > section > p.primary")
	if err != nil {
		t.Fatalf("Matches() error = %v", err)
	}
	if matched {
		t.Fatalf("Matches() = true, want false")
	}

	closestID, ok, err := store.Closest(firstID, "section")
	if err != nil {
		t.Fatalf("Closest() error = %v", err)
	}
	if !ok || closestID != sectionID {
		t.Fatalf("Closest() = (%d, %v), want (%d, true)", closestID, ok, sectionID)
	}

	closestID, ok, err = store.Closest(firstID, "div > section")
	if err != nil {
		t.Fatalf("Closest() error = %v", err)
	}
	if !ok || closestID != sectionID {
		t.Fatalf("Closest() = (%d, %v), want (%d, true)", closestID, ok, sectionID)
	}

	closestID, ok, err = store.Closest(firstID, "div")
	if err != nil {
		t.Fatalf("Closest() error = %v", err)
	}
	if !ok || closestID != rootID {
		t.Fatalf("Closest() = (%d, %v), want (%d, true)", closestID, ok, rootID)
	}
}

func TestQueryHelpersHandleMissingMatchesAndInvalidInputs(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><p id="one">x</p></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	gotID, ok, err := store.QuerySelector("#missing")
	if err != nil {
		t.Fatalf("QuerySelector() error = %v", err)
	}
	if ok || gotID != 0 {
		t.Fatalf("QuerySelector(#missing) = (%d, %v), want (0, false)", gotID, ok)
	}

	var nilStore *Store
	if _, _, err := nilStore.QuerySelector("div"); err == nil {
		t.Fatalf("nil QuerySelector() error = nil, want dom store error")
	}
	if _, err := nilStore.QuerySelectorAll("div"); err == nil {
		t.Fatalf("nil QuerySelectorAll() error = nil, want dom store error")
	}
	if _, err := nilStore.Matches(1, "div"); err == nil {
		t.Fatalf("nil Matches() error = nil, want dom store error")
	}
	if _, _, err := nilStore.Closest(1, "div"); err == nil {
		t.Fatalf("nil Closest() error = nil, want dom store error")
	}

	if _, err := store.Matches(999, "div"); err == nil {
		t.Fatalf("Matches(invalid node) error = nil, want invalid node error")
	}
	if _, _, err := store.Closest(999, "div"); err == nil {
		t.Fatalf("Closest(invalid node) error = nil, want invalid node error")
	}
}
