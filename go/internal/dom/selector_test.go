package dom

import "testing"

func TestSelectSimpleSelectors(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<div id="main">` +
			`<p class="item primary">first</p>` +
			`<p class="item">second</p>` +
			`<span class="item auxiliary">third</span>` +
			`</div>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "div", wantLen: 1},
		{selector: "#main", wantLen: 1},
		{selector: ".item", wantLen: 3},
		{selector: "p.item", wantLen: 2},
		{selector: "p.primary", wantLen: 1},
		{selector: "*.auxiliary", wantLen: 1},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectBoundedCombinators(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<div id="main">` +
			`<section class="pane"><p class="item primary">first</p></section>` +
			`<p class="item">second</p>` +
			`<span class="item auxiliary">third</span>` +
			`<p class="item tail">fourth</p>` +
			`</div>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "div p", wantLen: 3},
		{selector: "div > p", wantLen: 2},
		{selector: "div > section > p.primary", wantLen: 1},
		{selector: "#main > .item", wantLen: 3},
		{selector: "section .primary", wantLen: 1},
		{selector: "section + p", wantLen: 1},
		{selector: "p ~ span", wantLen: 1},
		{selector: "section ~ p.tail", wantLen: 1},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectBoundedPseudoClasses(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><a id="nav" href="/next">Go</a><map><area id="area" href="/popup" alt="Open"></map><form id="profile"><button id="submit-1" type="submit">Save</button><button id="submit-2" type="submit">Extra</button><input id="checked" type="checkbox" checked><option id="opt" selected>Opt</option></form><input id="enabled" type="text"><input id="placeholder" type="text" placeholder="Name"><textarea id="textarea" placeholder="Story"></textarea><input id="filled" type="text" placeholder="Name" value="Ada"><input id="off" type="text" disabled><div id="empty"></div><p id="first">one</p><span id="middle">two</span><p id="last">three</p></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: ":root", wantLen: 1},
		{selector: "main:root", wantLen: 1},
		{selector: "input:enabled", wantLen: 4},
		{selector: "input:disabled", wantLen: 1},
		{selector: "input:checked", wantLen: 1},
		{selector: "div:empty", wantLen: 1},
		{selector: "a:first-child", wantLen: 1},
		{selector: "p:last-child", wantLen: 1},
		{selector: "main > form > button:default", wantLen: 1},
		{selector: "input:default", wantLen: 1},
		{selector: "option:default", wantLen: 1},
		{selector: "a:link", wantLen: 1},
		{selector: "area:link", wantLen: 1},
		{selector: "a:any-link", wantLen: 1},
		{selector: "area:any-link", wantLen: 1},
		{selector: "input:placeholder-shown", wantLen: 1},
		{selector: "textarea:placeholder-shown", wantLen: 1},
		{selector: "input:blank", wantLen: 3},
		{selector: "textarea:blank", wantLen: 1},
		{selector: "main > form > input:checked", wantLen: 1},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectLocalLinkPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><a id="self" href="#top">Self</a><a id="next" href="/next">Next</a><map><area id="area-self" href="#top" alt="Self"></map></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}
	store.SyncCurrentURL("https://example.test/page#top")

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "a:local-link", wantLen: 1},
		{selector: "area:local-link", wantLen: 1},
		{selector: "#self:local-link", wantLen: 1},
		{selector: "#next:local-link", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectVisitedPseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><a id="visited" href="https://example.test/visited">Visited</a><a id="other" href="https://example.test/other">Other</a><map><area id="area" href="https://example.test/visited" alt="Visited"></map></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}
	store.SyncCurrentURL("https://example.test/page")
	store.SyncVisitedURLs([]string{"https://example.test/visited"})

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "a:link", wantLen: 1},
		{selector: "a:visited", wantLen: 1},
		{selector: "area:visited", wantLen: 1},
		{selector: "#visited:visited", wantLen: 1},
		{selector: "#visited:link", wantLen: 0},
		{selector: "#area:visited", wantLen: 1},
		{selector: "#other:visited", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectDefinedPseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><div id="known"></div><x-widget id="widget" defined></x-widget><x-ghost id="ghost"></x-ghost></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "div:defined", wantLen: 1},
		{selector: "x-widget:defined", wantLen: 1},
		{selector: "#ghost:defined", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectStatePseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><x-widget id="primary" state="checked pressed"><span>One</span></x-widget><x-widget id="secondary" state="pressed"><span>Two</span></x-widget><div id="plain" state="checked"></div></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "x-widget:state(checked)", wantLen: 1},
		{selector: "x-widget:state(pressed)", wantLen: 2},
		{selector: "x-widget:state(checked):state(pressed)", wantLen: 1},
		{selector: "#plain:state(checked)", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectStatePseudoClassRejectsInvalidIdentifiers(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><x-widget id="widget" state="checked"></x-widget></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	for _, selector := range []string{
		"x-widget:state()",
		"x-widget:state(1bad)",
	} {
		if _, err := store.Select(selector); err == nil {
			t.Fatalf("Select(%q) error = nil, want selector error", selector)
		}
	}
}

func TestSelectAutofillPseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><input id="name" autofill value="Ada"><input id="other" value="Bob"></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "input:autofill", wantLen: 1},
		{selector: "input:-webkit-autofill", wantLen: 1},
		{selector: "#name:autofill", wantLen: 1},
		{selector: "#other:autofill", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectActiveHoverPseudoClasses(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><div id="wrap"><button id="btn" active>Go</button><span id="hovered" hover>Hover</span></div><p id="plain">Text</p></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "button:active", wantLen: 1},
		{selector: "div:active", wantLen: 1},
		{selector: "span:hover", wantLen: 1},
		{selector: "div:hover", wantLen: 1},
		{selector: "#plain:active", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectIndeterminatePseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><input id="mixed" type="checkbox" indeterminate><input id="radio-a" type="radio" name="size"><input id="radio-b" type="radio" name="size"><progress id="task"></progress><progress id="done" value="42"></progress></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	mixedID := mustSelectSingle(t, store, "#mixed")
	radioAID := mustSelectSingle(t, store, "#radio-a")
	radioBID := mustSelectSingle(t, store, "#radio-b")
	taskID := mustSelectSingle(t, store, "#task")
	doneID := mustSelectSingle(t, store, "#done")

	if got, err := store.Select("input:indeterminate"); err != nil || len(got) != 3 {
		t.Fatalf("Select(input:indeterminate) = (%v, %v), want three indeterminate inputs", got, err)
	}
	if got, err := store.Select("progress:indeterminate"); err != nil || len(got) != 1 {
		t.Fatalf("Select(progress:indeterminate) = (%v, %v), want one indeterminate progress", got, err)
	}
	if matched, err := store.Matches(mixedID, ":indeterminate"); err != nil || !matched {
		t.Fatalf("Matches(#mixed, :indeterminate) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(taskID, ":indeterminate"); err != nil || !matched {
		t.Fatalf("Matches(#task, :indeterminate) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(doneID, ":indeterminate"); err != nil || matched {
		t.Fatalf("Matches(#done, :indeterminate) = (%v, %v), want (false, nil)", matched, err)
	}

	if err := store.SetFormControlChecked(radioAID, true); err != nil {
		t.Fatalf("SetFormControlChecked(#radio-a) error = %v", err)
	}
	if got, err := store.Select("input:indeterminate"); err != nil || len(got) != 1 || got[0] != mixedID {
		t.Fatalf("Select(input:indeterminate) after SetFormControlChecked = (%v, %v), want only checkbox", got, err)
	}
	if matched, err := store.Matches(radioAID, ":indeterminate"); err != nil || matched {
		t.Fatalf("Matches(#radio-a, :indeterminate) after SetFormControlChecked = (%v, %v), want (false, nil)", matched, err)
	}
	if matched, err := store.Matches(radioBID, ":indeterminate"); err != nil || matched {
		t.Fatalf("Matches(#radio-b, :indeterminate) after SetFormControlChecked = (%v, %v), want (false, nil)", matched, err)
	}
}

func TestSelectHasPseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><section id="wrap"><article id="a1"><span class="hit">Hit</span></article><article id="a2"><span class="miss">Miss</span></article></section><aside id="plain"><span class="hit">Outside</span></aside></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "section:has(.hit)", wantLen: 1},
		{selector: "section:has(article > .hit)", wantLen: 1},
		{selector: "article:has(.hit, .miss)", wantLen: 2},
		{selector: "section:has(.missing)", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectHasPseudoClassRejectsInvalidSelectors(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><section><article><span class="hit"></span></article></section></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	for _, selector := range []string{
		"section:has()",
		"section:has(> .hit)",
	} {
		if _, err := store.Select(selector); err == nil {
			t.Fatalf("Select(%q) error = nil, want selector error", selector)
		}
	}
}

func TestSelectNotPseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><section id="wrap"><article id="a1" class="match"><span class="hit">Hit</span></article><article id="a2"><span class="miss">Miss</span></article></section><aside id="plain"><span class="hit">Outside</span></aside></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "section:not(.missing)", wantLen: 1},
		{selector: "article:not(.match)", wantLen: 1},
		{selector: "article:not(.match, .other)", wantLen: 1},
		{selector: "#a1:not(.match)", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectNotPseudoClassRejectsInvalidSelectors(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><section><article><span class="hit"></span></article></section></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	for _, selector := range []string{
		"section:not()",
		"section:not(.hit, )",
	} {
		if _, err := store.Select(selector); err == nil {
			t.Fatalf("Select(%q) error = nil, want selector error", selector)
		}
	}
}

func TestSelectIsAndWherePseudoClasses(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><section id="wrap" class="match"><article id="a1" class="hit">One</article><article id="a2" class="miss">Two</article></section><aside id="plain"><span class="hit">Outside</span></aside></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "section:is(#wrap, .missing)", wantLen: 1},
		{selector: "section:where(#wrap)", wantLen: 1},
		{selector: "article:where(.hit, .miss)", wantLen: 2},
		{selector: "article:is(.hit)", wantLen: 1},
		{selector: "#plain:is(.hit)", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectIsAndWherePseudoClassesRejectInvalidSelectors(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><section id="wrap"></section></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	for _, selector := range []string{
		":is()",
		":where()",
		":is(> .hit)",
		":where(> .hit)",
	} {
		if _, err := store.Select(selector); err == nil {
			t.Fatalf("Select(%q) error = nil, want selector error", selector)
		}
	}
}

func TestSelectScopePseudoClass(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main id="root"><section id="panel"><p id="child">one</p></section><p id="sibling">two</p></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: ":scope", wantLen: 1},
		{selector: ":scope > section", wantLen: 1},
		{selector: ":scope > p", wantLen: 1},
		{selector: "section :scope", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectMoreBoundedPseudoClasses(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><h1 id="title">Title</h1><details id="details" open><summary>Sum</summary></details><dialog id="dialog" open></dialog><form id="profile"><input id="required" type="text" required><input id="optional" type="text"><input id="readonly" type="text" readonly><textarea id="editable"></textarea><textarea id="readonly-ta" readonly>Locked</textarea></form></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "input:required", wantLen: 1},
		{selector: "input:optional", wantLen: 2},
		{selector: "input:read-write", wantLen: 2},
		{selector: "input:read-only", wantLen: 1},
		{selector: "textarea:read-write", wantLen: 1},
		{selector: "textarea:read-only", wantLen: 1},
		{selector: "h1:heading", wantLen: 1},
		{selector: "h6:heading", wantLen: 0},
		{selector: "details:open", wantLen: 1},
		{selector: "dialog:open", wantLen: 1},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectModalPseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><dialog id="dialog" modal></dialog><video id="player" fullscreen></video><div id="other" open></div></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "dialog:modal", wantLen: 1},
		{selector: "video:modal", wantLen: 1},
		{selector: "#other:modal", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectPopoverOpenPseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><div id="menu" popover popover-open></div><div id="closed" popover></div><dialog id="dialog" open></dialog></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "div:popover-open", wantLen: 1},
		{selector: "#menu:popover-open", wantLen: 1},
		{selector: "#closed:popover-open", wantLen: 0},
		{selector: "dialog:popover-open", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectHeadingLevelPseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><h1 id="title">Title</h1><section><h2 id="sub">Sub</h2><div><h4 id="deep">Deep</h4></div></section><article><h6 id="final">Final</h6></article><p id="plain">Body</p></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	titleID := mustSelectSingle(t, store, "#title")
	subID := mustSelectSingle(t, store, "#sub")
	deepID := mustSelectSingle(t, store, "#deep")
	finalID := mustSelectSingle(t, store, "#final")

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: ":heading(1)", wantLen: 1},
		{selector: ":heading(2, 4)", wantLen: 2},
		{selector: ":heading(1, 2, 4, 6)", wantLen: 4},
		{selector: "h4:heading(4)", wantLen: 1},
		{selector: "p:heading(1)", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}

	if matched, err := store.Matches(titleID, ":heading(1)"); err != nil || !matched {
		t.Fatalf("Matches(#title, :heading(1)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(subID, ":heading(2)"); err != nil || !matched {
		t.Fatalf("Matches(#sub, :heading(2)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(deepID, ":heading(2, 4)"); err != nil || !matched {
		t.Fatalf("Matches(#deep, :heading(2, 4)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(finalID, ":heading(2, 4)"); err != nil || matched {
		t.Fatalf("Matches(#final, :heading(2, 4)) = (%v, %v), want (false, nil)", matched, err)
	}

	if _, err := store.Select(":heading(0)"); err == nil {
		t.Fatalf("Select(:heading(0)) error = nil, want invalid heading level")
	}
	if _, err := store.Select(":heading(7)"); err == nil {
		t.Fatalf("Select(:heading(7)) error = nil, want invalid heading level")
	}
}

func TestSelectMediaPseudoClasses(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><audio id="song" src="song.mp3"></audio><video id="film"></video><video id="paused" paused></video><video id="seeking" seeking></video><video id="muted" muted></video><video id="buffering" networkstate="loading" readystate="2"></video><video id="stalled" networkstate="loading" readystate="1" stalled volume-locked></video><div id="other" paused muted></div></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	songID := mustSelectSingle(t, store, "#song")
	pausedID := mustSelectSingle(t, store, "#paused")
	seekingID := mustSelectSingle(t, store, "#seeking")
	mutedID := mustSelectSingle(t, store, "#muted")
	bufferingID := mustSelectSingle(t, store, "#buffering")
	stalledID := mustSelectSingle(t, store, "#stalled")
	otherID := mustSelectSingle(t, store, "#other")

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "audio:playing", wantLen: 1},
		{selector: "video:paused", wantLen: 1},
		{selector: "video:seeking", wantLen: 1},
		{selector: "video:muted", wantLen: 1},
		{selector: "video:buffering", wantLen: 2},
		{selector: "video:stalled", wantLen: 1},
		{selector: "video:volume-locked", wantLen: 1},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}

	if matched, err := store.Matches(songID, ":playing"); err != nil || !matched {
		t.Fatalf("Matches(#song, :playing) = (%v, %v), want (true, nil)", matched, err)
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
	if matched, err := store.Matches(bufferingID, ":buffering"); err != nil || !matched {
		t.Fatalf("Matches(#buffering, :buffering) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(stalledID, ":stalled"); err != nil || !matched {
		t.Fatalf("Matches(#stalled, :stalled) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(stalledID, ":volume-locked"); err != nil || !matched {
		t.Fatalf("Matches(#stalled, :volume-locked) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(otherID, ":paused"); err != nil || matched {
		t.Fatalf("Matches(#other, :paused) = (%v, %v), want (false, nil)", matched, err)
	}
}

func TestSelectFocusPseudoClasses(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><section id="panel"><button id="cta">Go</button><input id="name"></section><aside id="sidebar"><input id="secondary"></aside></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	nameID := mustSelectSingle(t, store, "#name")
	panelID := mustSelectSingle(t, store, "#panel")
	rootID := mustSelectSingle(t, store, "#root")
	sidebarID := mustSelectSingle(t, store, "#sidebar")

	if err := store.SetFocusedNode(nameID); err != nil {
		t.Fatalf("SetFocusedNode(#name) error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "input:focus", wantLen: 1},
		{selector: "input:focus-visible", wantLen: 1},
		{selector: "section:focus-within", wantLen: 1},
		{selector: "main:focus-within", wantLen: 1},
		{selector: "aside:focus-within", wantLen: 0},
		{selector: "button:focus", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}

	if matched, err := store.Matches(nameID, ":focus"); err != nil || !matched {
		t.Fatalf("Matches(#name, :focus) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(panelID, ":focus-within"); err != nil || !matched {
		t.Fatalf("Matches(#panel, :focus-within) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(rootID, ":focus-within"); err != nil || !matched {
		t.Fatalf("Matches(#root, :focus-within) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(sidebarID, ":focus-within"); err != nil || matched {
		t.Fatalf("Matches(#sidebar, :focus-within) = (%v, %v), want (false, nil)", matched, err)
	}
}

func TestSelectTargetPseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(`<main id="root"><section id="panel"><a name="legacy">legacy</a><div id="space target"><p id="inner">space</p></div></section><p id="tail">tail</p></main>`)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	legacyID := mustSelectSingle(t, store, "a")
	spaceID := mustSelectSingle(t, store, "#inner")
	panelID := mustSelectSingle(t, store, "#panel")
	rootID := mustSelectSingle(t, store, "#root")

	store.SyncTargetFromURL("https://example.test/page#legacy")
	if got := store.TargetNodeID(); got != legacyID {
		t.Fatalf("TargetNodeID() after legacy fragment = %d, want %d", got, legacyID)
	}
	if got, err := store.Select(":target"); err != nil || len(got) != 1 || got[0] != legacyID {
		t.Fatalf("Select(:target) after legacy fragment = (%v, %v), want one legacy anchor", got, err)
	}
	if got, err := store.Select(":target-within"); err != nil || len(got) != 3 {
		t.Fatalf("Select(:target-within) after legacy fragment = (%v, %v), want three ancestors", got, err)
	}

	store.SyncTargetFromURL("https://example.test/page#inner")
	if got := store.TargetNodeID(); got != spaceID {
		t.Fatalf("TargetNodeID() after encoded fragment = %d, want %d", got, spaceID)
	}
	if got, err := store.Select(":target"); err != nil || len(got) != 1 || got[0] != spaceID {
		t.Fatalf("Select(:target) after encoded fragment = (%v, %v), want one encoded target", got, err)
	}
	if got, err := store.Select(":target-within"); err != nil || len(got) != 4 {
		t.Fatalf("Select(:target-within) after encoded fragment = (%v, %v), want four nodes", got, err)
	}
	if matched, err := store.Matches(panelID, ":target-within"); err != nil || !matched {
		t.Fatalf("Matches(#panel, :target-within) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(rootID, ":target-within"); err != nil || !matched {
		t.Fatalf("Matches(#root, :target-within) = (%v, %v), want (true, nil)", matched, err)
	}

	store.SyncTargetFromURL("https://example.test/page#missing")
	if got := store.TargetNodeID(); got != 0 {
		t.Fatalf("TargetNodeID() after missing fragment = %d, want 0", got)
	}
	if got, err := store.Select(":target"); err != nil || len(got) != 0 {
		t.Fatalf("Select(:target) after missing fragment = (%v, %v), want no matches", got, err)
	}
	if got, err := store.Select(":target-within"); err != nil || len(got) != 0 {
		t.Fatalf("Select(:target-within) after missing fragment = (%v, %v), want no matches", got, err)
	}

	store.SyncTargetFromURL("https://example.test/page#top")
	if got := store.TargetNodeID(); got != 0 {
		t.Fatalf("TargetNodeID() after top fragment = %d, want 0", got)
	}
}

func TestSelectLangPseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root" lang="en-US"><section id="panel"><p id="inherited">Hello</p></section><article id="french" lang="fr"><span id="direct">Salut</span><div id="unknown" lang=""><em id="blank">Nada</em></div></article></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	inheritedID := mustSelectSingle(t, store, "#inherited")
	directID := mustSelectSingle(t, store, "#direct")
	frenchID := mustSelectSingle(t, store, "#french")
	blankID := mustSelectSingle(t, store, "#blank")

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "main:lang(en)", wantLen: 1},
		{selector: "section:lang(en)", wantLen: 1},
		{selector: "p:lang(en)", wantLen: 1},
		{selector: "article:lang(fr)", wantLen: 1},
		{selector: "span:lang(fr)", wantLen: 1},
		{selector: "em:lang(fr)", wantLen: 0},
		{selector: "main:lang(de)", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}

	if matched, err := store.Matches(inheritedID, "p:lang(en)"); err != nil || !matched {
		t.Fatalf("Matches(#inherited, p:lang(en)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(directID, "span:lang(fr)"); err != nil || !matched {
		t.Fatalf("Matches(#direct, span:lang(fr)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(frenchID, "article:lang(fr)"); err != nil || !matched {
		t.Fatalf("Matches(#french, article:lang(fr)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(blankID, "em:lang(fr)"); err != nil || matched {
		t.Fatalf("Matches(#blank, em:lang(fr)) = (%v, %v), want (false, nil)", matched, err)
	}
}

func TestSelectDirPseudoClass(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root" dir="rtl"><section id="panel"><p id="inherited">Hello</p><div id="auto-ltr" dir="auto">abc</div><div id="auto-rtl" dir="auto">مرحبا</div></section><article id="ltr" dir="ltr"><span id="nested">Salut</span></article></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	inheritedID := mustSelectSingle(t, store, "#inherited")
	autoLTRID := mustSelectSingle(t, store, "#auto-ltr")
	autoRTLID := mustSelectSingle(t, store, "#auto-rtl")
	nestedID := mustSelectSingle(t, store, "#nested")
	articleID := mustSelectSingle(t, store, "#ltr")

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "main:dir(rtl)", wantLen: 1},
		{selector: "section:dir(rtl)", wantLen: 1},
		{selector: "p:dir(rtl)", wantLen: 1},
		{selector: "div:dir(ltr)", wantLen: 1},
		{selector: "div:dir(rtl)", wantLen: 1},
		{selector: "article:dir(ltr)", wantLen: 1},
		{selector: "span:dir(ltr)", wantLen: 1},
		{selector: "span:dir(rtl)", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}

	if matched, err := store.Matches(inheritedID, "p:dir(rtl)"); err != nil || !matched {
		t.Fatalf("Matches(#inherited, p:dir(rtl)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(autoLTRID, "div:dir(ltr)"); err != nil || !matched {
		t.Fatalf("Matches(#auto-ltr, div:dir(ltr)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(autoRTLID, "div:dir(rtl)"); err != nil || !matched {
		t.Fatalf("Matches(#auto-rtl, div:dir(rtl)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(nestedID, "span:dir(ltr)"); err != nil || !matched {
		t.Fatalf("Matches(#nested, span:dir(ltr)) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(articleID, "article:dir(ltr)"); err != nil || !matched {
		t.Fatalf("Matches(#ltr, article:dir(ltr)) = (%v, %v), want (true, nil)", matched, err)
	}
}

func TestSelectOfTypePseudoClasses(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><section id="single"><em id="only-child">one</em></section><div id="mixed"><p id="para-a">A</p><span id="only-of-type">S</span><p id="para-b">B</p></div><details id="details" open><summary id="summary-a">A</summary><div id="middle">M</div><summary id="summary-b">B</summary></details></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "em:only-child", wantLen: 1},
		{selector: "em:only-of-type", wantLen: 1},
		{selector: "span:only-of-type", wantLen: 1},
		{selector: "span:only-child", wantLen: 0},
		{selector: "summary:first-of-type", wantLen: 1},
		{selector: "summary:last-of-type", wantLen: 1},
		{selector: "summary:only-of-type", wantLen: 0},
		{selector: "summary:only-child", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectNthPseudoClasses(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><ul id="list"><li id="one">1</li><li id="two">2</li><li id="three">3</li><li id="four">4</li><li id="five">5</li></ul><div id="mixed"><p id="para-a">A</p><span id="mid">M</span><p id="para-b">B</p><p id="para-c">C</p></div></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "li:nth-child(odd)", wantLen: 3},
		{selector: "li:nth-child(even)", wantLen: 2},
		{selector: "li:nth-child(3)", wantLen: 1},
		{selector: "li:nth-child(2n+1)", wantLen: 3},
		{selector: "p:nth-of-type(2)", wantLen: 1},
		{selector: "p:nth-of-type(odd)", wantLen: 2},
		{selector: "span:nth-of-type(1)", wantLen: 1},
		{selector: "li:nth-last-child(1)", wantLen: 1},
		{selector: "li:nth-last-child(2)", wantLen: 1},
		{selector: "li:nth-last-child(odd)", wantLen: 3},
		{selector: "p:nth-last-of-type(1)", wantLen: 1},
		{selector: "p:nth-last-of-type(2)", wantLen: 1},
		{selector: "span:nth-last-of-type(1)", wantLen: 1},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}
}

func TestSelectNthPseudoClassesRejectsInvalidSelectors(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><ul><li id="one">1</li><li id="two">2</li></ul></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	for _, selector := range []string{
		"li:nth-child()",
		"li:nth-child(foo)",
		"li:nth-of-type(2n+)",
		"li:nth-last-child()",
		"li:nth-last-child(foo)",
		"li:nth-last-of-type(2n+)",
	} {
		if _, err := store.Select(selector); err == nil {
			t.Fatalf("Select(%q) error = nil, want selector error", selector)
		}
	}
}

func TestSelectConstraintValidationPseudoClasses(t *testing.T) {
	store := NewStore()
	err := store.BootstrapHTML(
		`<main id="root"><form id="valid-form"><input id="name" type="text" required value="Ada"><input id="age" type="number" min="1" max="10" value="5"><select id="mode"><option value="a" selected>A</option><option value="b">B</option></select></form><form id="invalid-form"><input id="missing" type="text" required><input id="low" type="number" min="1" max="10" value="0"><input id="high" type="number" min="1" max="10" value="11"></form></main>`,
	)
	if err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	if ids, err := store.Select("#name"); err != nil || len(ids) != 1 {
		t.Fatalf("Select(#name) for user-validity prep = (%v, %v), want one node", ids, err)
	} else if err := store.SetUserValidity(ids[0], true); err != nil {
		t.Fatalf("SetUserValidity(#name) error = %v", err)
	}
	if ids, err := store.Select("#missing"); err != nil || len(ids) != 1 {
		t.Fatalf("Select(#missing) for user-validity prep = (%v, %v), want one node", ids, err)
	} else if err := store.SetUserValidity(ids[0], true); err != nil {
		t.Fatalf("SetUserValidity(#missing) error = %v", err)
	}
	if ids, err := store.Select("#age"); err != nil || len(ids) != 1 {
		t.Fatalf("Select(#age) for user-validity prep = (%v, %v), want one node", ids, err)
	} else if err := store.SetUserValidity(ids[0], true); err != nil {
		t.Fatalf("SetUserValidity(#age) error = %v", err)
	}
	if ids, err := store.Select("#low"); err != nil || len(ids) != 1 {
		t.Fatalf("Select(#low) for user-validity prep = (%v, %v), want one node", ids, err)
	} else if err := store.SetUserValidity(ids[0], true); err != nil {
		t.Fatalf("SetUserValidity(#low) error = %v", err)
	}
	if ids, err := store.Select("#mode"); err != nil || len(ids) != 1 {
		t.Fatalf("Select(#mode) for user-validity prep = (%v, %v), want one node", ids, err)
	} else if err := store.SetUserValidity(ids[0], true); err != nil {
		t.Fatalf("SetUserValidity(#mode) error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "input:valid", wantLen: 2},
		{selector: "input:invalid", wantLen: 3},
		{selector: "input:in-range", wantLen: 1},
		{selector: "input:out-of-range", wantLen: 2},
		{selector: "select:valid", wantLen: 1},
		{selector: "input:user-valid", wantLen: 2},
		{selector: "input:user-invalid", wantLen: 2},
		{selector: "select:user-valid", wantLen: 1},
		{selector: "select:user-invalid", wantLen: 0},
		{selector: "form:valid", wantLen: 1},
		{selector: "form:invalid", wantLen: 1},
		{selector: "form:user-valid", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}

	nameID := mustSelectSingle(t, store, "#name")
	missingID := mustSelectSingle(t, store, "#missing")
	ageID := mustSelectSingle(t, store, "#age")
	lowID := mustSelectSingle(t, store, "#low")
	modeID := mustSelectSingle(t, store, "#mode")
	validFormID := mustSelectSingle(t, store, "#valid-form")
	invalidFormID := mustSelectSingle(t, store, "#invalid-form")

	if matched, err := store.Matches(nameID, ":valid"); err != nil || !matched {
		t.Fatalf("Matches(#name, :valid) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(missingID, ":invalid"); err != nil || !matched {
		t.Fatalf("Matches(#missing, :invalid) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(ageID, ":in-range"); err != nil || !matched {
		t.Fatalf("Matches(#age, :in-range) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(lowID, ":out-of-range"); err != nil || !matched {
		t.Fatalf("Matches(#low, :out-of-range) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(modeID, "select:valid"); err != nil || !matched {
		t.Fatalf("Matches(#mode, select:valid) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(nameID, ":user-valid"); err != nil || !matched {
		t.Fatalf("Matches(#name, :user-valid) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(missingID, ":user-invalid"); err != nil || !matched {
		t.Fatalf("Matches(#missing, :user-invalid) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(validFormID, "form:valid"); err != nil || !matched {
		t.Fatalf("Matches(#valid-form, form:valid) = (%v, %v), want (true, nil)", matched, err)
	}
	if matched, err := store.Matches(invalidFormID, "form:invalid"); err != nil || !matched {
		t.Fatalf("Matches(#invalid-form, form:invalid) = (%v, %v), want (true, nil)", matched, err)
	}
}

func TestSelectBoundedAttributeSelectors(t *testing.T) {
	store := NewStore()
	if err := store.BootstrapHTML(`<main><div id="root" data-kind="panel"><a id="nav" href="/next" data-role="nav">Go</a><input id="name" type="text"><p id="flag" hidden></p><span id="meta" data-tags="alpha beta gamma" data-locale="en-US" data-note="prefix-middle-suffix" data-code="abc123"></span></div></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	tests := []struct {
		selector string
		wantLen  int
	}{
		{selector: "div[data-kind]", wantLen: 1},
		{selector: "a[href]", wantLen: 1},
		{selector: "a[href=\"/next\"]", wantLen: 1},
		{selector: "a[data-role=nav]", wantLen: 1},
		{selector: "input[type=text]", wantLen: 1},
		{selector: "p[hidden]", wantLen: 1},
		{selector: "span[data-tags~=beta]", wantLen: 1},
		{selector: "span[data-locale|=en]", wantLen: 1},
		{selector: "span[data-note^=prefix]", wantLen: 1},
		{selector: "span[data-note$=suffix]", wantLen: 1},
		{selector: "span[data-note*=middle]", wantLen: 1},
		{selector: "span[data-tags~=BETA i]", wantLen: 1},
		{selector: "span[data-locale|=EN i]", wantLen: 1},
		{selector: "span[data-note^=PREFIX i]", wantLen: 1},
		{selector: "span[data-note$=SUFFIX i]", wantLen: 1},
		{selector: "span[data-note*=MIDDLE i]", wantLen: 1},
		{selector: "input[type=TEXT i]", wantLen: 1},
		{selector: "a[data-role=missing]", wantLen: 0},
		{selector: "span[data-tags~=delta]", wantLen: 0},
		{selector: "span[data-tags~=BETA s]", wantLen: 0},
		{selector: "input[type=TEXT s]", wantLen: 0},
	}

	for _, tc := range tests {
		got, err := store.Select(tc.selector)
		if err != nil {
			t.Fatalf("Select(%q) error = %v", tc.selector, err)
		}
		if len(got) != tc.wantLen {
			t.Fatalf("Select(%q) len = %d, want %d", tc.selector, len(got), tc.wantLen)
		}
	}

	if _, err := store.Select("div[item="); err == nil {
		t.Fatalf("Select(\"div[item=\") error = nil, want unsupported selector error")
	}
	if _, err := store.Select("div:bogus"); err == nil {
		t.Fatalf("Select(\"div:bogus\") error = nil, want unsupported pseudo-class error")
	}
}
