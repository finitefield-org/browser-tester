package browsertester

import "testing"

func TestDebugViewReportsRandomSeedWhenConfigured(t *testing.T) {
	harness, err := NewHarnessBuilder().RandomSeed(42).Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	got, ok := harness.Debug().RandomSeed()
	if !ok {
		t.Fatalf("Debug().RandomSeed() ok = false, want true")
	}
	if got != 42 {
		t.Fatalf("Debug().RandomSeed() = %d, want 42", got)
	}

	defaultHarness, err := NewHarnessBuilder().Build()
	if err != nil {
		t.Fatalf("Build() default error = %v", err)
	}
	if got, ok := defaultHarness.Debug().RandomSeed(); ok || got != 0 {
		t.Fatalf("default Debug().RandomSeed() = (%d, %v), want (0, false)", got, ok)
	}
}

func TestMatchMediaContract(t *testing.T) {
	harness, err := NewHarnessBuilder().
		MatchMedia(map[string]bool{"(prefers-reduced-motion: reduce)": true}).
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	got, err := harness.MatchMedia("(prefers-reduced-motion: reduce)")
	if err != nil {
		t.Fatalf("MatchMedia() error = %v", err)
	}
	if !got {
		t.Fatalf("MatchMedia() = false, want true")
	}

	if _, err := harness.MatchMedia("(prefers-color-scheme: dark)"); err == nil {
		t.Fatalf("MatchMedia(unseeded) error = nil, want mock error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindMock {
		t.Fatalf("MatchMedia(unseeded) error = %#v, want mock error", err)
	}
}

func TestDebugViewReportsScrollPosition(t *testing.T) {
	harness, err := FromHTML(`<main><div>scroll</div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if gotX, gotY := harness.Debug().ScrollPosition(); gotX != 0 || gotY != 0 {
		t.Fatalf("Debug().ScrollPosition() = (%d, %d), want (0, 0)", gotX, gotY)
	}

	if err := harness.ScrollTo(13, 21); err != nil {
		t.Fatalf("ScrollTo() error = %v", err)
	}
	if gotX, gotY := harness.Debug().ScrollPosition(); gotX != 13 || gotY != 21 {
		t.Fatalf("Debug().ScrollPosition() after ScrollTo = (%d, %d), want (13, 21)", gotX, gotY)
	}

	if err := harness.ScrollBy(2, -1); err != nil {
		t.Fatalf("ScrollBy() error = %v", err)
	}
	if gotX, gotY := harness.Debug().ScrollPosition(); gotX != 15 || gotY != 20 {
		t.Fatalf("Debug().ScrollPosition() after ScrollBy = (%d, %d), want (15, 20)", gotX, gotY)
	}

	var nilHarness *Harness
	if gotX, gotY := nilHarness.Debug().ScrollPosition(); gotX != 0 || gotY != 0 {
		t.Fatalf("nil Debug().ScrollPosition() = (%d, %d), want (0, 0)", gotX, gotY)
	}
}

func TestConstructorHelpersCaptureExpectedState(t *testing.T) {
	t.Run("FromHTML", func(t *testing.T) {
		harness, err := FromHTML("<main>one</main>")
		if err != nil {
			t.Fatalf("FromHTML() error = %v", err)
		}
		if got, want := harness.URL(), "https://app.local/"; got != want {
			t.Fatalf("URL() = %q, want %q", got, want)
		}
		if got, want := harness.HTML(), "<main>one</main>"; got != want {
			t.Fatalf("HTML() = %q, want %q", got, want)
		}
		if got := harness.Mocks().Storage().Local(); len(got) != 0 {
			t.Fatalf("Storage().Local() = %#v, want empty", got)
		}
		if got := harness.Mocks().Storage().Session(); len(got) != 0 {
			t.Fatalf("Storage().Session() = %#v, want empty", got)
		}
	})

	t.Run("FromHTMLWithURL", func(t *testing.T) {
		harness, err := FromHTMLWithURL("https://example.test/from-url", "<main>two</main>")
		if err != nil {
			t.Fatalf("FromHTMLWithURL() error = %v", err)
		}
		if got, want := harness.URL(), "https://example.test/from-url"; got != want {
			t.Fatalf("URL() = %q, want %q", got, want)
		}
		if got, want := harness.HTML(), "<main>two</main>"; got != want {
			t.Fatalf("HTML() = %q, want %q", got, want)
		}
	})

	t.Run("FromHTMLWithLocalStorage", func(t *testing.T) {
		entries := map[string]string{"token": "abc"}
		harness, err := FromHTMLWithLocalStorage("<main>three</main>", entries)
		if err != nil {
			t.Fatalf("FromHTMLWithLocalStorage() error = %v", err)
		}
		entries["token"] = "mutated"
		if got, want := harness.Mocks().Storage().Local()["token"], "abc"; got != want {
			t.Fatalf("Storage().Local()[\"token\"] = %q, want %q", got, want)
		}
		if got := harness.Mocks().Storage().Session(); len(got) != 0 {
			t.Fatalf("Storage().Session() = %#v, want empty", got)
		}
	})

	t.Run("FromHTMLWithURLAndLocalStorage", func(t *testing.T) {
		entries := map[string]string{"token": "xyz"}
		harness, err := FromHTMLWithURLAndLocalStorage(
			"https://example.test/local",
			"<main>four</main>",
			entries,
		)
		if err != nil {
			t.Fatalf("FromHTMLWithURLAndLocalStorage() error = %v", err)
		}
		entries["token"] = "mutated"
		if got, want := harness.URL(), "https://example.test/local"; got != want {
			t.Fatalf("URL() = %q, want %q", got, want)
		}
		if got, want := harness.Mocks().Storage().Local()["token"], "xyz"; got != want {
			t.Fatalf("Storage().Local()[\"token\"] = %q, want %q", got, want)
		}
	})

	t.Run("FromHTMLWithSessionStorage", func(t *testing.T) {
		entries := map[string]string{"tab": "main"}
		harness, err := FromHTMLWithSessionStorage("<main>five</main>", entries)
		if err != nil {
			t.Fatalf("FromHTMLWithSessionStorage() error = %v", err)
		}
		entries["tab"] = "mutated"
		if got, want := harness.Mocks().Storage().Session()["tab"], "main"; got != want {
			t.Fatalf("Storage().Session()[\"tab\"] = %q, want %q", got, want)
		}
		if got := harness.Mocks().Storage().Local(); len(got) != 0 {
			t.Fatalf("Storage().Local() = %#v, want empty", got)
		}
	})

	t.Run("FromHTMLWithURLAndSessionStorage", func(t *testing.T) {
		entries := map[string]string{"tab": "detail"}
		harness, err := FromHTMLWithURLAndSessionStorage(
			"https://example.test/session",
			"<main>six</main>",
			entries,
		)
		if err != nil {
			t.Fatalf("FromHTMLWithURLAndSessionStorage() error = %v", err)
		}
		entries["tab"] = "mutated"
		if got, want := harness.URL(), "https://example.test/session"; got != want {
			t.Fatalf("URL() = %q, want %q", got, want)
		}
		if got, want := harness.Mocks().Storage().Session()["tab"], "detail"; got != want {
			t.Fatalf("Storage().Session()[\"tab\"] = %q, want %q", got, want)
		}
	})
}

func TestPromptCancelContract(t *testing.T) {
	harness, err := FromHTML("<main></main>")
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	harness.Mocks().Dialogs().QueuePromptCancel()

	got, submitted, err := harness.Prompt("Cancel?")
	if err != nil {
		t.Fatalf("Prompt() error = %v", err)
	}
	if got != "" || submitted {
		t.Fatalf("Prompt() = (%q, %v), want (\"\", false)", got, submitted)
	}
	if messages := harness.Mocks().Dialogs().PromptMessages(); len(messages) != 1 || messages[0] != "Cancel?" {
		t.Fatalf("PromptMessages() = %#v, want [\"Cancel?\"]", messages)
	}
}

func TestInlineScriptsCanDriveHistoryThroughPublicFacade(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/app",
		`<main><script>host:historyPushState("step-1", "", "/step-1"); host:historyReplaceState("step-2", "", "step-2"); host:historyBack(); host:historyForward(); host:historyGo(-1)</script></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got := harness.Debug().DumpDOM(); got == "" {
		t.Fatalf("Debug().DumpDOM() after history script = empty string, want DOM snapshot")
	}
	if got, want := harness.URL(), "https://example.test/app"; got != want {
		t.Fatalf("URL() after history script = %q, want %q", got, want)
	}
	if got, want := harness.Mocks().Location().CurrentURL(), "https://example.test/app"; got != want {
		t.Fatalf("Location().CurrentURL() after history script = %q, want %q", got, want)
	}
	if got := harness.Mocks().Location().Navigations(); len(got) != 5 || got[0] != "https://example.test/step-1" || got[1] != "https://example.test/step-2" || got[2] != "https://example.test/app" || got[3] != "https://example.test/step-2" || got[4] != "https://example.test/app" {
		t.Fatalf("Location().Navigations() after history script = %#v, want ordered history navigations", got)
	}
}

func TestStorageViewReturnsCopies(t *testing.T) {
	harness, err := NewHarnessBuilder().
		URL("https://example.test/").
		HTML("<main></main>").
		LocalStorage(map[string]string{"local": "value"}).
		SessionStorage(map[string]string{"session": "value"}).
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	local := harness.Mocks().Storage().Local()
	local["local"] = "mutated"
	local["extra"] = "added"

	session := harness.Mocks().Storage().Session()
	session["session"] = "mutated"

	freshLocal := harness.Mocks().Storage().Local()
	if got, want := freshLocal["local"], "value"; got != want {
		t.Fatalf("Storage().Local()[\"local\"] = %q, want %q", got, want)
	}
	if _, ok := freshLocal["extra"]; ok {
		t.Fatalf("Storage().Local()[\"extra\"] should not exist")
	}

	freshSession := harness.Mocks().Storage().Session()
	if got, want := freshSession["session"], "value"; got != want {
		t.Fatalf("Storage().Session()[\"session\"] = %q, want %q", got, want)
	}
}

func TestInteractionSliceReportsFocusAndLog(t *testing.T) {
	harness, err := FromHTML(`<main><button id="cta">Go</button><input id="name"></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Focus(" #name "); err != nil {
		t.Fatalf("Focus(#name) error = %v", err)
	}
	if got, want := harness.Debug().FocusedSelector(), "#name"; got != want {
		t.Fatalf("Debug().FocusedSelector() after Focus = %q, want %q", got, want)
	}
	if err := harness.AssertExists("input:focus"); err != nil {
		t.Fatalf("AssertExists(input:focus) after Focus error = %v", err)
	}
	if err := harness.AssertExists("input:focus-visible"); err != nil {
		t.Fatalf("AssertExists(input:focus-visible) after Focus error = %v", err)
	}
	if err := harness.AssertExists("main:focus-within"); err != nil {
		t.Fatalf("AssertExists(main:focus-within) after Focus error = %v", err)
	}

	if err := harness.Click("#cta"); err != nil {
		t.Fatalf("Click(#cta) error = %v", err)
	}
	if err := harness.Blur(); err != nil {
		t.Fatalf("Blur() error = %v", err)
	}
	if got := harness.Debug().FocusedSelector(); got != "" {
		t.Fatalf("Debug().FocusedSelector() after Blur = %q, want empty", got)
	}
	if err := harness.AssertExists("input:focus"); err == nil {
		t.Fatalf("AssertExists(input:focus) after Blur error = nil, want no match")
	}
	if err := harness.AssertExists("input:focus-visible"); err == nil {
		t.Fatalf("AssertExists(input:focus-visible) after Blur error = nil, want no match")
	}

	log := harness.Debug().Interactions()
	if len(log) != 3 {
		t.Fatalf("Debug().Interactions() len = %d, want 3", len(log))
	}
	if log[0].Kind != InteractionKindFocus || log[0].Selector != "#name" {
		t.Fatalf("Debug().Interactions()[0] = %#v, want focus #name", log[0])
	}
	if log[1].Kind != InteractionKindClick || log[1].Selector != "#cta" {
		t.Fatalf("Debug().Interactions()[1] = %#v, want click #cta", log[1])
	}
	if log[2].Kind != InteractionKindBlur || log[2].Selector != "#name" {
		t.Fatalf("Debug().Interactions()[2] = %#v, want blur #name", log[2])
	}

	log[0].Selector = "mutated"
	if fresh := harness.Debug().Interactions(); fresh[0].Selector != "#name" {
		t.Fatalf("Debug().Interactions() should return copies, got %#v", fresh[0])
	}
}

func TestInteractionSliceRejectsMissingTargets(t *testing.T) {
	harness, err := FromHTML(`<main><button id="cta">Go</button></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Click("main[item="); err == nil {
		t.Fatalf("Click(main[item=) error = nil, want selector syntax error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindEvent {
		t.Fatalf("Click(main[item=) error = %#v, want event error", err)
	}

	if err := harness.Focus("#missing"); err == nil {
		t.Fatalf("Focus(#missing) error = nil, want missing-element error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindEvent {
		t.Fatalf("Focus(#missing) error = %#v, want event error", err)
	}

	if err := harness.Blur(); err != nil {
		t.Fatalf("Blur() error = %v", err)
	}

	if got := harness.Debug().Interactions(); len(got) != 1 || got[0].Kind != InteractionKindBlur {
		t.Fatalf("Debug().Interactions() = %#v, want one blur event after rejected interactions", got)
	}
}

func TestInteractionSliceSupportsBoundedCombinators(t *testing.T) {
	harness, err := FromHTML(`<main><section><button id="cta">Go</button></section><input id="name"></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Click("main section > button"); err != nil {
		t.Fatalf("Click(main section > button) error = %v", err)
	}
	if err := harness.Focus("section + input"); err != nil {
		t.Fatalf("Focus(section + input) error = %v", err)
	}
	if err := harness.AssertExists("section + input"); err != nil {
		t.Fatalf("AssertExists(section + input) error = %v", err)
	}

	if got, want := harness.Debug().FocusedSelector(), "section + input"; got != want {
		t.Fatalf("Debug().FocusedSelector() = %q, want %q", got, want)
	}

	log := harness.Debug().Interactions()
	if len(log) != 2 {
		t.Fatalf("Debug().Interactions() len = %d, want 2", len(log))
	}
	if log[0].Kind != InteractionKindClick || log[0].Selector != "main section > button" {
		t.Fatalf("Debug().Interactions()[0] = %#v, want click main section > button", log[0])
	}
	if log[1].Kind != InteractionKindFocus || log[1].Selector != "section + input" {
		t.Fatalf("Debug().Interactions()[1] = %#v, want focus section + input", log[1])
	}
}

func TestInteractionSliceSupportsBoundedPseudoClasses(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><input id="enabled" type="text"><input id="flag" type="checkbox" checked><input id="off" type="text" disabled><div id="empty"></div><p id="last">two</p></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists(":root"); err != nil {
		t.Fatalf("AssertExists(:root) error = %v", err)
	}
	if err := harness.AssertExists("input:checked"); err != nil {
		t.Fatalf("AssertExists(input:checked) error = %v", err)
	}
	if err := harness.AssertExists("input:default"); err != nil {
		t.Fatalf("AssertExists(input:default) error = %v", err)
	}
	if err := harness.AssertExists("input:disabled"); err != nil {
		t.Fatalf("AssertExists(input:disabled) error = %v", err)
	}
	if err := harness.AssertExists("div:empty"); err != nil {
		t.Fatalf("AssertExists(div:empty) error = %v", err)
	}
	if err := harness.Focus("input:first-child"); err != nil {
		t.Fatalf("Focus(input:first-child) error = %v", err)
	}
	if got, want := harness.Debug().FocusedSelector(), "input:first-child"; got != want {
		t.Fatalf("Debug().FocusedSelector() = %q, want %q", got, want)
	}
	if err := harness.AssertExists("p:last-child"); err != nil {
		t.Fatalf("AssertExists(p:last-child) error = %v", err)
	}
}

func TestInteractionSliceSupportsIndeterminatePseudoClass(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><input id="mixed" type="checkbox" indeterminate><input id="radio-a" type="radio" name="size"><input id="radio-b" type="radio" name="size"><progress id="task"></progress><progress id="done" value="42"></progress></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("input:indeterminate"); err != nil {
		t.Fatalf("AssertExists(input:indeterminate) error = %v", err)
	}
	if err := harness.AssertExists("progress:indeterminate"); err != nil {
		t.Fatalf("AssertExists(progress:indeterminate) error = %v", err)
	}
	if err := harness.AssertExists("#radio-a:indeterminate"); err != nil {
		t.Fatalf("AssertExists(#radio-a:indeterminate) error = %v", err)
	}
	if err := harness.AssertExists("#radio-b:indeterminate"); err != nil {
		t.Fatalf("AssertExists(#radio-b:indeterminate) error = %v", err)
	}

	if err := harness.SetChecked("#radio-a", true); err != nil {
		t.Fatalf("SetChecked(#radio-a) error = %v", err)
	}
	if err := harness.AssertExists("#radio-a:indeterminate"); err == nil {
		t.Fatalf("AssertExists(#radio-a:indeterminate) after SetChecked error = nil, want no match")
	}
	if err := harness.AssertExists("#radio-b:indeterminate"); err == nil {
		t.Fatalf("AssertExists(#radio-b:indeterminate) after SetChecked error = nil, want no match")
	}
}

func TestInteractionSliceSupportsLinkAndPlaceholderPseudoClasses(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/base/",
		`<main><a id="nav" href="/next">Go</a><map><area id="area" href="/popup" alt="Open"></map><form id="profile"><button id="submit-1" type="submit">Save</button><input id="placeholder" type="text" placeholder="Name"><textarea id="story" placeholder="Story"></textarea></form></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if err := harness.AssertExists("a:link"); err != nil {
		t.Fatalf("AssertExists(a:link) error = %v", err)
	}
	if err := harness.AssertExists("area:link"); err != nil {
		t.Fatalf("AssertExists(area:link) error = %v", err)
	}
	if err := harness.AssertExists("a:any-link"); err != nil {
		t.Fatalf("AssertExists(a:any-link) error = %v", err)
	}
	if err := harness.AssertExists("area:any-link"); err != nil {
		t.Fatalf("AssertExists(area:any-link) error = %v", err)
	}
	if err := harness.AssertExists("input:placeholder-shown"); err != nil {
		t.Fatalf("AssertExists(input:placeholder-shown) error = %v", err)
	}
	if err := harness.AssertExists("textarea:placeholder-shown"); err != nil {
		t.Fatalf("AssertExists(textarea:placeholder-shown) error = %v", err)
	}
	if err := harness.AssertExists("input:blank"); err != nil {
		t.Fatalf("AssertExists(input:blank) error = %v", err)
	}
	if err := harness.AssertExists("textarea:blank"); err != nil {
		t.Fatalf("AssertExists(textarea:blank) error = %v", err)
	}
	if err := harness.AssertExists("button:default"); err != nil {
		t.Fatalf("AssertExists(button:default) error = %v", err)
	}
}

func TestInteractionSliceSupportsLocalLinkPseudoClass(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/page#top",
		`<main><a id="self" href="#top">Self</a><a id="next" href="/next">Next</a><map><area id="area-self" href="#top" alt="Self"></map></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if err := harness.AssertExists("a:local-link"); err != nil {
		t.Fatalf("AssertExists(a:local-link) error = %v", err)
	}
	if err := harness.AssertExists("area:local-link"); err != nil {
		t.Fatalf("AssertExists(area:local-link) error = %v", err)
	}
	if err := harness.AssertExists("#self:local-link"); err != nil {
		t.Fatalf("AssertExists(#self:local-link) error = %v", err)
	}
	if err := harness.AssertExists("#next:local-link"); err == nil {
		t.Fatalf("AssertExists(#next:local-link) error = nil, want no match")
	}
}

func TestInteractionSliceSupportsVisitedPseudoClass(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/page",
		`<main><a id="nav" href="https://example.test/visited">Go</a><a id="other" href="https://example.test/other">Other</a><map><area id="area" href="https://example.test/visited" alt="Visited"></map></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if err := harness.Navigate("https://example.test/visited"); err != nil {
		t.Fatalf("Navigate() error = %v", err)
	}

	if err := harness.AssertExists("a:visited"); err != nil {
		t.Fatalf("AssertExists(a:visited) error = %v", err)
	}
	if err := harness.AssertExists("area:visited"); err != nil {
		t.Fatalf("AssertExists(area:visited) error = %v", err)
	}
	if err := harness.AssertExists("#nav:visited"); err != nil {
		t.Fatalf("AssertExists(#nav:visited) error = %v", err)
	}
	if err := harness.AssertExists("#other:visited"); err == nil {
		t.Fatalf("AssertExists(#other:visited) error = nil, want no match")
	}
}

func TestInteractionSliceSupportsAttributeSelectors(t *testing.T) {
	harness, err := FromHTML(`<main><div id="panel" data-kind="panel"><a id="nav" href="/next" data-role="nav">Go</a><input id="name" type="text"><p id="flag" hidden></p></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("div[data-kind]"); err != nil {
		t.Fatalf("AssertExists(div[data-kind]) error = %v", err)
	}
	if err := harness.AssertExists("a[href]"); err != nil {
		t.Fatalf("AssertExists(a[href]) error = %v", err)
	}
	if err := harness.AssertExists("a[href=\"/next\"]"); err != nil {
		t.Fatalf("AssertExists(a[href=\"/next\"]) error = %v", err)
	}
	if err := harness.AssertExists("a[data-role=nav]"); err != nil {
		t.Fatalf("AssertExists(a[data-role=nav]) error = %v", err)
	}
	if err := harness.AssertExists("input[type=text]"); err != nil {
		t.Fatalf("AssertExists(input[type=text]) error = %v", err)
	}
	if err := harness.AssertExists("p[hidden]"); err != nil {
		t.Fatalf("AssertExists(p[hidden]) error = %v", err)
	}
	if err := harness.AssertExists("a[data-role=missing]"); err == nil {
		t.Fatalf("AssertExists(a[data-role=missing]) error = nil, want no match")
	}
}

func TestInteractionSliceSupportsMoreBoundedPseudoClasses(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><h1 id="title">Title</h1><details id="details" open><summary>Sum</summary></details><dialog id="dialog" open></dialog><form id="profile"><input id="required" type="text" required><input id="optional" type="text"><input id="readonly" type="text" readonly><textarea id="editable"></textarea><textarea id="readonly-ta" readonly>Locked</textarea></form></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("input:required"); err != nil {
		t.Fatalf("AssertExists(input:required) error = %v", err)
	}
	if err := harness.AssertExists("input:optional"); err != nil {
		t.Fatalf("AssertExists(input:optional) error = %v", err)
	}
	if err := harness.AssertExists("input:read-write"); err != nil {
		t.Fatalf("AssertExists(input:read-write) error = %v", err)
	}
	if err := harness.AssertExists("input:read-only"); err != nil {
		t.Fatalf("AssertExists(input:read-only) error = %v", err)
	}
	if err := harness.AssertExists("textarea:read-write"); err != nil {
		t.Fatalf("AssertExists(textarea:read-write) error = %v", err)
	}
	if err := harness.AssertExists("textarea:read-only"); err != nil {
		t.Fatalf("AssertExists(textarea:read-only) error = %v", err)
	}
	if err := harness.AssertExists("h1:heading"); err != nil {
		t.Fatalf("AssertExists(h1:heading) error = %v", err)
	}
	if err := harness.AssertExists("details:open"); err != nil {
		t.Fatalf("AssertExists(details:open) error = %v", err)
	}
	if err := harness.AssertExists("dialog:open"); err != nil {
		t.Fatalf("AssertExists(dialog:open) error = %v", err)
	}
}

func TestInteractionSliceSupportsModalPseudoClass(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><dialog id="dialog" modal></dialog><video id="player" fullscreen></video><div id="other" open></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("dialog:modal"); err != nil {
		t.Fatalf("AssertExists(dialog:modal) error = %v", err)
	}
	if err := harness.AssertExists("video:modal"); err != nil {
		t.Fatalf("AssertExists(video:modal) error = %v", err)
	}
	if err := harness.AssertExists("#other:modal"); err == nil {
		t.Fatalf("AssertExists(#other:modal) error = nil, want no match")
	}
}

func TestInteractionSliceSupportsPopoverOpenPseudoClass(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><div id="menu" popover popover-open></div><div id="closed" popover></div><dialog id="dialog" open></dialog></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("div:popover-open"); err != nil {
		t.Fatalf("AssertExists(div:popover-open) error = %v", err)
	}
	if err := harness.AssertExists("#menu:popover-open"); err != nil {
		t.Fatalf("AssertExists(#menu:popover-open) error = %v", err)
	}
	if err := harness.AssertExists("#closed:popover-open"); err == nil {
		t.Fatalf("AssertExists(#closed:popover-open) error = nil, want no match")
	}
}

func TestInteractionSliceSupportsDefinedPseudoClass(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><div id="known"></div><x-widget id="widget" defined></x-widget><x-ghost id="ghost"></x-ghost></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("div:defined"); err != nil {
		t.Fatalf("AssertExists(div:defined) error = %v", err)
	}
	if err := harness.AssertExists("x-widget:defined"); err != nil {
		t.Fatalf("AssertExists(x-widget:defined) error = %v", err)
	}
	if err := harness.AssertExists("#ghost:defined"); err == nil {
		t.Fatalf("AssertExists(#ghost:defined) error = nil, want no match")
	}
}

func TestInteractionSliceSupportsStatePseudoClass(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><x-widget id="widget"></x-widget><div id="plain" state="checked"></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.SetAttribute("#widget", "state", "checked pressed"); err != nil {
		t.Fatalf("SetAttribute(#widget, state, checked pressed) error = %v", err)
	}

	if err := harness.AssertExists("#widget:state(checked)"); err != nil {
		t.Fatalf("AssertExists(#widget:state(checked)) error = %v", err)
	}
	if err := harness.AssertExists("#widget:state(checked):state(pressed)"); err != nil {
		t.Fatalf("AssertExists(#widget:state(checked):state(pressed)) error = %v", err)
	}
	if err := harness.AssertExists("div:state(checked)"); err == nil {
		t.Fatalf("AssertExists(div:state(checked)) error = nil, want no match")
	}

	if err := harness.RemoveAttribute("#widget", "state"); err != nil {
		t.Fatalf("RemoveAttribute(#widget, state) error = %v", err)
	}
	if err := harness.AssertExists("#widget:state(checked)"); err == nil {
		t.Fatalf("AssertExists(#widget:state(checked)) after RemoveAttribute error = nil, want no match")
	}
}

func TestInteractionSliceSupportsAutofillPseudoClass(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><input id="name" autofill value="Ada"><input id="other" value="Bob"></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("input:autofill"); err != nil {
		t.Fatalf("AssertExists(input:autofill) error = %v", err)
	}
	if err := harness.AssertExists("input:-webkit-autofill"); err != nil {
		t.Fatalf("AssertExists(input:-webkit-autofill) error = %v", err)
	}

	if err := harness.TypeText("#name", "Zed"); err != nil {
		t.Fatalf("TypeText(#name) error = %v", err)
	}
	if err := harness.AssertValue("#name", "Zed"); err != nil {
		t.Fatalf("AssertValue(#name) after TypeText error = %v", err)
	}
	if err := harness.AssertExists("#name:autofill"); err == nil {
		t.Fatalf("AssertExists(#name:autofill) after TypeText error = nil, want no match")
	}
}

func TestInteractionSliceSupportsActiveHoverPseudoClasses(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><div id="wrap"><button id="btn" active>Go</button><span id="hovered" hover>Hover</span></div><p id="plain">Text</p></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("button:active"); err != nil {
		t.Fatalf("AssertExists(button:active) error = %v", err)
	}
	if err := harness.AssertExists("div:active"); err != nil {
		t.Fatalf("AssertExists(div:active) error = %v", err)
	}
	if err := harness.AssertExists("span:hover"); err != nil {
		t.Fatalf("AssertExists(span:hover) error = %v", err)
	}
	if err := harness.AssertExists("div:hover"); err != nil {
		t.Fatalf("AssertExists(div:hover) error = %v", err)
	}
	if err := harness.AssertExists("#plain:active"); err == nil {
		t.Fatalf("AssertExists(#plain:active) error = nil, want no match")
	}
}

func TestInteractionSliceSupportsHeadingLevelPseudoClass(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><h1 id="title">Title</h1><section><h2 id="sub">Sub</h2><div><h4 id="deep">Deep</h4></div></section><article><h6 id="final">Final</h6></article><p id="plain">Body</p></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists(":heading(1)"); err != nil {
		t.Fatalf("AssertExists(:heading(1)) error = %v", err)
	}
	if err := harness.AssertExists(":heading(2, 4)"); err != nil {
		t.Fatalf("AssertExists(:heading(2, 4)) error = %v", err)
	}
	if err := harness.AssertExists("h4:heading(4)"); err != nil {
		t.Fatalf("AssertExists(h4:heading(4)) error = %v", err)
	}
	if err := harness.AssertExists("h6:heading(6)"); err != nil {
		t.Fatalf("AssertExists(h6:heading(6)) error = %v", err)
	}
}

func TestInteractionSliceSupportsMediaPseudoClasses(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><audio id="song" src="song.mp3"></audio><video id="film"></video><video id="paused" paused></video><video id="seeking" seeking></video><video id="muted" muted></video><video id="buffering" networkstate="loading" readystate="2"></video><video id="stalled" networkstate="loading" readystate="1" stalled volume-locked></video><div id="other" paused muted></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("audio:playing"); err != nil {
		t.Fatalf("AssertExists(audio:playing) error = %v", err)
	}
	if err := harness.AssertExists("video:paused"); err != nil {
		t.Fatalf("AssertExists(video:paused) error = %v", err)
	}
	if err := harness.AssertExists("video:seeking"); err != nil {
		t.Fatalf("AssertExists(video:seeking) error = %v", err)
	}
	if err := harness.AssertExists("video:muted"); err != nil {
		t.Fatalf("AssertExists(video:muted) error = %v", err)
	}
	if err := harness.AssertExists("video:buffering"); err != nil {
		t.Fatalf("AssertExists(video:buffering) error = %v", err)
	}
	if err := harness.AssertExists("video:stalled"); err != nil {
		t.Fatalf("AssertExists(video:stalled) error = %v", err)
	}
	if err := harness.AssertExists("video:volume-locked"); err != nil {
		t.Fatalf("AssertExists(video:volume-locked) error = %v", err)
	}
	if err := harness.AssertExists("#other:paused"); err == nil {
		t.Fatalf("AssertExists(#other:paused) error = nil, want no match")
	}
}

func TestInteractionSliceSupportsOfTypePseudoClasses(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><section id="single"><em id="only-child">one</em></section><div id="mixed"><p id="para-a">A</p><span id="only-of-type">S</span><p id="para-b">B</p></div><details id="details" open><summary id="summary-a">A</summary><div id="middle">M</div><summary id="summary-b">B</summary></details></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("em:only-child"); err != nil {
		t.Fatalf("AssertExists(em:only-child) error = %v", err)
	}
	if err := harness.AssertExists("em:only-of-type"); err != nil {
		t.Fatalf("AssertExists(em:only-of-type) error = %v", err)
	}
	if err := harness.AssertExists("span:only-of-type"); err != nil {
		t.Fatalf("AssertExists(span:only-of-type) error = %v", err)
	}
	if err := harness.AssertExists("summary:first-of-type"); err != nil {
		t.Fatalf("AssertExists(summary:first-of-type) error = %v", err)
	}
	if err := harness.AssertExists("summary:last-of-type"); err != nil {
		t.Fatalf("AssertExists(summary:last-of-type) error = %v", err)
	}
}

func TestInteractionSliceSupportsConstraintValidationPseudoClasses(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><form id="valid-form"><input id="name" type="text" required value="Ada"><input id="age" type="number" min="1" max="10" value="5"><select id="mode"><option value="a" selected>A</option><option value="b">B</option></select></form><form id="invalid-form"><input id="missing" type="text" required><input id="low" type="number" min="1" max="10" value="0"><input id="high" type="number" min="1" max="10" value="11"></form></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("input:valid"); err != nil {
		t.Fatalf("AssertExists(input:valid) error = %v", err)
	}
	if err := harness.AssertExists("input:invalid"); err != nil {
		t.Fatalf("AssertExists(input:invalid) error = %v", err)
	}
	if err := harness.AssertExists("input:in-range"); err != nil {
		t.Fatalf("AssertExists(input:in-range) error = %v", err)
	}
	if err := harness.AssertExists("input:out-of-range"); err != nil {
		t.Fatalf("AssertExists(input:out-of-range) error = %v", err)
	}
	if err := harness.AssertExists("select:valid"); err != nil {
		t.Fatalf("AssertExists(select:valid) error = %v", err)
	}
	if err := harness.AssertExists("form:valid"); err != nil {
		t.Fatalf("AssertExists(form:valid) error = %v", err)
	}
	if err := harness.AssertExists("form:invalid"); err != nil {
		t.Fatalf("AssertExists(form:invalid) error = %v", err)
	}
}

func TestAttributeReflectionContracts(t *testing.T) {
	harness, err := FromHTML(`<main><div id="root" data-x="1" class="alpha beta"></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, ok, err := harness.GetAttribute(" #root ", "DATA-X"); err != nil || !ok || got != "1" {
		t.Fatalf("GetAttribute(DATA-X) = (%q, %v, %v), want (\"1\", true, nil)", got, ok, err)
	}
	if ok, err := harness.HasAttribute("#root", "data-x"); err != nil || !ok {
		t.Fatalf("HasAttribute(data-x) = (%v, %v), want (true, nil)", ok, err)
	}

	if err := harness.SetAttribute("#root", "data-x", "2"); err != nil {
		t.Fatalf("SetAttribute(data-x) error = %v", err)
	}
	if got, ok, err := harness.GetAttribute("#root", "data-x"); err != nil || !ok || got != "2" {
		t.Fatalf("GetAttribute(data-x) after SetAttribute = (%q, %v, %v), want (\"2\", true, nil)", got, ok, err)
	}
	if got, want := harness.Debug().DumpDOM(), `<main><div id="root" data-x="2" class="alpha beta"></div></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after SetAttribute = %q, want %q", got, want)
	}

	if err := harness.RemoveAttribute("#root", "data-x"); err != nil {
		t.Fatalf("RemoveAttribute(data-x) error = %v", err)
	}
	if got, ok, err := harness.GetAttribute("#root", "data-x"); err != nil || ok || got != "" {
		t.Fatalf("GetAttribute(data-x) after RemoveAttribute = (%q, %v, %v), want (\"\", false, nil)", got, ok, err)
	}
	if ok, err := harness.HasAttribute("#root", "data-x"); err != nil || ok {
		t.Fatalf("HasAttribute(data-x) after RemoveAttribute = (%v, %v), want (false, nil)", ok, err)
	}
}

func TestClassListAndDatasetContracts(t *testing.T) {
	harness, err := FromHTML(`<main><div id="root" class="alpha beta" data-foo-bar="1"></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	classList, err := harness.ClassList("#root")
	if err != nil {
		t.Fatalf("ClassList(#root) error = %v", err)
	}
	if got := classList.Values(); len(got) != 2 || got[0] != "alpha" || got[1] != "beta" {
		t.Fatalf("ClassList.Values() = %#v, want [alpha beta]", got)
	}
	if !classList.Contains("beta") {
		t.Fatalf("ClassList.Contains(beta) = false, want true")
	}

	classes := classList.Values()
	classes[0] = "mutated"
	if got := classList.Values(); got[0] != "alpha" {
		t.Fatalf("ClassList.Values() should return copies, got %#v", got)
	}

	if err := classList.Add("gamma"); err != nil {
		t.Fatalf("ClassList.Add(gamma) error = %v", err)
	}
	if err := classList.Remove("alpha"); err != nil {
		t.Fatalf("ClassList.Remove(alpha) error = %v", err)
	}

	dataset, err := harness.Dataset("#root")
	if err != nil {
		t.Fatalf("Dataset(#root) error = %v", err)
	}
	if got := dataset.Values(); got["fooBar"] != "1" || len(got) != 1 {
		t.Fatalf("Dataset.Values() = %#v, want fooBar=1", got)
	}
	if got, ok := dataset.Get("fooBar"); !ok || got != "1" {
		t.Fatalf("Dataset.Get(fooBar) = (%q, %v), want (\"1\", true)", got, ok)
	}

	values := dataset.Values()
	values["fooBar"] = "mutated"
	if got, ok := dataset.Get("fooBar"); !ok || got != "1" {
		t.Fatalf("Dataset.Get(fooBar) after Values mutation = (%q, %v), want (\"1\", true)", got, ok)
	}

	if err := dataset.Set("shipId", "92432"); err != nil {
		t.Fatalf("Dataset.Set(shipId) error = %v", err)
	}
	if err := dataset.Remove("fooBar"); err != nil {
		t.Fatalf("Dataset.Remove(fooBar) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="root" class="beta gamma" data-ship-id="92432"></div></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after class/dataset view mutation = %q, want %q", got, want)
	}
}

func TestClassListAndDatasetContractsRejectInvalidInputs(t *testing.T) {
	harness, err := FromHTML(`<main><div id="root" class="alpha" data-foo="1"></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if _, err := harness.ClassList("main[item="); err == nil {
		t.Fatalf("ClassList(main[item=) error = nil, want selector error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("ClassList(main[item=) error = %#v, want DOM error", err)
	}
	if _, err := harness.Dataset("#missing"); err == nil {
		t.Fatalf("Dataset(#missing) error = nil, want missing-element error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("Dataset(#missing) error = %#v, want DOM error", err)
	}

	classList, err := harness.ClassList("#root")
	if err != nil {
		t.Fatalf("ClassList(#root) error = %v", err)
	}
	if err := classList.Add(" "); err == nil {
		t.Fatalf("ClassList.Add(empty) error = nil, want validation error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("ClassList.Add(empty) error = %#v, want DOM error", err)
	}

	dataset, err := harness.Dataset("#root")
	if err != nil {
		t.Fatalf("Dataset(#root) error = %v", err)
	}
	if err := dataset.Set("foo-bar", "x"); err == nil {
		t.Fatalf("Dataset.Set(foo-bar) error = nil, want validation error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("Dataset.Set(foo-bar) error = %#v, want DOM error", err)
	}

	var nilHarness *Harness
	if _, err := nilHarness.ClassList("#root"); err == nil {
		t.Fatalf("nil Harness.ClassList() error = nil, want DOM error")
	}
	if _, err := nilHarness.Dataset("#root"); err == nil {
		t.Fatalf("nil Harness.Dataset() error = nil, want DOM error")
	}

	var emptyClassList ClassListView
	if got := emptyClassList.Values(); len(got) != 0 {
		t.Fatalf("zero ClassListView.Values() = %#v, want empty", got)
	}
	if emptyClassList.Contains("alpha") {
		t.Fatalf("zero ClassListView.Contains(alpha) = true, want false")
	}
	if err := emptyClassList.Add("alpha"); err == nil {
		t.Fatalf("zero ClassListView.Add() error = nil, want DOM error")
	}

	var emptyDataset DatasetView
	if got := emptyDataset.Values(); len(got) != 0 {
		t.Fatalf("zero DatasetView.Values() = %#v, want empty", got)
	}
	if got, ok := emptyDataset.Get("fooBar"); ok || got != "" {
		t.Fatalf("zero DatasetView.Get(fooBar) = (%q, %v), want (\"\", false)", got, ok)
	}
	if err := emptyDataset.Set("shipId", "1"); err == nil {
		t.Fatalf("zero DatasetView.Set() error = nil, want DOM error")
	}
}

func TestAttributeReflectionContractsRejectInvalidInputs(t *testing.T) {
	harness, err := FromHTML(`<main><div id="root"></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if _, _, err := harness.GetAttribute("#missing", "id"); err == nil {
		t.Fatalf("GetAttribute(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("GetAttribute(#missing) error = %#v, want DOM error", err)
	}

	if err := harness.SetAttribute("#root", " ", "x"); err == nil {
		t.Fatalf("SetAttribute(empty name) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("SetAttribute(empty name) error = %#v, want DOM error", err)
	}

	if err := harness.RemoveAttribute("#root", " "); err == nil {
		t.Fatalf("RemoveAttribute(empty name) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("RemoveAttribute(empty name) error = %#v, want DOM error", err)
	}
}

func TestFormControlActionsUpdateDebugDom(t *testing.T) {
	harness, err := FromHTML(`<main><input id="name"><input id="flag" type="checkbox"><textarea id="bio">Base</textarea><select id="mode"><option value="a" selected>A</option><option>B</option><option value="c">C</option></select><form id="profile"><button id="submit" type="submit">Save</button></form></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.TypeText("#name", "Ada"); err != nil {
		t.Fatalf("TypeText(#name) error = %v", err)
	}
	if err := harness.SetChecked("#flag", true); err != nil {
		t.Fatalf("SetChecked(#flag) error = %v", err)
	}
	if err := harness.SetSelectValue("#mode", "B"); err != nil {
		t.Fatalf("SetSelectValue(#mode) error = %v", err)
	}
	if err := harness.Submit("#profile"); err != nil {
		t.Fatalf("Submit(#profile) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><input id="name" value="Ada"><input id="flag" type="checkbox" checked><textarea id="bio">Base</textarea><select id="mode"><option value="a">A</option><option selected>B</option><option value="c">C</option></select><form id="profile"><button id="submit" type="submit">Save</button></form></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() = %q, want %q", got, want)
	}
	if got, want := harness.HTML(), `<main><input id="name"><input id="flag" type="checkbox"><textarea id="bio">Base</textarea><select id="mode"><option value="a" selected>A</option><option>B</option><option value="c">C</option></select><form id="profile"><button id="submit" type="submit">Save</button></form></main>`; got != want {
		t.Fatalf("HTML() = %q, want original source snapshot %q", got, want)
	}

	log := harness.Debug().Interactions()
	if len(log) != 4 {
		t.Fatalf("Debug().Interactions() len = %d, want 4", len(log))
	}
	if log[0].Kind != InteractionKindTypeText || log[0].Selector != "#name" {
		t.Fatalf("Debug().Interactions()[0] = %#v, want type_text #name", log[0])
	}
	if log[1].Kind != InteractionKindSetChecked || log[1].Selector != "#flag" {
		t.Fatalf("Debug().Interactions()[1] = %#v, want set_checked #flag", log[1])
	}
	if log[2].Kind != InteractionKindSetSelectValue || log[2].Selector != "#mode" {
		t.Fatalf("Debug().Interactions()[2] = %#v, want set_select_value #mode", log[2])
	}
	if log[3].Kind != InteractionKindSubmit || log[3].Selector != "#profile" {
		t.Fatalf("Debug().Interactions()[3] = %#v, want submit #profile", log[3])
	}
}

func TestClickAppliesDefaultActions(t *testing.T) {
	harness, err := FromHTML(`<form id="profile"><input id="agree" type="checkbox"><button id="submit" type="submit">Save</button></form>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Click("#agree"); err != nil {
		t.Fatalf("Click(#agree) error = %v", err)
	}
	if err := harness.Click("#submit"); err != nil {
		t.Fatalf("Click(#submit) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<form id="profile"><input id="agree" type="checkbox" checked><button id="submit" type="submit">Save</button></form>`; got != want {
		t.Fatalf("Debug().DumpDOM() = %q, want %q", got, want)
	}

	log := harness.Debug().Interactions()
	if len(log) != 3 {
		t.Fatalf("Debug().Interactions() len = %d, want 3", len(log))
	}
	if log[0].Kind != InteractionKindClick || log[0].Selector != "#agree" {
		t.Fatalf("Debug().Interactions()[0] = %#v, want click #agree", log[0])
	}
	if log[1].Kind != InteractionKindClick || log[1].Selector != "#submit" {
		t.Fatalf("Debug().Interactions()[1] = %#v, want click #submit", log[1])
	}
	if log[2].Kind != InteractionKindSubmit || log[2].Selector != "#submit" {
		t.Fatalf("Debug().Interactions()[2] = %#v, want submit #submit", log[2])
	}
}

func TestSetFilesMarksFileInputAsUserValid(t *testing.T) {
	harness, err := FromHTML(`<main><input id="upload" type="file"></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.SetFiles("#upload", []string{"report.csv"}); err != nil {
		t.Fatalf("SetFiles(#upload) error = %v", err)
	}
	if err := harness.AssertValue("#upload", "report.csv"); err != nil {
		t.Fatalf("AssertValue(#upload) error = %v", err)
	}
	if err := harness.AssertExists("#upload:user-valid"); err != nil {
		t.Fatalf("AssertExists(#upload:user-valid) error = %v", err)
	}
}

func TestClickAppliesHyperlinkDefaultActions(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/base/",
		`<main><a id="nav" href="/next">Go</a><map name="hot"><area id="popup" href="https://example.test/popup" target="_blank" alt="Open"></map><a id="download" href="https://example.test/files/report.csv" download="report.csv">Download</a></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if err := harness.Click("#nav"); err != nil {
		t.Fatalf("Click(#nav) error = %v", err)
	}
	if got, want := harness.URL(), "https://example.test/next"; got != want {
		t.Fatalf("URL() after anchor click = %q, want %q", got, want)
	}
	if got := harness.Mocks().Location().Navigations(); len(got) != 1 || got[0] != "https://example.test/next" {
		t.Fatalf("Location().Navigations() = %#v, want one navigation to https://example.test/next", got)
	}

	if err := harness.Click("#popup"); err != nil {
		t.Fatalf("Click(#popup) error = %v", err)
	}
	if got, want := harness.URL(), "https://example.test/next"; got != want {
		t.Fatalf("URL() after target=_blank click = %q, want %q", got, want)
	}
	if got := harness.Mocks().Open().Calls(); len(got) != 1 || got[0].URL != "https://example.test/popup" {
		t.Fatalf("Open().Calls() = %#v, want one open call to popup", got)
	}

	if err := harness.Click("#download"); err != nil {
		t.Fatalf("Click(#download) error = %v", err)
	}
	if got, want := harness.URL(), "https://example.test/next"; got != want {
		t.Fatalf("URL() after download click = %q, want %q", got, want)
	}
	downloads := harness.Mocks().Downloads().Artifacts()
	if len(downloads) != 1 || downloads[0].FileName != "report.csv" || string(downloads[0].Bytes) != "https://example.test/files/report.csv" {
		t.Fatalf("Downloads().Artifacts() = %#v, want one captured download", downloads)
	}
}

func TestClickAppliesResetDefaultAction(t *testing.T) {
	harness, err := FromHTML(`<form id="profile"><input id="name"><input id="flag" type="checkbox"><input id="radio-a" type="radio" name="size" checked><input id="radio-b" type="radio" name="size"><textarea id="bio">Base</textarea><select id="mode"><option value="a" selected>A</option><option>B</option><option value="c">C</option></select><button id="reset" type="reset">Reset</button></form>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.TypeText("#name", "Ada"); err != nil {
		t.Fatalf("TypeText(#name) error = %v", err)
	}
	if err := harness.SetChecked("#flag", true); err != nil {
		t.Fatalf("SetChecked(#flag) error = %v", err)
	}
	if err := harness.SetChecked("#radio-b", true); err != nil {
		t.Fatalf("SetChecked(#radio-b) error = %v", err)
	}
	if err := harness.TypeText("#bio", "Line 1\nLine 2"); err != nil {
		t.Fatalf("TypeText(#bio) error = %v", err)
	}
	if err := harness.SetSelectValue("#mode", "B"); err != nil {
		t.Fatalf("SetSelectValue(#mode) error = %v", err)
	}

	if err := harness.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<form id="profile"><input id="name"><input id="flag" type="checkbox"><input id="radio-a" type="radio" name="size" checked><input id="radio-b" type="radio" name="size"><textarea id="bio">Base</textarea><select id="mode"><option value="a" selected>A</option><option>B</option><option value="c">C</option></select><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("Debug().DumpDOM() after reset click = %q, want %q", got, want)
	}

	log := harness.Debug().Interactions()
	if len(log) != 6 {
		t.Fatalf("Debug().Interactions() len = %d, want 6", len(log))
	}
	if log[5].Kind != InteractionKindClick || log[5].Selector != "#reset" {
		t.Fatalf("Debug().Interactions()[5] = %#v, want click #reset", log[5])
	}
}

func TestWriteHTMLContract(t *testing.T) {
	harness, err := FromHTML(`<main><button id="btn">old</button><div id="out">before</div><script>host:addEventListener("#btn", "click", 'host:setInnerHTML("#out", "old-listener")')</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Focus("#btn"); err != nil {
		t.Fatalf("Focus(#btn) error = %v", err)
	}
	if err := harness.ScrollTo(7, 9); err != nil {
		t.Fatalf("ScrollTo(7, 9) error = %v", err)
	}

	markup := `<main><button id="btn">new</button><div id="out">fresh</div><script>host:setInnerHTML("#out", "written")</script></main>`
	if err := harness.WriteHTML(markup); err != nil {
		t.Fatalf("WriteHTML() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><button id="btn">new</button><div id="out">written</div><script>host:setInnerHTML("#out", "written")</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after WriteHTML = %q, want %q", got, want)
	}
	if got, want := harness.HTML(), markup; got != want {
		t.Fatalf("HTML() after WriteHTML = %q, want %q", got, want)
	}
	if got := harness.Debug().FocusedSelector(); got != "" {
		t.Fatalf("Debug().FocusedSelector() after WriteHTML = %q, want empty", got)
	}
	if gotX, gotY := harness.Debug().ScrollPosition(); gotX != 0 || gotY != 0 {
		t.Fatalf("Debug().ScrollPosition() after WriteHTML = (%d, %d), want (0, 0)", gotX, gotY)
	}

	if err := harness.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) after WriteHTML error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<main><button id="btn">new</button><div id="out">written</div><script>host:setInnerHTML("#out", "written")</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after Click on rewritten document = %q, want %q", got, want)
	}
}

func TestInlineScriptsCanWriteHTMLThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out">old</div><script>host:writeHTML('<main><div id="out">new</div></main>'); host:setInnerHTML("#out", "after")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">after</div></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after host writeHTML = %q, want %q", got, want)
	}
	if got, want := harness.HTML(), `<main><div id="out">new</div></main>`; got != want {
		t.Fatalf("HTML() after host writeHTML = %q, want %q", got, want)
	}
}

func TestInlineScriptsCanDriveLocationMockThroughPublicActions(t *testing.T) {
	markup := `<main><div id="out"></div><script>host:locationAssign("/assign"); host:locationReplace("replace"); host:locationReload()</script></main>`
	harness, err := FromHTMLWithURL("https://example.test/start", markup)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), markup; got != want {
		t.Fatalf("Debug().DumpDOM() after location host bridge = %q, want %q", got, want)
	}
	if got, want := harness.URL(), "https://example.test/replace"; got != want {
		t.Fatalf("URL() after location host bridge = %q, want %q", got, want)
	}
	if got, want := harness.Mocks().Location().CurrentURL(), "https://example.test/replace"; got != want {
		t.Fatalf("Mocks().Location().CurrentURL() = %q, want %q", got, want)
	}
	if got, want := harness.Mocks().Location().Navigations(), []string{
		"https://example.test/assign",
		"https://example.test/replace",
		"https://example.test/replace",
	}; len(got) != len(want) {
		t.Fatalf("Mocks().Location().Navigations() = %#v, want %#v", got, want)
	} else {
		for i := range want {
			if got[i] != want[i] {
				t.Fatalf("Mocks().Location().Navigations()[%d] = %q, want %q", i, got[i], want[i])
			}
		}
	}
}

func TestInlineScriptsCanSetLocationPropertiesThroughPublicActions(t *testing.T) {
	markup := `<main><div id="out"></div><script>host:locationSet("hash", "#step1"); host:locationSet("pathname", "next"); host:locationSet("search", "?mode=full")</script></main>`
	harness, err := FromHTMLWithURL("https://example.test/start?old=1", markup)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), markup; got != want {
		t.Fatalf("Debug().DumpDOM() after locationSet host bridge = %q, want %q", got, want)
	}
	if got, want := harness.URL(), "https://example.test/next?mode=full#step1"; got != want {
		t.Fatalf("URL() after locationSet host bridge = %q, want %q", got, want)
	}
	if got, want := harness.Mocks().Location().Navigations(), []string{
		"https://example.test/start?old=1#step1",
		"https://example.test/next?old=1#step1",
		"https://example.test/next?mode=full#step1",
	}; len(got) != len(want) {
		t.Fatalf("Mocks().Location().Navigations() = %#v, want %#v", got, want)
	} else {
		for i := range want {
			if got[i] != want[i] {
				t.Fatalf("Mocks().Location().Navigations()[%d] = %q, want %q", i, got[i], want[i])
			}
		}
	}
}

func TestInlineScriptsCanDriveWindowNameThroughPublicActions(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/start",
		`<main><div id="out">old</div><script>host:setWindowName("alpha"); host:locationAssign("/next"); host:setInnerHTML("#out", "done")</script></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">done</div><script>host:setWindowName("alpha"); host:locationAssign("/next"); host:setInnerHTML("#out", "done")</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after window name host bridge = %q, want %q", got, want)
	}
	if got, want := harness.URL(), "https://example.test/next"; got != want {
		t.Fatalf("URL() after window name host bridge = %q, want %q", got, want)
	}
	if got, want := harness.Debug().WindowName(), "alpha"; got != want {
		t.Fatalf("Debug().WindowName() after window name host bridge = %q, want %q", got, want)
	}
}

func TestInlineScriptsCanExposeCurrentScriptThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out">old</div><script id="boot">host:setInnerHTML("#out", expr(host:documentCurrentScript()))</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="out"><script id="boot">host:setInnerHTML("#out", expr(host:documentCurrentScript()))</script></div><script id="boot">host:setInnerHTML("#out", expr(host:documentCurrentScript()))</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after documentCurrentScript bootstrap = %q, want %q", got, want)
	}
}

func TestInlineScriptsTreatEventHandlersAsNonScriptContextsForCurrentScript(t *testing.T) {
	harness, err := FromHTML(`<main><button id="btn">Go</button><div id="out">old</div><script>host:addEventListener("#btn", "click", 'host:setInnerHTML("#out", expr(host:documentCurrentScript()))')</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><button id="btn">Go</button><div id="out"></div><script>host:addEventListener("#btn", "click", 'host:setInnerHTML("#out", expr(host:documentCurrentScript()))')</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after event handler currentScript = %q, want %q", got, want)
	}
}

func TestNavigateResolvesRelativeURLs(t *testing.T) {
	harness, err := FromHTMLWithURL("https://example.test/start", "<main></main>")
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if err := harness.Navigate("next"); err != nil {
		t.Fatalf("Navigate(next) error = %v", err)
	}

	if got, want := harness.URL(), "https://example.test/next"; got != want {
		t.Fatalf("URL() after relative Navigate = %q, want %q", got, want)
	}
	if got, want := harness.Debug().URL(), "https://example.test/next"; got != want {
		t.Fatalf("Debug().URL() after relative Navigate = %q, want %q", got, want)
	}
}

func TestHasPseudoClassMatchesDescendantSubtrees(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><section id="wrap"><article id="a1"><span class="hit">Hit</span></article><article id="a2"><span class="miss">Miss</span></article></section><aside id="plain"><span class="hit">Outside</span></aside></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("section:has(.hit)"); err != nil {
		t.Fatalf("AssertExists(section:has(.hit)) error = %v", err)
	}
	if err := harness.AssertExists("section:has(article > .hit)"); err != nil {
		t.Fatalf("AssertExists(section:has(article > .hit)) error = %v", err)
	}
	if err := harness.AssertExists("article:has(.hit, .miss)"); err != nil {
		t.Fatalf("AssertExists(article:has(.hit, .miss)) error = %v", err)
	}
	if err := harness.AssertExists("section:has(.missing)"); err == nil {
		t.Fatalf("AssertExists(section:has(.missing)) error = nil, want no match")
	}
}

func TestNotPseudoClassFiltersCurrentNodes(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><section id="wrap"><article id="a1" class="match"><span class="hit">Hit</span></article><article id="a2"><span class="miss">Miss</span></article></section><aside id="plain"><span class="hit">Outside</span></aside></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("section:not(.missing)"); err != nil {
		t.Fatalf("AssertExists(section:not(.missing)) error = %v", err)
	}
	if err := harness.AssertExists("article:not(.match, .other)"); err != nil {
		t.Fatalf("AssertExists(article:not(.match, .other)) error = %v", err)
	}
	if err := harness.AssertExists("#a1:not(.match)"); err == nil {
		t.Fatalf("AssertExists(#a1:not(.match)) error = nil, want no match")
	}
}

func TestIsAndWherePseudoClassesMatchCurrentNodes(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><section id="wrap" class="match"><article id="a1" class="hit">One</article><article id="a2" class="miss">Two</article></section><aside id="plain"><span class="hit">Outside</span></aside></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("section:is(#wrap, .missing)"); err != nil {
		t.Fatalf("AssertExists(section:is(#wrap, .missing)) error = %v", err)
	}
	if err := harness.AssertExists("section:where(#wrap)"); err != nil {
		t.Fatalf("AssertExists(section:where(#wrap)) error = %v", err)
	}
	if err := harness.AssertExists("article:where(.hit, .miss)"); err != nil {
		t.Fatalf("AssertExists(article:where(.hit, .miss)) error = %v", err)
	}
	if err := harness.AssertExists("article:is(.hit)"); err != nil {
		t.Fatalf("AssertExists(article:is(.hit)) error = %v", err)
	}
	if err := harness.AssertExists("#plain:is(.hit)"); err == nil {
		t.Fatalf("AssertExists(#plain:is(.hit)) error = nil, want no match")
	}
}

func TestScopePseudoClassMatchesDocumentRootContext(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><section id="panel"><p id="child">one</p></section><p id="sibling">two</p></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists(":scope"); err != nil {
		t.Fatalf("AssertExists(:scope) error = %v", err)
	}
	if err := harness.AssertExists(":scope > section"); err != nil {
		t.Fatalf("AssertExists(:scope > section) error = %v", err)
	}
	if err := harness.AssertExists(":scope > p"); err != nil {
		t.Fatalf("AssertExists(:scope > p) error = %v", err)
	}
	if err := harness.AssertExists("section :scope"); err == nil {
		t.Fatalf("AssertExists(section :scope) error = nil, want no match")
	}
}

func TestNthPseudoClassMatchesChildPositions(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><ul id="list"><li id="one">1</li><li id="two">2</li><li id="three">3</li><li id="four">4</li><li id="five">5</li></ul><div id="mixed"><p id="para-a">A</p><span id="mid">M</span><p id="para-b">B</p><p id="para-c">C</p></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertText("li:nth-child(3)", "3"); err != nil {
		t.Fatalf("AssertText(li:nth-child(3)) error = %v", err)
	}
	if err := harness.AssertExists("li:nth-child(odd)"); err != nil {
		t.Fatalf("AssertExists(li:nth-child(odd)) error = %v", err)
	}
	if err := harness.AssertText("p:nth-of-type(3)", "C"); err != nil {
		t.Fatalf("AssertText(p:nth-of-type(3)) error = %v", err)
	}
	if err := harness.AssertText("li:nth-last-child(1)", "5"); err != nil {
		t.Fatalf("AssertText(li:nth-last-child(1)) error = %v", err)
	}
	if err := harness.AssertText("p:nth-last-of-type(2)", "B"); err != nil {
		t.Fatalf("AssertText(p:nth-last-of-type(2)) error = %v", err)
	}
	if err := harness.AssertExists("span:nth-of-type(2)"); err == nil {
		t.Fatalf("AssertExists(span:nth-of-type(2)) error = nil, want no match")
	}
	if err := harness.AssertExists("li:nth-last-child(6)"); err == nil {
		t.Fatalf("AssertExists(li:nth-last-child(6)) error = nil, want no match")
	}
}

func TestTargetPseudoClassTracksLocationFragments(t *testing.T) {
	harness, err := FromHTMLWithURL("https://example.test/page#legacy", `<main id="root"><a name="legacy">legacy</a><div id="space target">space</div><p id="tail">tail</p></main>`)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if err := harness.AssertText("a:target", "legacy"); err != nil {
		t.Fatalf("AssertText(a:target) error = %v", err)
	}
	if err := harness.AssertExists("main:target-within"); err != nil {
		t.Fatalf("AssertExists(main:target-within) after bootstrap error = %v", err)
	}
	if err := harness.Navigate("#space%20target"); err != nil {
		t.Fatalf("Navigate(#space%%20target) error = %v", err)
	}
	if err := harness.AssertText("div:target", "space"); err != nil {
		t.Fatalf("AssertText(div:target) error = %v", err)
	}
	if err := harness.AssertExists("main:target-within"); err != nil {
		t.Fatalf("AssertExists(main:target-within) after encoded fragment error = %v", err)
	}
	if err := harness.Navigate("#missing"); err != nil {
		t.Fatalf("Navigate(#missing) error = %v", err)
	}
	if err := harness.AssertExists(":target"); err == nil {
		t.Fatalf("AssertExists(:target) after missing fragment error = nil, want no match")
	}
	if err := harness.AssertExists(":target-within"); err == nil {
		t.Fatalf("AssertExists(:target-within) after missing fragment error = nil, want no match")
	}
}

func TestLangPseudoClassTracksInheritedLanguage(t *testing.T) {
	harness, err := FromHTML(`<main id="root" lang="en-US"><section id="panel"><p id="inherited">Hello</p></section><article id="french" lang="fr"><span id="direct">Salut</span><div id="unknown" lang=""><em id="blank">Nada</em></div></article></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertText("p:lang(en)", "Hello"); err != nil {
		t.Fatalf("AssertText(p:lang(en)) error = %v", err)
	}
	if err := harness.AssertText("span:lang(fr)", "Salut"); err != nil {
		t.Fatalf("AssertText(span:lang(fr)) error = %v", err)
	}

	if err := harness.SetAttribute("#root", "lang", "fr"); err != nil {
		t.Fatalf("SetAttribute(#root, lang, fr) error = %v", err)
	}
	if err := harness.AssertText("p:lang(fr)", "Hello"); err != nil {
		t.Fatalf("AssertText(p:lang(fr)) after SetAttribute error = %v", err)
	}
	if err := harness.AssertExists("p:lang(en)"); err == nil {
		t.Fatalf("AssertExists(p:lang(en)) after SetAttribute error = nil, want no match")
	}
}

func TestDirPseudoClassTracksInheritedDirection(t *testing.T) {
	harness, err := FromHTML(`<main id="root" dir="rtl"><section id="panel"><p id="inherited">Hello</p><div id="auto-ltr" dir="auto">abc</div><div id="auto-rtl" dir="auto">مرحبا</div></section><article id="ltr" dir="ltr"><span id="nested">Salut</span></article></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertText("p:dir(rtl)", "Hello"); err != nil {
		t.Fatalf("AssertText(p:dir(rtl)) error = %v", err)
	}
	if err := harness.AssertText("div:dir(ltr)", "abc"); err != nil {
		t.Fatalf("AssertText(div:dir(ltr)) error = %v", err)
	}
	if err := harness.AssertText("div:dir(rtl)", "مرحبا"); err != nil {
		t.Fatalf("AssertText(div:dir(rtl)) error = %v", err)
	}
	if err := harness.AssertText("span:dir(ltr)", "Salut"); err != nil {
		t.Fatalf("AssertText(span:dir(ltr)) error = %v", err)
	}

	if err := harness.SetAttribute("#root", "dir", "ltr"); err != nil {
		t.Fatalf("SetAttribute(#root, dir, ltr) error = %v", err)
	}
	if err := harness.AssertText("p:dir(ltr)", "Hello"); err != nil {
		t.Fatalf("AssertText(p:dir(ltr)) after SetAttribute error = %v", err)
	}
	if err := harness.AssertExists("p:dir(rtl)"); err == nil {
		t.Fatalf("AssertExists(p:dir(rtl)) after SetAttribute error = nil, want no match")
	}
}

func TestWriteHTMLRejectsInvalidMarkupWithoutMutatingDocument(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out">old</div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.WriteHTML(`<main><div id="broken"></main>`); err == nil {
		t.Fatalf("WriteHTML(invalid) error = nil, want parse error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("WriteHTML(invalid) error = %#v, want DOM error", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">old</div></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after failed WriteHTML = %q, want %q", got, want)
	}
	if got, want := harness.HTML(), `<main><div id="out">old</div></main>`; got != want {
		t.Fatalf("HTML() after failed WriteHTML = %q, want %q", got, want)
	}
}

func TestNilHarnessWriteHTMLWrapperReturnsError(t *testing.T) {
	var harness *Harness

	if err := harness.WriteHTML("<main></main>"); err == nil {
		t.Fatalf("nil Harness.WriteHTML() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.WriteHTML() error = %#v, want DOM error", err)
	}
}

func TestInlineScriptsDispatchTargetListenersThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><button id="btn">Go</button><div id="out"></div><script>host:addEventListener("#btn", "click", 'host:setInnerHTML("#out", "clicked"); host:setInnerHTML("#out", "done")')</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><button id="btn">Go</button><div id="out">done</div><script>host:addEventListener("#btn", "click", 'host:setInnerHTML("#out", "clicked"); host:setInnerHTML("#out", "done")')</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after listener dispatch = %q, want %q", got, want)
	}
}

func TestInlineScriptsDispatchCaptureTargetAndBubbleListenersThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><section id="wrap"><button id="btn">Go</button></section><div id="log"></div><script>host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>")', "capture"); host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"><span>capture</span><span>target</span><span>bubble</span></div><script>host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>")', "capture"); host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after capture/target/bubble listeners = %q, want %q", got, want)
	}
}

func TestInlineScriptsCanPreventDefaultActionsThroughPublicActions(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/base/",
		`<main><a id="nav" href="/next">Go</a><div id="out"></div><script>host:addEventListener("#nav", "click", 'host:preventDefault(); host:setInnerHTML("#out", "blocked")')</script></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if err := harness.Click("#nav"); err != nil {
		t.Fatalf("Click(#nav) error = %v", err)
	}

	if got, want := harness.URL(), "https://example.test/base/"; got != want {
		t.Fatalf("URL() after prevented click = %q, want %q", got, want)
	}
	if got, want := harness.Debug().DumpDOM(), `<main><a id="nav" href="/next">Go</a><div id="out">blocked</div><script>host:addEventListener("#nav", "click", 'host:preventDefault(); host:setInnerHTML("#out", "blocked")')</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after prevented click = %q, want %q", got, want)
	}
	if got := harness.Mocks().Location().Navigations(); len(got) != 0 {
		t.Fatalf("Location().Navigations() = %#v, want no navigation", got)
	}
}

func TestInlineScriptsCanStopPropagationThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><section id="wrap"><button id="btn">Go</button></section><div id="log"></div><script>host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>"); host:stopPropagation()', "capture"); host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"><span>capture</span><span>target</span></div><script>host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>"); host:stopPropagation()', "capture"); host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after stopPropagation click = %q, want %q", got, want)
	}
}

func TestInlineScriptsCanUseOnceListenersThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><button id="btn">Go</button><div id="log"></div><script>host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>once</span>")', "target", true)</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) first error = %v", err)
	}
	if err := harness.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) second error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><button id="btn">Go</button><div id="log"><span>once</span></div><script>host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>once</span>")', "target", true)</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after once listener = %q, want %q", got, want)
	}
}

