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

	if err := harness.Click("#cta"); err != nil {
		t.Fatalf("Click(#cta) error = %v", err)
	}
	if err := harness.Blur(); err != nil {
		t.Fatalf("Blur() error = %v", err)
	}
	if got := harness.Debug().FocusedSelector(); got != "" {
		t.Fatalf("Debug().FocusedSelector() after Blur = %q, want empty", got)
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

	if err := harness.Click("main + button"); err == nil {
		t.Fatalf("Click(main + button) error = nil, want selector syntax error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindEvent {
		t.Fatalf("Click(main + button) error = %#v, want event error", err)
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
	if err := harness.Focus("main > input"); err != nil {
		t.Fatalf("Focus(main > input) error = %v", err)
	}

	if got, want := harness.Debug().FocusedSelector(), "main > input"; got != want {
		t.Fatalf("Debug().FocusedSelector() = %q, want %q", got, want)
	}

	log := harness.Debug().Interactions()
	if len(log) != 2 {
		t.Fatalf("Debug().Interactions() len = %d, want 2", len(log))
	}
	if log[0].Kind != InteractionKindClick || log[0].Selector != "main section > button" {
		t.Fatalf("Debug().Interactions()[0] = %#v, want click main section > button", log[0])
	}
	if log[1].Kind != InteractionKindFocus || log[1].Selector != "main > input" {
		t.Fatalf("Debug().Interactions()[1] = %#v, want focus main > input", log[1])
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
