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

	gotID, ok, err = store.QuerySelector("section + p")
	if err != nil {
		t.Fatalf("QuerySelector(sibling) error = %v", err)
	}
	if !ok || gotID != secondID {
		t.Fatalf("QuerySelector(sibling) = (%d, %v), want (%d, true)", gotID, ok, secondID)
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

	siblingNodes, err := store.QuerySelectorAll("section ~ .item")
	if err != nil {
		t.Fatalf("QuerySelectorAll(sibling) error = %v", err)
	}
	if got, want := siblingNodes.Length(), 2; got != want {
		t.Fatalf("QuerySelectorAll(sibling) len = %d, want %d", got, want)
	}
	if got, ok := siblingNodes.Item(0); !ok || got != secondID {
		t.Fatalf("QuerySelectorAll(sibling).Item(0) = (%d, %v), want (%d, true)", got, ok, secondID)
	}
	if got, ok := siblingNodes.Item(1); !ok || got != thirdID {
		t.Fatalf("QuerySelectorAll(sibling).Item(1) = (%d, %v), want (%d, true)", got, ok, thirdID)
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

	matched, err = store.Matches(secondID, "section + p")
	if err != nil {
		t.Fatalf("Matches(sibling) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(sibling) = false, want true")
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

func TestQueryHelpersSupportDefinedPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><div id="known"></div><x-widget id="widget" defined></x-widget><x-ghost id="ghost"></x-ghost></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	knownID := mustSelectSingle(t, store, "#known")
	widgetID := mustSelectSingle(t, store, "#widget")
	ghostID := mustSelectSingle(t, store, "#ghost")

	gotID, ok, err := store.QuerySelector("div:defined")
	if err != nil {
		t.Fatalf("QuerySelector(div:defined) error = %v", err)
	}
	if !ok || gotID != knownID {
		t.Fatalf("QuerySelector(div:defined) = (%d, %v), want (%d, true)", gotID, ok, knownID)
	}

	matched, err := store.Matches(widgetID, "x-widget:defined")
	if err != nil {
		t.Fatalf("Matches(x-widget:defined) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(x-widget:defined) = false, want true")
	}
	matched, err = store.Matches(ghostID, "x-ghost:defined")
	if err != nil {
		t.Fatalf("Matches(x-ghost:defined) error = %v", err)
	}
	if matched {
		t.Fatalf("Matches(x-ghost:defined) = true, want false")
	}
}

func TestQueryHelpersSupportStatePseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><x-widget id="widget" state="checked pressed"><span id="child">One</span></x-widget><x-widget id="other" state="pressed"></x-widget><div id="plain" state="checked"></div></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	widgetID := mustSelectSingle(t, store, "#widget")
	otherID := mustSelectSingle(t, store, "#other")
	childID := mustSelectSingle(t, store, "#child")

	gotID, ok, err := store.QuerySelector("x-widget:state(checked):state(pressed)")
	if err != nil {
		t.Fatalf("QuerySelector(x-widget:state(checked):state(pressed)) error = %v", err)
	}
	if !ok || gotID != widgetID {
		t.Fatalf("QuerySelector(x-widget:state(checked):state(pressed)) = (%d, %v), want (%d, true)", gotID, ok, widgetID)
	}

	nodes, err := store.QuerySelectorAll("x-widget:state(pressed)")
	if err != nil {
		t.Fatalf("QuerySelectorAll(x-widget:state(pressed)) error = %v", err)
	}
	if got, want := nodes.Length(), 2; got != want {
		t.Fatalf("QuerySelectorAll(x-widget:state(pressed)) len = %d, want %d", got, want)
	}
	if got, ok := nodes.Item(0); !ok || got != widgetID {
		t.Fatalf("QuerySelectorAll(x-widget:state(pressed)).Item(0) = (%d, %v), want (%d, true)", got, ok, widgetID)
	}
	if got, ok := nodes.Item(1); !ok || got != otherID {
		t.Fatalf("QuerySelectorAll(x-widget:state(pressed)).Item(1) = (%d, %v), want (%d, true)", got, ok, otherID)
	}

	matched, err := store.Matches(widgetID, "x-widget:state(checked):state(pressed)")
	if err != nil {
		t.Fatalf("Matches(x-widget:state(checked):state(pressed)) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(x-widget:state(checked):state(pressed)) = false, want true")
	}

	matched, err = store.Matches(otherID, "x-widget:state(checked):state(pressed)")
	if err != nil {
		t.Fatalf("Matches(other x-widget:state(checked):state(pressed)) error = %v", err)
	}
	if matched {
		t.Fatalf("Matches(other x-widget:state(checked):state(pressed)) = true, want false")
	}

	closestID, ok, err := store.Closest(childID, "x-widget:state(checked):state(pressed)")
	if err != nil {
		t.Fatalf("Closest(x-widget:state(checked):state(pressed)) error = %v", err)
	}
	if !ok || closestID != widgetID {
		t.Fatalf("Closest(x-widget:state(checked):state(pressed)) = (%d, %v), want (%d, true)", closestID, ok, widgetID)
	}
}

func TestQueryHelpersSupportAutofillPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><input id="name" autofill value="Ada"><input id="other" value="Bob"></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	nameID := mustSelectSingle(t, store, "#name")
	otherID := mustSelectSingle(t, store, "#other")

	gotID, ok, err := store.QuerySelector("input:autofill")
	if err != nil {
		t.Fatalf("QuerySelector(input:autofill) error = %v", err)
	}
	if !ok || gotID != nameID {
		t.Fatalf("QuerySelector(input:autofill) = (%d, %v), want (%d, true)", gotID, ok, nameID)
	}

	gotID, ok, err = store.QuerySelector("input:-webkit-autofill")
	if err != nil {
		t.Fatalf("QuerySelector(input:-webkit-autofill) error = %v", err)
	}
	if !ok || gotID != nameID {
		t.Fatalf("QuerySelector(input:-webkit-autofill) = (%d, %v), want (%d, true)", gotID, ok, nameID)
	}

	matched, err := store.Matches(nameID, "#name:autofill")
	if err != nil {
		t.Fatalf("Matches(#name:autofill) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(#name:autofill) = false, want true")
	}
	matched, err = store.Matches(otherID, "#other:autofill")
	if err != nil {
		t.Fatalf("Matches(#other:autofill) error = %v", err)
	}
	if matched {
		t.Fatalf("Matches(#other:autofill) = true, want false")
	}
}