func TestInlineScriptsCanRemoveLaterListenersThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><section id="wrap"><button id="btn">Go</button></section><div id="log"></div><script>host:addEventListener("#wrap", "click", 'host:removeEventListener("#btn", "click", host:removeNode("#btn")); host:insertAdjacentHTML("#log", "beforeend", "<span>remover</span>")', "capture"); host:addEventListener("#btn", "click", 'host:removeNode("#btn")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"><span>remover</span><span>bubble</span></div><script>host:addEventListener("#wrap", "click", 'host:removeEventListener("#btn", "click", host:removeNode("#btn")); host:insertAdjacentHTML("#log", "beforeend", "<span>remover</span>")', "capture"); host:addEventListener("#btn", "click", 'host:removeNode("#btn")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after listener removal = %q, want %q", got, want)
	}
}

func TestInlineScriptsCanDispatchCustomEventsThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><section id="wrap"><button id="btn">Go</button></section><div id="log"></div><script>host:addEventListener("#wrap", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>")', "capture"); host:addEventListener("#btn", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Dispatch("#btn", "custom"); err != nil {
		t.Fatalf("Dispatch(#btn, custom) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"><span>capture</span><span>target</span><span>bubble</span></div><script>host:addEventListener("#wrap", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>")', "capture"); host:addEventListener("#btn", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after custom dispatch = %q, want %q", got, want)
	}
}

func TestInlineScriptsCanDispatchKeyboardSequencesThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><button id="btn">Go</button><div id="log"></div><script>host:addEventListener("#btn", "keydown", 'host:insertAdjacentHTML("#log", "beforeend", "<span>down</span>")'); host:addEventListener("#btn", "keypress", 'host:insertAdjacentHTML("#log", "beforeend", "<span>press</span>")'); host:addEventListener("#btn", "keyup", 'host:insertAdjacentHTML("#log", "beforeend", "<span>up</span>")')</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.DispatchKeyboard("#btn"); err != nil {
		t.Fatalf("DispatchKeyboard(#btn) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><button id="btn">Go</button><div id="log"><span>down</span><span>press</span><span>up</span></div><script>host:addEventListener("#btn", "keydown", 'host:insertAdjacentHTML("#log", "beforeend", "<span>down</span>")'); host:addEventListener("#btn", "keypress", 'host:insertAdjacentHTML("#log", "beforeend", "<span>press</span>")'); host:addEventListener("#btn", "keyup", 'host:insertAdjacentHTML("#log", "beforeend", "<span>up</span>")')</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after keyboard dispatch = %q, want %q", got, want)
	}
}

func TestInlineScriptsQueueMicrotasksDuringBootstrapThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out">start</div><script>host:queueMicrotask('host:setInnerHTML(#out, micro)')</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">micro</div><script>host:queueMicrotask('host:setInnerHTML(#out, micro)')</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after bootstrap microtask = %q, want %q", got, want)
	}
}

func TestInlineScriptsQueueMicrotasksAfterClickThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><button id="btn">Go</button><div id="out">start</div><script>host:addEventListener("#btn", "click", "host:queueMicrotask('host:setInnerHTML(#out, micro)')")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><button id="btn">Go</button><div id="out">micro</div><script>host:addEventListener("#btn", "click", "host:queueMicrotask('host:setInnerHTML(#out, micro)')")</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after click microtask = %q, want %q", got, want)
	}
}

func TestInlineScriptsCanAdvanceTimersThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out">start</div><script>host:setTimeout('host:setInnerHTML(#out, micro)', 5)</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">start</div><script>host:setTimeout('host:setInnerHTML(#out, micro)', 5)</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() before AdvanceTime = %q, want %q", got, want)
	}
	if err := harness.AdvanceTime(4); err != nil {
		t.Fatalf("AdvanceTime(4) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">start</div><script>host:setTimeout('host:setInnerHTML(#out, micro)', 5)</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() before timer due = %q, want %q", got, want)
	}
	if err := harness.AdvanceTime(1); err != nil {
		t.Fatalf("AdvanceTime(1) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">micro</div><script>host:setTimeout('host:setInnerHTML(#out, micro)', 5)</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after timer = %q, want %q", got, want)
	}
}

func TestInlineScriptsCanScheduleRepeatingTimersThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out">start</div><script>host:setInterval('host:insertAdjacentHTML("#out", "beforeend", "<span>tick</span>")', 5)</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AdvanceTime(5); err != nil {
		t.Fatalf("AdvanceTime(5) first error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">start<span>tick</span></div><script>host:setInterval('host:insertAdjacentHTML("#out", "beforeend", "<span>tick</span>")', 5)</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after first interval = %q, want %q", got, want)
	}

	if err := harness.AdvanceTime(5); err != nil {
		t.Fatalf("AdvanceTime(5) second error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">start<span>tick</span><span>tick</span></div><script>host:setInterval('host:insertAdjacentHTML("#out", "beforeend", "<span>tick</span>")', 5)</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after second interval = %q, want %q", got, want)
	}
}

func TestFormControlActionsRejectUnsupportedTargets(t *testing.T) {
	harness, err := FromHTML(`<main><input id="name"><input id="flag" type="checkbox"><select id="mode"><option>A</option></select></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.TypeText("#flag", "Ada"); err == nil {
		t.Fatalf("TypeText(#flag) error = nil, want unsupported control error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("TypeText(#flag) error = %#v, want DOM error", err)
	}
	if err := harness.SetChecked("#name", true); err == nil {
		t.Fatalf("SetChecked(#name) error = nil, want unsupported control error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("SetChecked(#name) error = %#v, want DOM error", err)
	}
	if err := harness.SetSelectValue("#name", "A"); err == nil {
		t.Fatalf("SetSelectValue(#name) error = nil, want unsupported control error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("SetSelectValue(#name) error = %#v, want DOM error", err)
	}
	if err := harness.Submit("#name"); err == nil {
		t.Fatalf("Submit(#name) error = nil, want unsupported target error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("Submit(#name) error = %#v, want DOM error", err)
	}
}

func TestNilHarnessFormControlWrappersReturnErrors(t *testing.T) {
	var harness *Harness

	if err := harness.TypeText("#name", "Ada"); err == nil {
		t.Fatalf("nil Harness.TypeText() error = nil, want DOM error")
	}
	if err := harness.SetChecked("#flag", true); err == nil {
		t.Fatalf("nil Harness.SetChecked() error = nil, want DOM error")
	}
	if err := harness.SetSelectValue("#mode", "B"); err == nil {
		t.Fatalf("nil Harness.SetSelectValue() error = nil, want DOM error")
	}
	if err := harness.Submit("#profile"); err == nil {
		t.Fatalf("nil Harness.Submit() error = nil, want DOM error")
	}
}

func TestNilHarnessEventWrappersReturnErrors(t *testing.T) {
	var harness *Harness

	if err := harness.Click("#cta"); err == nil {
		t.Fatalf("nil Harness.Click() error = nil, want event error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindEvent {
		t.Fatalf("nil Harness.Click() error = %#v, want event error", err)
	}
	if err := harness.Focus("#cta"); err == nil {
		t.Fatalf("nil Harness.Focus() error = nil, want event error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindEvent {
		t.Fatalf("nil Harness.Focus() error = %#v, want event error", err)
	}
	if err := harness.Blur(); err == nil {
		t.Fatalf("nil Harness.Blur() error = nil, want event error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindEvent {
		t.Fatalf("nil Harness.Blur() error = %#v, want event error", err)
	}
	if err := harness.Dispatch("#cta", "custom"); err == nil {
		t.Fatalf("nil Harness.Dispatch() error = nil, want event error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindEvent {
		t.Fatalf("nil Harness.Dispatch() error = %#v, want event error", err)
	}
	if err := harness.DispatchKeyboard("#cta"); err == nil {
		t.Fatalf("nil Harness.DispatchKeyboard() error = nil, want event error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindEvent {
		t.Fatalf("nil Harness.DispatchKeyboard() error = %#v, want event error", err)
	}
}

func TestNilHarnessTimeWrappersReturnErrors(t *testing.T) {
	var harness *Harness

	if err := harness.AdvanceTime(1); err == nil {
		t.Fatalf("nil Harness.AdvanceTime() error = nil, want timer error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindTimer {
		t.Fatalf("nil Harness.AdvanceTime() error = %#v, want timer error", err)
	}
}

func TestNilHarnessMatchMediaWrapperReturnsError(t *testing.T) {
	var harness *Harness

	if _, err := harness.MatchMedia("(prefers-reduced-motion: reduce)"); err == nil {
		t.Fatalf("nil Harness.MatchMedia() error = nil, want mock error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindMock {
		t.Fatalf("nil Harness.MatchMedia() error = %#v, want mock error", err)
	}
}

func TestNilHarnessAttributeWrappersReturnErrors(t *testing.T) {
	var harness *Harness

	if _, _, err := harness.GetAttribute("#root", "id"); err == nil {
		t.Fatalf("nil Harness.GetAttribute() error = nil, want DOM error")
	}
	if _, err := harness.HasAttribute("#root", "id"); err == nil {
		t.Fatalf("nil Harness.HasAttribute() error = nil, want DOM error")
	}
	if err := harness.SetAttribute("#root", "id", "x"); err == nil {
		t.Fatalf("nil Harness.SetAttribute() error = nil, want DOM error")
	}
	if err := harness.RemoveAttribute("#root", "id"); err == nil {
		t.Fatalf("nil Harness.RemoveAttribute() error = nil, want DOM error")
	}
}

func TestInlineScriptsMutateDOMDuringBootstrap(t *testing.T) {
	harness, err := FromHTML(`<main><div id="target">old</div><script>host:setInnerHTML("#target", "<em>updated</em>")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="target"><em>updated</em></div><script>host:setInnerHTML("#target", "<em>updated</em>")</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after inline script bootstrap = %q, want %q", got, want)
	}
}

func TestMutationContractsGettersAndSetters(t *testing.T) {
	harness, err := FromHTML(`<section id="wrap"><div id="target"><p>Hello</p><span>world</span></div><p id="tail">tail</p></section>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, err := harness.InnerHTML("#target"); err != nil {
		t.Fatalf("InnerHTML(#target) error = %v", err)
	} else if want := `<p>Hello</p><span>world</span>`; got != want {
		t.Fatalf("InnerHTML(#target) = %q, want %q", got, want)
	}

	if got, err := harness.OuterHTML("#target"); err != nil {
		t.Fatalf("OuterHTML(#target) error = %v", err)
	} else if want := `<div id="target"><p>Hello</p><span>world</span></div>`; got != want {
		t.Fatalf("OuterHTML(#target) = %q, want %q", got, want)
	}

	if err := harness.SetInnerHTML("#target", `<em id="next">updated</em>tail`); err != nil {
		t.Fatalf("SetInnerHTML(#target) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<section id="wrap"><div id="target"><em id="next">updated</em>tail</div><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("Debug().DumpDOM() after SetInnerHTML = %q, want %q", got, want)
	}

	if err := harness.InsertAdjacentHTML("#target", "beforebegin", `<a id="bb"></a>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(beforebegin) error = %v", err)
	}
	if err := harness.InsertAdjacentHTML("#target", "afterbegin", `<i id="ab">a</i>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(afterbegin) error = %v", err)
	}
	if err := harness.InsertAdjacentHTML("#target", "beforeend", `<i id="be">b</i>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(beforeend) error = %v", err)
	}
	if err := harness.InsertAdjacentHTML("#target", "afterend", `<a id="ae"></a>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(afterend) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<section id="wrap"><a id="bb"></a><div id="target"><i id="ab">a</i><em id="next">updated</em>tail<i id="be">b</i></div><a id="ae"></a><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("Debug().DumpDOM() after InsertAdjacentHTML = %q, want %q", got, want)
	}

	if err := harness.SetOuterHTML("#target", `<article id="next-outer">n</article><aside id="extra"></aside>`); err != nil {
		t.Fatalf("SetOuterHTML(#target) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<section id="wrap"><a id="bb"></a><article id="next-outer">n</article><aside id="extra"></aside><a id="ae"></a><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("Debug().DumpDOM() after SetOuterHTML = %q, want %q", got, want)
	}
}

func TestMutationContractsRemoveNodeRemovesSubtree(t *testing.T) {
	harness, err := FromHTML(`<section id="wrap"><div id="remove"><span id="child">x</span></div><p id="keep">k</p></section>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.RemoveNode("#remove"); err != nil {
		t.Fatalf("RemoveNode(#remove) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<section id="wrap"><p id="keep">k</p></section>`; got != want {
		t.Fatalf("Debug().DumpDOM() after RemoveNode = %q, want %q", got, want)
	}
	if err := harness.RemoveNode("#child"); err == nil {
		t.Fatalf("RemoveNode(#child) error = nil, want DOM error after subtree removal")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("RemoveNode(#child) error = %#v, want DOM error", err)
	}
}

func TestMutationContractsRejectInvalidTargets(t *testing.T) {
	harness, err := FromHTML(`<div id="target"><span>ok</span></div><p id="sibling">tail</p>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if _, err := harness.InnerHTML("#missing"); err == nil {
		t.Fatalf("InnerHTML(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("InnerHTML(#missing) error = %#v, want DOM error", err)
	}

	if _, err := harness.OuterHTML("#missing"); err == nil {
		t.Fatalf("OuterHTML(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("OuterHTML(#missing) error = %#v, want DOM error", err)
	}

	if err := harness.SetInnerHTML("#missing", "<p>x</p>"); err == nil {
		t.Fatalf("SetInnerHTML(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("SetInnerHTML(#missing) error = %#v, want DOM error", err)
	}

	if err := harness.SetOuterHTML("#missing", "<p>x</p>"); err == nil {
		t.Fatalf("SetOuterHTML(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("SetOuterHTML(#missing) error = %#v, want DOM error", err)
	}

	if err := harness.InsertAdjacentHTML("#missing", "beforeend", "<p>x</p>"); err == nil {
		t.Fatalf("InsertAdjacentHTML(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("InsertAdjacentHTML(#missing) error = %#v, want DOM error", err)
	}

	if err := harness.RemoveNode("#missing"); err == nil {
		t.Fatalf("RemoveNode(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("RemoveNode(#missing) error = %#v, want DOM error", err)
	}
}

func TestMutationContractsRejectDocumentParentRestrictions(t *testing.T) {
	harness, err := FromHTML(`<div id="target"><span>ok</span></div><p id="sibling">tail</p>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.SetOuterHTML("#target", `<section id="new"></section>`); err == nil {
		t.Fatalf("SetOuterHTML(document child) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("SetOuterHTML(document child) error = %#v, want DOM error", err)
	}

	if err := harness.InsertAdjacentHTML("#target", "beforebegin", `<a id="bb"></a>`); err == nil {
		t.Fatalf("InsertAdjacentHTML(beforebegin on document child) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("InsertAdjacentHTML(beforebegin on document child) error = %#v, want DOM error", err)
	}

	if err := harness.InsertAdjacentHTML("#target", "afterend", `<a id="ae"></a>`); err == nil {
		t.Fatalf("InsertAdjacentHTML(afterend on document child) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("InsertAdjacentHTML(afterend on document child) error = %#v, want DOM error", err)
	}
}

func TestNilHarnessMutationWrappersReturnDomErrors(t *testing.T) {
	var nilHarness *Harness

	if _, err := nilHarness.InnerHTML("#target"); err == nil {
		t.Fatalf("nil Harness.InnerHTML() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.InnerHTML() error = %#v, want DOM error", err)
	}
	if _, err := nilHarness.OuterHTML("#target"); err == nil {
		t.Fatalf("nil Harness.OuterHTML() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.OuterHTML() error = %#v, want DOM error", err)
	}
	if err := nilHarness.SetInnerHTML("#target", "<p>x</p>"); err == nil {
		t.Fatalf("nil Harness.SetInnerHTML() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.SetInnerHTML() error = %#v, want DOM error", err)
	}
	if err := nilHarness.SetOuterHTML("#target", "<p>x</p>"); err == nil {
		t.Fatalf("nil Harness.SetOuterHTML() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.SetOuterHTML() error = %#v, want DOM error", err)
	}
	if err := nilHarness.InsertAdjacentHTML("#target", "beforeend", "<p>x</p>"); err == nil {
		t.Fatalf("nil Harness.InsertAdjacentHTML() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.InsertAdjacentHTML() error = %#v, want DOM error", err)
	}
	if err := nilHarness.RemoveNode("#target"); err == nil {
		t.Fatalf("nil Harness.RemoveNode() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.RemoveNode() error = %#v, want DOM error", err)
	}

	zeroSessionHarness := &Harness{}
	if _, err := zeroSessionHarness.InnerHTML("#target"); err == nil {
		t.Fatalf("Harness{nil session}.InnerHTML() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("Harness{nil session}.InnerHTML() error = %#v, want DOM error", err)
	}
}