func TestQueryHelpersSupportActiveHoverPseudoClasses(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><div id="wrap"><button id="btn" active>Go</button><span id="hovered" hover>Hover</span></div><p id="plain">Text</p></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	wrapID := mustSelectSingle(t, store, "#wrap")
	btnID := mustSelectSingle(t, store, "#btn")
	hoveredID := mustSelectSingle(t, store, "#hovered")

	gotID, ok, err := store.QuerySelector("button:active")
	if err != nil {
		t.Fatalf("QuerySelector(button:active) error = %v", err)
	}
	if !ok || gotID != btnID {
		t.Fatalf("QuerySelector(button:active) = (%d, %v), want (%d, true)", gotID, ok, btnID)
	}

	gotID, ok, err = store.QuerySelector("div:active")
	if err != nil {
		t.Fatalf("QuerySelector(div:active) error = %v", err)
	}
	if !ok || gotID != wrapID {
		t.Fatalf("QuerySelector(div:active) = (%d, %v), want (%d, true)", gotID, ok, wrapID)
	}

	gotID, ok, err = store.QuerySelector("span:hover")
	if err != nil {
		t.Fatalf("QuerySelector(span:hover) error = %v", err)
	}
	if !ok || gotID != hoveredID {
		t.Fatalf("QuerySelector(span:hover) = (%d, %v), want (%d, true)", gotID, ok, hoveredID)
	}

	matched, err := store.Matches(wrapID, "div:active")
	if err != nil {
		t.Fatalf("Matches(div:active) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(div:active) = false, want true")
	}
	matched, err = store.Matches(hoveredID, "span:hover")
	if err != nil {
		t.Fatalf("Matches(span:hover) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(span:hover) = false, want true")
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

func TestQueryHelpersSupportBoundedPseudoClasses(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><input id="enabled" type="text"><input id="flag" type="checkbox" checked><input id="off" type="text" disabled><div id="empty"></div><p id="first">one</p><span id="middle">two</span><p id="last">three</p></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	flagID := mustSelectSingle(t, store, "#flag")
	rootID := mustSelectSingle(t, store, "#root")

	gotID, ok, err := store.QuerySelector("input:checked")
	if err != nil {
		t.Fatalf("QuerySelector(input:checked) error = %v", err)
	}
	if !ok {
		t.Fatalf("QuerySelector(input:checked) ok = false, want true")
	}
	if gotID != flagID {
		t.Fatalf("QuerySelector(input:checked) = %d, want %d", gotID, flagID)
	}

	nodes, err := store.QuerySelectorAll("input:enabled")
	if err != nil {
		t.Fatalf("QuerySelectorAll(input:enabled) error = %v", err)
	}
	if got, want := nodes.Length(), 2; got != want {
		t.Fatalf("QuerySelectorAll(input:enabled) len = %d, want %d", got, want)
	}

	matched, err := store.Matches(gotID, ":checked")
	if err != nil {
		t.Fatalf("Matches(:checked) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(:checked) = false, want true")
	}

	closestID, ok, err := store.Closest(gotID, "main:root")
	if err != nil {
		t.Fatalf("Closest(main:root) error = %v", err)
	}
	if !ok || closestID != rootID {
		t.Fatalf("Closest(main:root) = (%d, %v), want (%d, true)", closestID, ok, rootID)
	}
}

func TestQueryHelpersSupportExtendedPseudoClasses(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><a id="nav" href="/next">Go</a><a id="visited" href="https://example.test/visited">Visited</a><map><area id="area" href="https://example.test/visited" alt="Open"></map><form id="profile"><button id="submit-1" type="submit">Save</button><button id="submit-2" type="submit">Extra</button><input id="checked" type="checkbox" checked><option id="opt" selected>Opt</option></form><input id="placeholder" type="text" placeholder="Name"><textarea id="textarea" placeholder="Story"></textarea><input id="filled" type="text" placeholder="Name" value="Ada"></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	store.SyncCurrentURL("https://example.test/page")
	store.SyncVisitedURLs([]string{"https://example.test/visited"})

	linkID := mustSelectSingle(t, store, "a:link")
	areaID := mustSelectSingle(t, store, "#area")
	if got, ok, err := store.QuerySelector("area:link"); err != nil || ok {
		t.Fatalf("QuerySelector(area:link) = (%d, %v, %v), want no match", got, ok, err)
	}
	if got, ok, err := store.QuerySelector("a:any-link"); err != nil || !ok || got != linkID {
		t.Fatalf("QuerySelector(a:any-link) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, linkID)
	}
	if got, ok, err := store.QuerySelector("area:any-link"); err != nil || !ok || got != areaID {
		t.Fatalf("QuerySelector(area:any-link) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, areaID)
	}
	visitedID := mustSelectSingle(t, store, "#visited")
	if got, ok, err := store.QuerySelector("a:visited"); err != nil || !ok || got != visitedID {
		t.Fatalf("QuerySelector(a:visited) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, visitedID)
	}
	if got, ok, err := store.QuerySelector("area:visited"); err != nil || !ok || got != areaID {
		t.Fatalf("QuerySelector(area:visited) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, areaID)
	}
	submitID := mustSelectSingle(t, store, "#submit-1")
	if got, ok, err := store.QuerySelector("button:default"); err != nil || !ok || got != submitID {
		t.Fatalf("QuerySelector(button:default) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, submitID)
	}
	placeholderID := mustSelectSingle(t, store, "#placeholder")
	if got, ok, err := store.QuerySelector("input:placeholder-shown"); err != nil || !ok || got != placeholderID {
		t.Fatalf("QuerySelector(input:placeholder-shown) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, placeholderID)
	}
	textareaID := mustSelectSingle(t, store, "#textarea")
	if got, ok, err := store.QuerySelector("textarea:placeholder-shown"); err != nil || !ok || got != textareaID {
		t.Fatalf("QuerySelector(textarea:placeholder-shown) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, textareaID)
	}
	if got, ok, err := store.QuerySelector("input:blank"); err != nil || !ok || got != placeholderID {
		t.Fatalf("QuerySelector(input:blank) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, placeholderID)
	}
	if got, ok, err := store.QuerySelector("textarea:blank"); err != nil || !ok || got != textareaID {
		t.Fatalf("QuerySelector(textarea:blank) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, textareaID)
	}
	matched, err := store.Matches(linkID, "a:link")
	if err != nil {
		t.Fatalf("Matches(a:link) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(a:link) = false, want true")
	}
	if matched, err := store.Matches(linkID, "a:any-link"); err != nil {
		t.Fatalf("Matches(a:any-link) error = %v", err)
	} else if !matched {
		t.Fatalf("Matches(a:any-link) = false, want true")
	}
	if matched, err := store.Matches(areaID, "area:any-link"); err != nil {
		t.Fatalf("Matches(area:any-link) error = %v", err)
	} else if !matched {
		t.Fatalf("Matches(area:any-link) = false, want true")
	}
	if matched, err := store.Matches(visitedID, "a:visited"); err != nil {
		t.Fatalf("Matches(a:visited) error = %v", err)
	} else if !matched {
		t.Fatalf("Matches(a:visited) = false, want true")
	}
	if matched, err := store.Matches(areaID, "area:visited"); err != nil {
		t.Fatalf("Matches(area:visited) error = %v", err)
	} else if !matched {
		t.Fatalf("Matches(area:visited) = false, want true")
	}
	if matched, err := store.Matches(visitedID, "a:link"); err != nil {
		t.Fatalf("Matches(a:link) for visited anchor error = %v", err)
	} else if matched {
		t.Fatalf("Matches(a:link) for visited anchor = true, want false")
	}
	if blankMatched, err := store.Matches(placeholderID, "input:blank"); err != nil {
		t.Fatalf("Matches(input:blank) error = %v", err)
	} else if !blankMatched {
		t.Fatalf("Matches(input:blank) = false, want true")
	}
	if blankMatched, err := store.Matches(mustSelectSingle(t, store, "#filled"), "input:blank"); err != nil {
		t.Fatalf("Matches(#filled, input:blank) error = %v", err)
	} else if blankMatched {
		t.Fatalf("Matches(#filled, input:blank) = true, want false")
	}
	closestID, ok, err := store.Closest(linkID, ":root")
	if err != nil {
		t.Fatalf("Closest(:root) error = %v", err)
	}
	if !ok || closestID != mustSelectSingle(t, store, "#root") {
		t.Fatalf("Closest(:root) = (%d, %v), want root", closestID, ok)
	}
}

func TestQueryHelpersSupportLocalLinkPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><a id="self" href="#top">Self</a><a id="next" href="/next">Next</a><map><area id="area-self" href="#top" alt="Self"></map></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}
	store.SyncCurrentURL("https://example.test/page#top")

	selfID := mustSelectSingle(t, store, "#self")
	areaID := mustSelectSingle(t, store, "#area-self")
	if got, ok, err := store.QuerySelector("a:local-link"); err != nil || !ok || got != selfID {
		t.Fatalf("QuerySelector(a:local-link) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, selfID)
	}
	if got, ok, err := store.QuerySelector("area:local-link"); err != nil || !ok || got != areaID {
		t.Fatalf("QuerySelector(area:local-link) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, areaID)
	}
	if matched, err := store.Matches(selfID, "a:local-link"); err != nil || !matched {
		t.Fatalf("Matches(a:local-link) for #self = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(areaID, "area:local-link"); err != nil || !matched {
		t.Fatalf("Matches(area:local-link) for #area-self = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(mustSelectSingle(t, store, "#next"), "a:local-link"); err != nil {
		t.Fatalf("Matches(a:local-link) for #next error = %v", err)
	} else if matched {
		t.Fatalf("Matches(a:local-link) for #next = true, want false")
	}
}

func TestQueryHelpersSupportAttributeSelectors(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><div id="panel" data-kind="panel"><a id="nav" href="/next" data-role="nav">Go</a><input id="name" type="text"><p id="flag" hidden></p><span id="meta" data-tags="alpha beta gamma" data-locale="en-US" data-note="prefix-middle-suffix" data-code="abc123"></span></div></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	panelID := mustSelectSingle(t, store, "#panel")
	navID := mustSelectSingle(t, store, "#nav")
	nameID := mustSelectSingle(t, store, "#name")
	flagID := mustSelectSingle(t, store, "#flag")
	metaID := mustSelectSingle(t, store, "#meta")

	if got, ok, err := store.QuerySelector("div[data-kind]"); err != nil || !ok || got != panelID {
		t.Fatalf("QuerySelector(div[data-kind]) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, panelID)
	}
	if got, ok, err := store.QuerySelector("a[href]"); err != nil || !ok || got != navID {
		t.Fatalf("QuerySelector(a[href]) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, navID)
	}
	if got, ok, err := store.QuerySelector("a[href=\"/next\"]"); err != nil || !ok || got != navID {
		t.Fatalf("QuerySelector(a[href=\"/next\"]) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, navID)
	}
	if got, ok, err := store.QuerySelector("input[type=text]"); err != nil || !ok || got != nameID {
		t.Fatalf("QuerySelector(input[type=text]) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, nameID)
	}
	if got, ok, err := store.QuerySelector("p[hidden]"); err != nil || !ok || got != flagID {
		t.Fatalf("QuerySelector(p[hidden]) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, flagID)
	}
	if got, ok, err := store.QuerySelector("span[data-tags~=beta]"); err != nil || !ok {
		t.Fatalf("QuerySelector(span[data-tags~=beta]) = (%d, %v, %v), want match", got, ok, err)
	}
	if got, ok, err := store.QuerySelector("span[data-locale|=en]"); err != nil || !ok {
		t.Fatalf("QuerySelector(span[data-locale|=en]) = (%d, %v, %v), want match", got, ok, err)
	}
	if got, ok, err := store.QuerySelector("span[data-tags~=BETA i]"); err != nil || !ok {
		t.Fatalf("QuerySelector(span[data-tags~=BETA i]) = (%d, %v, %v), want match", got, ok, err)
	}
	if got, ok, err := store.QuerySelector("input[type=TEXT i]"); err != nil || !ok || got != nameID {
		t.Fatalf("QuerySelector(input[type=TEXT i]) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, nameID)
	}
	if got, ok, err := store.QuerySelector("a[data-role=missing]"); err != nil || ok {
		t.Fatalf("QuerySelector(a[data-role=missing]) = (%d, %v, %v), want no match", got, ok, err)
	}

	nodes, err := store.QuerySelectorAll("a[href]")
	if err != nil {
		t.Fatalf("QuerySelectorAll(a[href]) error = %v", err)
	}
	if got, want := nodes.Length(), 1; got != want {
		t.Fatalf("QuerySelectorAll(a[href]) len = %d, want %d", got, want)
	}
	if got, ok := nodes.Item(0); !ok || got != navID {
		t.Fatalf("QuerySelectorAll(a[href]).Item(0) = (%d, %v), want (%d, true)", got, ok, navID)
	}

	if matched, err := store.Matches(panelID, "div[data-kind]"); err != nil || !matched {
		t.Fatalf("Matches(div[data-kind]) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(navID, "a[href=\"/next\"]"); err != nil || !matched {
		t.Fatalf("Matches(a[href=\"/next\"]) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(metaID, "span[data-tags~=beta]"); err != nil || !matched {
		t.Fatalf("Matches(span[data-tags~=beta]) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(metaID, "span[data-locale|=en]"); err != nil || !matched {
		t.Fatalf("Matches(span[data-locale|=en]) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(metaID, "span[data-tags~=BETA i]"); err != nil || !matched {
		t.Fatalf("Matches(span[data-tags~=BETA i]) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(nameID, "input[type=TEXT i]"); err != nil || !matched {
		t.Fatalf("Matches(input[type=TEXT i]) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(navID, "a[data-role=missing]"); err != nil || matched {
		t.Fatalf("Matches(a[data-role=missing]) = (%v, %v), want (false, nil)", matched, err)
	}
	closestID, ok, err := store.Closest(navID, "div[data-kind]")
	if err != nil {
		t.Fatalf("Closest(div[data-kind]) error = %v", err)
	}
	if !ok || closestID != panelID {
		t.Fatalf("Closest(div[data-kind]) = (%d, %v), want (%d, true)", closestID, ok, panelID)
	}
}

func TestQueryHelpersSupportHeadingLevelPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><h1 id="title">Title</h1><section><h2 id="sub">Sub</h2><div><h4 id="deep">Deep</h4></div></section><article><h6 id="final">Final</h6></article><p id="plain">Body</p></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	titleID := mustSelectSingle(t, store, "#title")
	subID := mustSelectSingle(t, store, "#sub")
	deepID := mustSelectSingle(t, store, "#deep")
	finalID := mustSelectSingle(t, store, "#final")

	gotID, ok, err := store.QuerySelector(":heading(1)")
	if err != nil {
		t.Fatalf("QuerySelector(:heading(1)) error = %v", err)
	}
	if !ok || gotID != titleID {
		t.Fatalf("QuerySelector(:heading(1)) = (%d, %v), want (%d, true)", gotID, ok, titleID)
	}

	nodes, err := store.QuerySelectorAll(":heading(2, 4)")
	if err != nil {
		t.Fatalf("QuerySelectorAll(:heading(2, 4)) error = %v", err)
	}
	if got, want := nodes.Length(), 2; got != want {
		t.Fatalf("QuerySelectorAll(:heading(2, 4)) len = %d, want %d", got, want)
	}
	if got, ok := nodes.Item(0); !ok || got != subID {
		t.Fatalf("QuerySelectorAll(:heading(2, 4)).Item(0) = (%d, %v), want (%d, true)", got, ok, subID)
	}
	if got, ok := nodes.Item(1); !ok || got != deepID {
		t.Fatalf("QuerySelectorAll(:heading(2, 4)).Item(1) = (%d, %v), want (%d, true)", got, ok, deepID)
	}

	matched, err := store.Matches(finalID, ":heading(6)")
	if err != nil {
		t.Fatalf("Matches(:heading(6)) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(#final, :heading(6)) = false, want true")
	}

	closestID, ok, err := store.Closest(deepID, ":heading(4)")
	if err != nil {
		t.Fatalf("Closest(:heading(4)) error = %v", err)
	}
	if !ok || closestID != deepID {
		t.Fatalf("Closest(:heading(4)) = (%d, %v), want (%d, true)", closestID, ok, deepID)
	}

	if _, ok, err := store.QuerySelector(":heading(0)"); err == nil || ok {
		t.Fatalf("QuerySelector(:heading(0)) error = (%v, %v), want invalid heading selector", ok, err)
	}
}

func TestQueryHelpersSupportMediaPseudoClasses(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><audio id="song" src="song.mp3"></audio><video id="film"></video><video id="paused" paused></video><video id="seeking" seeking></video><video id="muted" muted></video><video id="buffering" networkstate="loading" readystate="2"></video><video id="stalled" networkstate="loading" readystate="1" stalled volume-locked></video><div id="other" paused muted></div></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	songID := mustSelectSingle(t, store, "#song")
	pausedID := mustSelectSingle(t, store, "#paused")
	seekingID := mustSelectSingle(t, store, "#seeking")
	mutedID := mustSelectSingle(t, store, "#muted")
	bufferingID := mustSelectSingle(t, store, "#buffering")
	stalledID := mustSelectSingle(t, store, "#stalled")

	gotID, ok, err := store.QuerySelector("audio:playing")
	if err != nil {
		t.Fatalf("QuerySelector(audio:playing) error = %v", err)
	}
	if !ok || gotID != songID {
		t.Fatalf("QuerySelector(audio:playing) = (%d, %v), want (%d, true)", gotID, ok, songID)
	}

	nodes, err := store.QuerySelectorAll("video:buffering")
	if err != nil {
		t.Fatalf("QuerySelectorAll(video:buffering) error = %v", err)
	}
	if got, want := nodes.Length(), 2; got != want {
		t.Fatalf("QuerySelectorAll(video:buffering) len = %d, want %d", got, want)
	}
	if got, ok := nodes.Item(0); !ok || got != bufferingID {
		t.Fatalf("QuerySelectorAll(video:buffering).Item(0) = (%d, %v), want (%d, true)", got, ok, bufferingID)
	}
	if got, ok := nodes.Item(1); !ok || got != stalledID {
		t.Fatalf("QuerySelectorAll(video:buffering).Item(1) = (%d, %v), want (%d, true)", got, ok, stalledID)
	}

	if matched, err := store.Matches(pausedID, ":playing"); err != nil || matched {
		t.Fatalf("Matches(#paused, :playing) = (%v, %v), want (false, nil)", matched, err)
	}
	if matched, err := store.Matches(seekingID, ":seeking"); err != nil || !matched {
		t.Fatalf("Matches(#seeking, :seeking) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(mutedID, ":muted"); err != nil || !matched {
		t.Fatalf("Matches(#muted, :muted) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(stalledID, ":stalled"); err != nil || !matched {
		t.Fatalf("Matches(#stalled, :stalled) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(stalledID, ":volume-locked"); err != nil || !matched {
		t.Fatalf("Matches(#stalled, :volume-locked) = (%v, %v), want (true, nil)", matched, err)
	}
}

func TestQueryHelpersSupportIndeterminatePseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><input id="mixed" type="checkbox" indeterminate><input id="radio-a" type="radio" name="size"><input id="radio-b" type="radio" name="size"><progress id="task"></progress><progress id="done" value="42"></progress></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	mixedID := mustSelectSingle(t, store, "#mixed")
	radioAID := mustSelectSingle(t, store, "#radio-a")
	radioBID := mustSelectSingle(t, store, "#radio-b")
	taskID := mustSelectSingle(t, store, "#task")
	doneID := mustSelectSingle(t, store, "#done")

	gotID, ok, err := store.QuerySelector("input:indeterminate")
	if err != nil {
		t.Fatalf("QuerySelector(input:indeterminate) error = %v", err)
	}
	if !ok || gotID != mixedID {
		t.Fatalf("QuerySelector(input:indeterminate) = (%d, %v), want (%d, true)", gotID, ok, mixedID)
	}

	nodes, err := store.QuerySelectorAll("input:indeterminate")
	if err != nil {
		t.Fatalf("QuerySelectorAll(input:indeterminate) error = %v", err)
	}
	if got, want := nodes.Length(), 3; got != want {
		t.Fatalf("QuerySelectorAll(input:indeterminate) len = %d, want %d", got, want)
	}
	if got, ok := nodes.Item(0); !ok || got != mixedID {
		t.Fatalf("QuerySelectorAll(input:indeterminate).Item(0) = (%d, %v), want (%d, true)", got, ok, mixedID)
	}
	if got, ok := nodes.Item(1); !ok || got != radioAID {
		t.Fatalf("QuerySelectorAll(input:indeterminate).Item(1) = (%d, %v), want (%d, true)", got, ok, radioAID)
	}
	if got, ok := nodes.Item(2); !ok || got != radioBID {
		t.Fatalf("QuerySelectorAll(input:indeterminate).Item(2) = (%d, %v), want (%d, true)", got, ok, radioBID)
	}

	gotID, ok, err = store.QuerySelector("progress:indeterminate")
	if err != nil {
		t.Fatalf("QuerySelector(progress:indeterminate) error = %v", err)
	}
	if !ok || gotID != taskID {
		t.Fatalf("QuerySelector(progress:indeterminate) = (%d, %v), want (%d, true)", gotID, ok, taskID)
	}
	if matched, err := store.Matches(doneID, ":indeterminate"); err != nil || matched {
		t.Fatalf("Matches(#done, :indeterminate) = (%v, %v), want (false, nil)", matched, err)
	}

	if err := store.SetFormControlChecked(radioAID, true); err != nil {
		t.Fatalf("SetFormControlChecked(#radio-a) error = %v", err)
	}
	nodes, err = store.QuerySelectorAll("input:indeterminate")
	if err != nil {
		t.Fatalf("QuerySelectorAll(input:indeterminate) after SetFormControlChecked error = %v", err)
	}
	if got, want := nodes.Length(), 1; got != want {
		t.Fatalf("QuerySelectorAll(input:indeterminate) after SetFormControlChecked len = %d, want %d", got, want)
	}
	if got, ok := nodes.Item(0); !ok || got != mixedID {
		t.Fatalf("QuerySelectorAll(input:indeterminate) after SetFormControlChecked Item(0) = (%d, %v), want (%d, true)", got, ok, mixedID)
	}
	if matched, err := store.Matches(radioAID, ":indeterminate"); err != nil || matched {
		t.Fatalf("Matches(#radio-a, :indeterminate) after SetFormControlChecked = (%v, %v), want (false, nil)", matched, err)
	}
	if matched, err := store.Matches(radioBID, ":indeterminate"); err != nil || matched {
		t.Fatalf("Matches(#radio-b, :indeterminate) after SetFormControlChecked = (%v, %v), want (false, nil)", matched, err)
	}
}

func TestQueryHelpersSupportOfTypePseudoClasses(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><section id="single"><em id="only-child">one</em></section><div id="mixed"><p id="para-a">A</p><span id="only-of-type">S</span><p id="para-b">B</p></div><details id="details" open><summary id="summary-a">A</summary><div id="middle">M</div><summary id="summary-b">B</summary></details></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	onlyChildID := mustSelectSingle(t, store, "#only-child")
	onlyOfTypeID := mustSelectSingle(t, store, "#only-of-type")
	summaryAID := mustSelectSingle(t, store, "#summary-a")
	summaryBID := mustSelectSingle(t, store, "#summary-b")

	if got, ok, err := store.QuerySelector("em:only-child"); err != nil || !ok || got != onlyChildID {
		t.Fatalf("QuerySelector(em:only-child) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, onlyChildID)
	}
	if got, ok, err := store.QuerySelector("span:only-of-type"); err != nil || !ok || got != onlyOfTypeID {
		t.Fatalf("QuerySelector(span:only-of-type) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, onlyOfTypeID)
	}
	if got, ok, err := store.QuerySelector("summary:first-of-type"); err != nil || !ok || got != summaryAID {
		t.Fatalf("QuerySelector(summary:first-of-type) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, summaryAID)
	}
	if got, ok, err := store.QuerySelector("summary:last-of-type"); err != nil || !ok || got != summaryBID {
		t.Fatalf("QuerySelector(summary:last-of-type) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, summaryBID)
	}

	matched, err := store.Matches(summaryAID, "summary:first-of-type")
	if err != nil {
		t.Fatalf("Matches(summary:first-of-type) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(summary:first-of-type) = false, want true")
	}
	matched, err = store.Matches(summaryBID, "summary:last-of-type")
	if err != nil {
		t.Fatalf("Matches(summary:last-of-type) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(summary:last-of-type) = false, want true")
	}
}

func TestQueryHelpersSupportHasPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><section id="wrap"><article id="a1"><span class="hit">Hit</span></article><article id="a2"><span class="miss">Miss</span></article></section><aside id="plain"><span class="hit">Outside</span></aside></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	wrapID := mustSelectSingle(t, store, "#wrap")
	a1ID := mustSelectSingle(t, store, "#a1")
	a2ID := mustSelectSingle(t, store, "#a2")

	if got, ok, err := store.QuerySelector("section:has(.hit)"); err != nil || !ok || got != wrapID {
		t.Fatalf("QuerySelector(section:has(.hit)) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, wrapID)
	}

	nodes, err := store.QuerySelectorAll("article:has(.hit, .miss)")
	if err != nil {
		t.Fatalf("QuerySelectorAll(article:has(.hit, .miss)) error = %v", err)
	}
	if got, want := nodes.Length(), 2; got != want {
		t.Fatalf("QuerySelectorAll(article:has(.hit, .miss)) len = %d, want %d", got, want)
	}

	if matched, err := store.Matches(wrapID, "section:has(article > .hit)"); err != nil || !matched {
		t.Fatalf("Matches(section:has(article > .hit)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(a1ID, "article:has(.hit)"); err != nil || !matched {
		t.Fatalf("Matches(article:has(.hit)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(a2ID, "article:has(.hit)"); err != nil || matched {
		t.Fatalf("Matches(article:has(.hit)) for #a2 = (%v, %v), want (false, nil)", matched, err)
	}
}

func TestQueryHelpersSupportNotPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><section id="wrap"><article id="a1" class="match"><span class="hit">Hit</span></article><article id="a2"><span class="miss">Miss</span></article></section><aside id="plain"><span class="hit">Outside</span></aside></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	a2ID := mustSelectSingle(t, store, "#a2")

	if got, ok, err := store.QuerySelector("section:not(.missing)"); err != nil || !ok || got != mustSelectSingle(t, store, "#wrap") {
		t.Fatalf("QuerySelector(section:not(.missing)) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, mustSelectSingle(t, store, "#wrap"))
	}

	nodes, err := store.QuerySelectorAll("article:not(.match, .other)")
	if err != nil {
		t.Fatalf("QuerySelectorAll(article:not(.match, .other)) error = %v", err)
	}
	if got, want := nodes.Length(), 1; got != want {
		t.Fatalf("QuerySelectorAll(article:not(.match, .other)) len = %d, want %d", got, want)
	}

	if matched, err := store.Matches(a2ID, "article:not(.match)"); err != nil || !matched {
		t.Fatalf("Matches(article:not(.match)) for #a2 = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(mustSelectSingle(t, store, "#a1"), "article:not(.match)"); err != nil || matched {
		t.Fatalf("Matches(article:not(.match)) for #a1 = (%v, %v), want (false, nil)", matched, err)
	}
}

func TestQueryHelpersSupportIsAndWherePseudoClasses(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><section id="wrap" class="match"><article id="a1" class="hit">One</article><article id="a2" class="miss">Two</article></section><aside id="plain"><span class="hit">Outside</span></aside></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	wrapID := mustSelectSingle(t, store, "#wrap")
	a1ID := mustSelectSingle(t, store, "#a1")
	a2ID := mustSelectSingle(t, store, "#a2")

	if got, ok, err := store.QuerySelector("section:is(#wrap, .missing)"); err != nil || !ok || got != wrapID {
		t.Fatalf("QuerySelector(section:is(#wrap, .missing)) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, wrapID)
	}

	nodes, err := store.QuerySelectorAll("article:where(.hit, .miss)")
	if err != nil {
		t.Fatalf("QuerySelectorAll(article:where(.hit, .miss)) error = %v", err)
	}
	if got, want := nodes.Length(), 2; got != want {
		t.Fatalf("QuerySelectorAll(article:where(.hit, .miss)) len = %d, want %d", got, want)
	}

	if matched, err := store.Matches(wrapID, "section:is(#wrap)"); err != nil || !matched {
		t.Fatalf("Matches(section:is(#wrap)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(a1ID, "article:where(.hit)"); err != nil || !matched {
		t.Fatalf("Matches(article:where(.hit)) for #a1 = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(a2ID, "article:is(.hit)"); err != nil || matched {
		t.Fatalf("Matches(article:is(.hit)) for #a2 = (%v, %v), want (false, nil)", matched, err)
	}
}

func TestQueryHelpersSupportScopePseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><section id="panel"><p id="child">one</p></section><p id="sibling">two</p></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	rootID := mustSelectSingle(t, store, "#root")
	panelID := mustSelectSingle(t, store, "#panel")
	childID := mustSelectSingle(t, store, "#child")

	if got, ok, err := store.QuerySelector(":scope"); err != nil || !ok || got != rootID {
		t.Fatalf("QuerySelector(:scope) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, rootID)
	}
	if got, ok, err := store.QuerySelector(":scope > section"); err != nil || !ok || got != panelID {
		t.Fatalf("QuerySelector(:scope > section) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, panelID)
	}
	if matched, err := store.Matches(rootID, ":scope"); err != nil || !matched {
		t.Fatalf("Matches(#root, :scope) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(panelID, ":scope"); err != nil || !matched {
		t.Fatalf("Matches(#panel, :scope) = (%v, %v), want (true, nil)", matched, err)
	}
	if closestID, ok, err := store.Closest(childID, ":scope"); err != nil || !ok || closestID != childID {
		t.Fatalf("Closest(#child, :scope) = (%d, %v, %v), want (%d, true, nil)", closestID, ok, err, childID)
	}
}

func TestQueryHelpersSupportNthPseudoClasses(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><ul id="list"><li id="one">1</li><li id="two">2</li><li id="three">3</li><li id="four">4</li><li id="five">5</li></ul><div id="mixed"><p id="para-a">A</p><span id="mid">M</span><p id="para-b">B</p><p id="para-c">C</p></div></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	threeID := mustSelectSingle(t, store, "#three")
	paraCID := mustSelectSingle(t, store, "#para-c")
	midID := mustSelectSingle(t, store, "#mid")
	fiveID := mustSelectSingle(t, store, "#five")
	paraAID := mustSelectSingle(t, store, "#para-a")
	paraBID := mustSelectSingle(t, store, "#para-b")

	if got, ok, err := store.QuerySelector("li:nth-child(3)"); err != nil || !ok || got != threeID {
		t.Fatalf("QuerySelector(li:nth-child(3)) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, threeID)
	}
	if got, ok, err := store.QuerySelector("li:nth-last-child(1)"); err != nil || !ok || got != fiveID {
		t.Fatalf("QuerySelector(li:nth-last-child(1)) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, fiveID)
	}

	nodes, err := store.QuerySelectorAll("li:nth-child(odd)")
	if err != nil {
		t.Fatalf("QuerySelectorAll(li:nth-child(odd)) error = %v", err)
	}
	if got, want := nodes.Length(), 3; got != want {
		t.Fatalf("QuerySelectorAll(li:nth-child(odd)) len = %d, want %d", got, want)
	}

	if matched, err := store.Matches(paraCID, "p:nth-of-type(3)"); err != nil || !matched {
		t.Fatalf("Matches(p:nth-of-type(3)) for #para-c = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(midID, "span:nth-of-type(1)"); err != nil || !matched {
		t.Fatalf("Matches(span:nth-of-type(1)) for #mid = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(paraCID, "p:nth-last-of-type(1)"); err != nil || !matched {
		t.Fatalf("Matches(p:nth-last-of-type(1)) for #para-c = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(paraBID, "p:nth-last-of-type(2)"); err != nil || !matched {
		t.Fatalf("Matches(p:nth-last-of-type(2)) for #para-b = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(paraAID, "p:nth-last-of-type(3)"); err != nil || !matched {
		t.Fatalf("Matches(p:nth-last-of-type(3)) for #para-a = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(fiveID, "li:nth-last-child(1)"); err != nil || !matched {
		t.Fatalf("Matches(li:nth-last-child(1)) for #five = (%v, %v), want (true, nil)", matched, err)
	}
}

func TestQueryHelpersSupportConstraintValidationPseudoClasses(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><form id="valid-form"><input id="name" type="text" required value="Ada"><input id="age" type="number" min="1" max="10" value="5"><select id="mode"><option value="a" selected>A</option><option value="b">B</option></select></form><form id="invalid-form"><input id="missing" type="text" required><input id="low" type="number" min="1" max="10" value="0"><input id="high" type="number" min="1" max="10" value="11"></form></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	nameID := mustSelectSingle(t, store, "#name")
	missingID := mustSelectSingle(t, store, "#missing")
	ageID := mustSelectSingle(t, store, "#age")
	lowID := mustSelectSingle(t, store, "#low")
	modeID := mustSelectSingle(t, store, "#mode")
	validFormID := mustSelectSingle(t, store, "#valid-form")
	invalidFormID := mustSelectSingle(t, store, "#invalid-form")

	if err := store.SetUserValidity(nameID, true); err != nil {
		t.Fatalf("SetUserValidity(#name) error = %v", err)
	}
	if err := store.SetUserValidity(missingID, true); err != nil {
		t.Fatalf("SetUserValidity(#missing) error = %v", err)
	}
	if err := store.SetUserValidity(ageID, true); err != nil {
		t.Fatalf("SetUserValidity(#age) error = %v", err)
	}
	if err := store.SetUserValidity(lowID, true); err != nil {
		t.Fatalf("SetUserValidity(#low) error = %v", err)
	}
	if err := store.SetUserValidity(modeID, true); err != nil {
		t.Fatalf("SetUserValidity(#mode) error = %v", err)
	}

	if got, ok, err := store.QuerySelector("input:valid"); err != nil || !ok || got != nameID {
		t.Fatalf("QuerySelector(input:valid) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, nameID)
	}
	if got, ok, err := store.QuerySelector("input:invalid"); err != nil || !ok || got != missingID {
		t.Fatalf("QuerySelector(input:invalid) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, missingID)
	}
	if got, ok, err := store.QuerySelector("input:in-range"); err != nil || !ok || got != ageID {
		t.Fatalf("QuerySelector(input:in-range) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, ageID)
	}
	if got, ok, err := store.QuerySelector("input:out-of-range"); err != nil || !ok || got != lowID {
		t.Fatalf("QuerySelector(input:out-of-range) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, lowID)
	}
	if matched, err := store.Matches(modeID, "select:valid"); err != nil || !matched {
		t.Fatalf("Matches(select:valid) = (%v, %v), want (true, nil)", matched, err)
	}
	if got, ok, err := store.QuerySelector("input:user-valid"); err != nil || !ok || got != nameID {
		t.Fatalf("QuerySelector(input:user-valid) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, nameID)
	}
	if got, ok, err := store.QuerySelector("input:user-invalid"); err != nil || !ok || got != missingID {
		t.Fatalf("QuerySelector(input:user-invalid) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, missingID)
	}
	if got, ok, err := store.QuerySelector("select:user-valid"); err != nil || !ok || got != modeID {
		t.Fatalf("QuerySelector(select:user-valid) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, modeID)
	}
	if got, ok, err := store.QuerySelector("form:valid"); err != nil || !ok || got != validFormID {
		t.Fatalf("QuerySelector(form:valid) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, validFormID)
	}
	if got, ok, err := store.QuerySelector("form:invalid"); err != nil || !ok || got != invalidFormID {
		t.Fatalf("QuerySelector(form:invalid) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, invalidFormID)
	}
}

func TestQueryHelpersSupportMoreBoundedPseudoClasses(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><h1 id="title">Title</h1><details id="details" open><summary>Sum</summary></details><dialog id="dialog" open></dialog><form id="profile"><input id="required" type="text" required><input id="optional" type="text"><input id="readonly" type="text" readonly><textarea id="editable"></textarea><textarea id="readonly-ta" readonly>Locked</textarea></form></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	requiredID := mustSelectSingle(t, store, "#required")
	optionalID := mustSelectSingle(t, store, "#optional")
	readonlyID := mustSelectSingle(t, store, "#readonly")
	editableID := mustSelectSingle(t, store, "#editable")
	readonlyTextareaID := mustSelectSingle(t, store, "#readonly-ta")
	titleID := mustSelectSingle(t, store, "#title")
	detailsID := mustSelectSingle(t, store, "#details")
	dialogID := mustSelectSingle(t, store, "#dialog")

	gotID, ok, err := store.QuerySelector("input:required")
	if err != nil {
		t.Fatalf("QuerySelector(input:required) error = %v", err)
	}
	if !ok || gotID != requiredID {
		t.Fatalf("QuerySelector(input:required) = (%d, %v), want (%d, true)", gotID, ok, requiredID)
	}

	optionalNodes, err := store.QuerySelectorAll("input:optional")
	if err != nil {
		t.Fatalf("QuerySelectorAll(input:optional) error = %v", err)
	}
	if got, want := optionalNodes.Length(), 2; got != want {
		t.Fatalf("QuerySelectorAll(input:optional) len = %d, want %d", got, want)
	}

	readWriteNodes, err := store.QuerySelectorAll("input:read-write")
	if err != nil {
		t.Fatalf("QuerySelectorAll(input:read-write) error = %v", err)
	}
	if got, want := readWriteNodes.Length(), 2; got != want {
		t.Fatalf("QuerySelectorAll(input:read-write) len = %d, want %d", got, want)
	}

	gotID, ok, err = store.QuerySelector("input:read-only")
	if err != nil {
		t.Fatalf("QuerySelector(input:read-only) error = %v", err)
	}
	if !ok || gotID != readonlyID {
		t.Fatalf("QuerySelector(input:read-only) = (%d, %v), want (%d, true)", gotID, ok, readonlyID)
	}

	gotID, ok, err = store.QuerySelector("textarea:read-write")
	if err != nil {
		t.Fatalf("QuerySelector(textarea:read-write) error = %v", err)
	}
	if !ok || gotID != editableID {
		t.Fatalf("QuerySelector(textarea:read-write) = (%d, %v), want (%d, true)", gotID, ok, editableID)
	}

	gotID, ok, err = store.QuerySelector("textarea:read-only")
	if err != nil {
		t.Fatalf("QuerySelector(textarea:read-only) error = %v", err)
	}
	if !ok || gotID != readonlyTextareaID {
		t.Fatalf("QuerySelector(textarea:read-only) = (%d, %v), want (%d, true)", gotID, ok, readonlyTextareaID)
	}

	gotID, ok, err = store.QuerySelector("h1:heading")
	if err != nil {
		t.Fatalf("QuerySelector(h1:heading) error = %v", err)
	}
	if !ok || gotID != titleID {
		t.Fatalf("QuerySelector(h1:heading) = (%d, %v), want (%d, true)", gotID, ok, titleID)
	}

	gotID, ok, err = store.QuerySelector("details:open")
	if err != nil {
		t.Fatalf("QuerySelector(details:open) error = %v", err)
	}
	if !ok || gotID != detailsID {
		t.Fatalf("QuerySelector(details:open) = (%d, %v), want (%d, true)", gotID, ok, detailsID)
	}

	gotID, ok, err = store.QuerySelector("dialog:open")
	if err != nil {
		t.Fatalf("QuerySelector(dialog:open) error = %v", err)
	}
	if !ok || gotID != dialogID {
		t.Fatalf("QuerySelector(dialog:open) = (%d, %v), want (%d, true)", gotID, ok, dialogID)
	}

	matched, err := store.Matches(optionalID, "input:optional")
	if err != nil {
		t.Fatalf("Matches(input:optional) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(input:optional) = false, want true")
	}
	matched, err = store.Matches(readonlyID, "input:read-only")
	if err != nil {
		t.Fatalf("Matches(input:read-only) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(input:read-only) = false, want true")
	}
	matched, err = store.Matches(titleID, ":heading")
	if err != nil {
		t.Fatalf("Matches(:heading) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(:heading) = false, want true")
	}
}

func TestQueryHelpersSupportModalPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><dialog id="dialog" modal></dialog><video id="player" fullscreen></video><div id="other" open></div></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	dialogID := mustSelectSingle(t, store, "#dialog")
	playerID := mustSelectSingle(t, store, "#player")

	gotID, ok, err := store.QuerySelector("dialog:modal")
	if err != nil {
		t.Fatalf("QuerySelector(dialog:modal) error = %v", err)
	}
	if !ok || gotID != dialogID {
		t.Fatalf("QuerySelector(dialog:modal) = (%d, %v), want (%d, true)", gotID, ok, dialogID)
	}

	gotID, ok, err = store.QuerySelector("video:modal")
	if err != nil {
		t.Fatalf("QuerySelector(video:modal) error = %v", err)
	}
	if !ok || gotID != playerID {
		t.Fatalf("QuerySelector(video:modal) = (%d, %v), want (%d, true)", gotID, ok, playerID)
	}

	matched, err := store.Matches(dialogID, "dialog:modal")
	if err != nil {
		t.Fatalf("Matches(dialog:modal) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(dialog:modal) = false, want true")
	}
	matched, err = store.Matches(playerID, "video:modal")
	if err != nil {
		t.Fatalf("Matches(video:modal) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(video:modal) = false, want true")
	}
	matched, err = store.Matches(mustSelectSingle(t, store, "#other"), "#other:modal")
	if err != nil {
		t.Fatalf("Matches(#other:modal) error = %v", err)
	}
	if matched {
		t.Fatalf("Matches(#other:modal) = true, want false")
	}
}

func TestQueryHelpersSupportPopoverOpenPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><div id="menu" popover popover-open></div><div id="closed" popover></div><dialog id="dialog" open></dialog></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	menuID := mustSelectSingle(t, store, "#menu")
	closedID := mustSelectSingle(t, store, "#closed")

	gotID, ok, err := store.QuerySelector("div:popover-open")
	if err != nil {
		t.Fatalf("QuerySelector(div:popover-open) error = %v", err)
	}
	if !ok || gotID != menuID {
		t.Fatalf("QuerySelector(div:popover-open) = (%d, %v), want (%d, true)", gotID, ok, menuID)
	}

	matched, err := store.Matches(menuID, "#menu:popover-open")
	if err != nil {
		t.Fatalf("Matches(#menu:popover-open) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(#menu:popover-open) = false, want true")
	}
	matched, err = store.Matches(closedID, "#closed:popover-open")
	if err != nil {
		t.Fatalf("Matches(#closed:popover-open) error = %v", err)
	}
	if matched {
		t.Fatalf("Matches(#closed:popover-open) = true, want false")
	}
}

func TestQueryHelpersSupportFocusPseudoClasses(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><section id="panel"><button id="cta">Go</button><input id="name"></section><aside id="sidebar"><input id="secondary"></aside></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	nameID := mustSelectSingle(t, store, "#name")
	panelID := mustSelectSingle(t, store, "#panel")
	rootID := mustSelectSingle(t, store, "#root")

	if err := store.SetFocusedNode(nameID); err != nil {
		t.Fatalf("SetFocusedNode(#name) error = %v", err)
	}

	if got, ok, err := store.QuerySelector("input:focus"); err != nil || !ok || got != nameID {
		t.Fatalf("QuerySelector(input:focus) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, nameID)
	}
	if got, ok, err := store.QuerySelector("input:focus-visible"); err != nil || !ok || got != nameID {
		t.Fatalf("QuerySelector(input:focus-visible) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, nameID)
	}
	if got, ok, err := store.QuerySelector("section:focus-within"); err != nil || !ok || got != panelID {
		t.Fatalf("QuerySelector(section:focus-within) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, panelID)
	}
	if got, ok, err := store.QuerySelector("main:focus-within"); err != nil || !ok || got != rootID {
		t.Fatalf("QuerySelector(main:focus-within) = (%d, %v, %v), want (%d, true, nil)", got, ok, err, rootID)
	}

	nodes, err := store.QuerySelectorAll(":focus-within")
	if err != nil {
		t.Fatalf("QuerySelectorAll(:focus-within) error = %v", err)
	}
	if got, want := nodes.Length(), 3; got != want {
		t.Fatalf("QuerySelectorAll(:focus-within) len = %d, want %d", got, want)
	}

	matched, err := store.Matches(nameID, ":focus")
	if err != nil {
		t.Fatalf("Matches(:focus) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(:focus) = false, want true")
	}
	if matched, err := store.Matches(nameID, ":focus-visible"); err != nil || !matched {
		t.Fatalf("Matches(:focus-visible) error = (%v, %v), want (true, nil)", matched, err)
	}
}

func TestQueryHelpersSupportTargetPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><section id="panel"><a name="legacy">legacy</a><div id="space target"><p id="inner">space</p></div></section><p id="tail">tail</p></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	legacyID := mustSelectSingle(t, store, "a")
	spaceID := mustSelectSingle(t, store, "#inner")
	panelID := mustSelectSingle(t, store, "#panel")
	rootID := mustSelectSingle(t, store, "#root")

	store.SyncTargetFromURL("https://example.test/page#legacy")
	gotID, ok, err := store.QuerySelector(":target")
	if err != nil {
		t.Fatalf("QuerySelector(:target) error = %v", err)
	}
	if !ok || gotID != legacyID {
		t.Fatalf("QuerySelector(:target) = (%d, %v), want (%d, true)", gotID, ok, legacyID)
	}

	nodes, err := store.QuerySelectorAll(":target")
	if err != nil {
		t.Fatalf("QuerySelectorAll(:target) error = %v", err)
	}
	if got, want := nodes.Length(), 1; got != want {
		t.Fatalf("QuerySelectorAll(:target) len = %d, want %d", got, want)
	}

	nodes, err = store.QuerySelectorAll(":target-within")
	if err != nil {
		t.Fatalf("QuerySelectorAll(:target-within) error = %v", err)
	}
	if got, want := nodes.Length(), 3; got != want {
		t.Fatalf("QuerySelectorAll(:target-within) len = %d, want %d", got, want)
	}

	store.SyncTargetFromURL("https://example.test/page#inner")
	gotID, ok, err = store.QuerySelector(":target")
	if err != nil {
		t.Fatalf("QuerySelector(:target) after encoded fragment error = %v", err)
	}
	if !ok || gotID != spaceID {
		t.Fatalf("QuerySelector(:target) after encoded fragment = (%d, %v), want (%d, true)", gotID, ok, spaceID)
	}
	if gotID, ok, err := store.QuerySelector("main:target-within"); err != nil || !ok || gotID != rootID {
		t.Fatalf("QuerySelector(main:target-within) = (%d, %v, %v), want (%d, true, nil)", gotID, ok, err, rootID)
	}
	if gotID, ok, err := store.QuerySelector("section:target-within"); err != nil || !ok || gotID != panelID {
		t.Fatalf("QuerySelector(section:target-within) = (%d, %v, %v), want (%d, true, nil)", gotID, ok, err, panelID)
	}
}

func TestQueryHelpersSupportLangPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root" lang="en-US"><section id="panel"><p id="inherited">Hello</p></section><article id="french" lang="fr"><span id="direct">Salut</span><div id="unknown" lang=""><em id="blank">Nada</em></div></article></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	inheritedID := mustSelectSingle(t, store, "#inherited")
	directID := mustSelectSingle(t, store, "#direct")
	rootID := mustSelectSingle(t, store, "#root")

	gotID, ok, err := store.QuerySelector("p:lang(en)")
	if err != nil {
		t.Fatalf("QuerySelector(p:lang(en)) error = %v", err)
	}
	if !ok || gotID != inheritedID {
		t.Fatalf("QuerySelector(p:lang(en)) = (%d, %v), want (%d, true)", gotID, ok, inheritedID)
	}

	nodes, err := store.QuerySelectorAll("main:lang(en)")
	if err != nil {
		t.Fatalf("QuerySelectorAll(main:lang(en)) error = %v", err)
	}
	if got, want := nodes.Length(), 1; got != want {
		t.Fatalf("QuerySelectorAll(main:lang(en)) len = %d, want %d", got, want)
	}

	matched, err := store.Matches(directID, "span:lang(fr)")
	if err != nil {
		t.Fatalf("Matches(span:lang(fr)) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(span:lang(fr)) = false, want true")
	}

	closestID, ok, err := store.Closest(inheritedID, "main:lang(en)")
	if err != nil {
		t.Fatalf("Closest(main:lang(en)) error = %v", err)
	}
	if !ok || closestID != rootID {
		t.Fatalf("Closest(main:lang(en)) = (%d, %v), want (%d, true)", closestID, ok, rootID)
	}
}

func TestQueryHelpersSupportDirPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root" dir="rtl"><section id="panel"><p id="inherited">Hello</p><div id="auto-ltr" dir="auto">abc</div><div id="auto-rtl" dir="auto">مرحبا</div></section><article id="ltr" dir="ltr"><span id="nested">Salut</span></article></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	inheritedID := mustSelectSingle(t, store, "#inherited")
	autoLTRID := mustSelectSingle(t, store, "#auto-ltr")
	autoRTLID := mustSelectSingle(t, store, "#auto-rtl")
	nestedID := mustSelectSingle(t, store, "#nested")
	articleID := mustSelectSingle(t, store, "#ltr")

	gotID, ok, err := store.QuerySelector("p:dir(rtl)")
	if err != nil {
		t.Fatalf("QuerySelector(p:dir(rtl)) error = %v", err)
	}
	if !ok || gotID != inheritedID {
		t.Fatalf("QuerySelector(p:dir(rtl)) = (%d, %v), want (%d, true)", gotID, ok, inheritedID)
	}

	nodes, err := store.QuerySelectorAll("div:dir(ltr)")
	if err != nil {
		t.Fatalf("QuerySelectorAll(div:dir(ltr)) error = %v", err)
	}
	if got, want := nodes.Length(), 1; got != want {
		t.Fatalf("QuerySelectorAll(div:dir(ltr)) len = %d, want %d", got, want)
	}

	matched, err := store.Matches(autoRTLID, "div:dir(rtl)")
	if err != nil {
		t.Fatalf("Matches(div:dir(rtl)) error = %v", err)
	}
	if !matched {
		t.Fatalf("Matches(div:dir(rtl)) = false, want true")
	}

	closestID, ok, err := store.Closest(nestedID, "article:dir(ltr)")
	if err != nil {
		t.Fatalf("Closest(article:dir(ltr)) error = %v", err)
	}
	if !ok || closestID != articleID {
		t.Fatalf("Closest(article:dir(ltr)) = (%d, %v), want (%d, true)", closestID, ok, articleID)
	}

	if matched, err := store.Matches(autoLTRID, "div:dir(ltr)"); err != nil || !matched {
		t.Fatalf("Matches(#auto-ltr, div:dir(ltr)) = (%v, %v), want (true, nil)", matched, err)
	}
}
