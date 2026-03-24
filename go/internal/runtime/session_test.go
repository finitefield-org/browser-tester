package runtime

import "testing"

func TestSessionAppliesConfigSeedsDeterministically(t *testing.T) {
	local := map[string]string{"token": "abc"}
	sessionStorage := map[string]string{"tab": "main"}
	match := map[string]bool{"(prefers-reduced-motion: reduce)": true}
	cfg := SessionConfig{
		URL:            "https://example.test/",
		LocalStorage:   local,
		SessionStorage: sessionStorage,
		MatchMedia:     match,
		OpenFailure:    "open blocked",
		CloseFailure:   "close blocked",
		PrintFailure:   "print blocked",
		ScrollFailure:  "scroll blocked",
	}

	s := NewSession(cfg)

	// Mutate source maps after NewSession to ensure config cloning is effective.
	local["token"] = "mutated"
	sessionStorage["tab"] = "mutated"
	match["(prefers-reduced-motion: reduce)"] = false

	if got, want := s.URL(), "https://example.test/"; got != want {
		t.Fatalf("URL() = %q, want %q", got, want)
	}

	if got, want := s.Registry().Storage().Local()["token"], "abc"; got != want {
		t.Fatalf("Storage().Local()[token] = %q, want %q", got, want)
	}
	if got, want := s.Registry().Storage().Session()["tab"], "main"; got != want {
		t.Fatalf("Storage().Session()[tab] = %q, want %q", got, want)
	}

	matches, err := s.MatchMedia("(prefers-reduced-motion: reduce)")
	if err != nil {
		t.Fatalf("MatchMedia() error = %v", err)
	}
	if !matches {
		t.Fatalf("MatchMedia() = false, want true")
	}

	if err := s.Open("https://example.test/new"); err == nil {
		t.Fatalf("Open() error = nil, want seeded failure")
	}
	if err := s.Close(); err == nil {
		t.Fatalf("Close() error = nil, want seeded failure")
	}
	if err := s.Print(); err == nil {
		t.Fatalf("Print() error = nil, want seeded failure")
	}
	if err := s.ScrollTo(1, 2); err == nil {
		t.Fatalf("ScrollTo() error = nil, want seeded failure")
	}
}

func TestSessionSchedulerBackedTime(t *testing.T) {
	s := NewSession(DefaultSessionConfig())

	if got, want := s.NowMs(), int64(0); got != want {
		t.Fatalf("NowMs() = %d, want %d", got, want)
	}

	if err := s.AdvanceTime(25); err != nil {
		t.Fatalf("AdvanceTime() error = %v", err)
	}
	if got, want := s.NowMs(), int64(25); got != want {
		t.Fatalf("NowMs() after AdvanceTime = %d, want %d", got, want)
	}

	s.Scheduler().Advance(10)
	if got, want := s.NowMs(), int64(35); got != want {
		t.Fatalf("NowMs() after Scheduler().Advance = %d, want %d", got, want)
	}

	s.SetNowMs(7)
	if got, want := s.NowMs(), int64(7); got != want {
		t.Fatalf("NowMs() after SetNowMs = %d, want %d", got, want)
	}

	s.ResetTime()
	if got, want := s.NowMs(), int64(0); got != want {
		t.Fatalf("NowMs() after ResetTime = %d, want %d", got, want)
	}

	if err := s.AdvanceTime(-1); err == nil {
		t.Fatalf("AdvanceTime(-1) error = nil, want validation error")
	}
	if got, want := s.NowMs(), int64(0); got != want {
		t.Fatalf("NowMs() after rejected negative advance = %d, want %d", got, want)
	}
}

func TestSessionConfigReturnsDeepClones(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:            "https://example.test/",
		LocalStorage:   map[string]string{"token": "abc"},
		SessionStorage: map[string]string{"tab": "main"},
		MatchMedia:     map[string]bool{"(prefers-reduced-motion: reduce)": true},
	})

	config := s.Config()
	config.LocalStorage["token"] = "mutated"
	config.LocalStorage["extra"] = "new"
	config.SessionStorage["tab"] = "mutated"
	config.SessionStorage["extra"] = "new"
	config.MatchMedia["(prefers-reduced-motion: reduce)"] = false
	config.MatchMedia["(prefers-color-scheme: dark)"] = true

	fresh := s.Config()
	if got, want := fresh.LocalStorage["token"], "abc"; got != want {
		t.Fatalf("fresh Config().LocalStorage()[token] = %q, want %q", got, want)
	}
	if _, ok := fresh.LocalStorage["extra"]; ok {
		t.Fatalf("fresh Config().LocalStorage()[extra] should not exist")
	}
	if got, want := fresh.SessionStorage["tab"], "main"; got != want {
		t.Fatalf("fresh Config().SessionStorage()[tab] = %q, want %q", got, want)
	}
	if _, ok := fresh.SessionStorage["extra"]; ok {
		t.Fatalf("fresh Config().SessionStorage()[extra] should not exist")
	}
	if got, want := fresh.MatchMedia["(prefers-reduced-motion: reduce)"], true; got != want {
		t.Fatalf("fresh Config().MatchMedia()[reduce] = %v, want %v", got, want)
	}
	if _, ok := fresh.MatchMedia["(prefers-color-scheme: dark)"]; ok {
		t.Fatalf("fresh Config().MatchMedia()[dark] should not exist")
	}

	if got, want := s.Registry().Storage().Local()["token"], "abc"; got != want {
		t.Fatalf("Storage().Local()[token] = %q, want %q", got, want)
	}
	if got, want := s.Registry().Storage().Session()["tab"], "main"; got != want {
		t.Fatalf("Storage().Session()[tab] = %q, want %q", got, want)
	}

	matches, err := s.MatchMedia("(prefers-reduced-motion: reduce)")
	if err != nil {
		t.Fatalf("MatchMedia(reduce) error = %v", err)
	}
	if !matches {
		t.Fatalf("MatchMedia(reduce) = false, want true")
	}
	if _, err := s.MatchMedia("(prefers-color-scheme: dark)"); err == nil {
		t.Fatalf("MatchMedia(dark) error = nil, want unseeded query error")
	}
}

func TestSessionNavigateResetsScrollState(t *testing.T) {
	s := NewSession(DefaultSessionConfig())

	if err := s.ScrollTo(10, 20); err != nil {
		t.Fatalf("ScrollTo() error = %v", err)
	}
	if err := s.ScrollBy(3, -4); err != nil {
		t.Fatalf("ScrollBy() error = %v", err)
	}
	if gotX, gotY := s.scrollX, s.scrollY; gotX != 13 || gotY != 16 {
		t.Fatalf("scroll state = (%d, %d), want (13, 16)", gotX, gotY)
	}

	if err := s.Navigate("https://example.test/next"); err != nil {
		t.Fatalf("Navigate() error = %v", err)
	}
	if got, want := s.URL(), "https://example.test/next"; got != want {
		t.Fatalf("URL() after Navigate = %q, want %q", got, want)
	}
	if got := s.Registry().Location().Navigations(); len(got) != 1 || got[0] != "https://example.test/next" {
		t.Fatalf("Location().Navigations() = %#v, want one navigation to example.test/next", got)
	}
	if gotX, gotY := s.scrollX, s.scrollY; gotX != 0 || gotY != 0 {
		t.Fatalf("scroll state after Navigate = (%d, %d), want (0, 0)", gotX, gotY)
	}
}

func TestSessionAttributeReflectionDelegatesToDOM(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><div id="root" data-x="1"></div></main>`,
	})

	if got, ok, err := s.GetAttribute("#root", "data-x"); err != nil || !ok || got != "1" {
		t.Fatalf("GetAttribute(data-x) = (%q, %v, %v), want (\"1\", true, nil)", got, ok, err)
	}
	if ok, err := s.HasAttribute("#root", "data-x"); err != nil || !ok {
		t.Fatalf("HasAttribute(data-x) = (%v, %v), want (true, nil)", ok, err)
	}

	if err := s.SetAttribute("#root", "data-x", "2"); err != nil {
		t.Fatalf("SetAttribute(data-x) error = %v", err)
	}
	if got, ok, err := s.GetAttribute("#root", "data-x"); err != nil || !ok || got != "2" {
		t.Fatalf("GetAttribute(data-x) after SetAttribute = (%q, %v, %v), want (\"2\", true, nil)", got, ok, err)
	}

	if err := s.RemoveAttribute("#root", "data-x"); err != nil {
		t.Fatalf("RemoveAttribute(data-x) error = %v", err)
	}
	if got, ok, err := s.GetAttribute("#root", "data-x"); err != nil || ok || got != "" {
		t.Fatalf("GetAttribute(data-x) after RemoveAttribute = (%q, %v, %v), want (\"\", false, nil)", got, ok, err)
	}
}

func TestSessionExecutesInlineScriptsDuringBootstrap(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><div id="target">old</div><script>host:setInnerHTML("#target", "<em>updated</em>")</script></main>`,
	})

	if got, want := s.DumpDOM(), `<main><div id="target"><em>updated</em></div><script>host:setInnerHTML("#target", "<em>updated</em>")</script></main>`; got != want {
		t.Fatalf("DumpDOM() after inline script bootstrap = %q, want %q", got, want)
	}

	if got, err := s.OuterHTML("#target"); err != nil {
		t.Fatalf("OuterHTML(#target) error = %v", err)
	} else if want := `<div id="target"><em>updated</em></div>`; got != want {
		t.Fatalf("OuterHTML(#target) = %q, want %q", got, want)
	}
}

func TestSessionRejectsInlineScriptHostErrors(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><div id="target">old</div><script>host:setInnerHTML("#missing", "<em>updated</em>")</script></main>`,
	})

	if _, err := s.ensureDOM(); err == nil {
		t.Fatalf("ensureDOM() error = nil, want inline script host error")
	}
	if got := s.DumpDOM(); got != "" {
		t.Fatalf("DumpDOM() after failed inline script bootstrap = %q, want empty string", got)
	}
}

func TestNilSessionHelpersStaySafe(t *testing.T) {
	var s *Session

	if got := s.URL(); got != "" {
		t.Fatalf("URL() = %q, want empty string", got)
	}
	if got := s.HTML(); got != "" {
		t.Fatalf("HTML() = %q, want empty string", got)
	}
	if got := s.NowMs(); got != 0 {
		t.Fatalf("NowMs() = %d, want 0", got)
	}
	if got := s.Scheduler(); got != nil {
		t.Fatalf("Scheduler() = %#v, want nil", got)
	}
	if got := s.FocusedSelector(); got != "" {
		t.Fatalf("FocusedSelector() = %q, want empty string", got)
	}
	if got := s.InteractionLog(); got != nil {
		t.Fatalf("InteractionLog() = %#v, want nil", got)
	}
	if got := s.DumpDOM(); got != "" {
		t.Fatalf("DumpDOM() = %q, want empty string", got)
	}

	config := s.Config()
	defaultConfig := DefaultSessionConfig()
	if got, want := config.URL, defaultConfig.URL; got != want {
		t.Fatalf("Config().URL = %q, want %q", got, want)
	}
	if len(config.LocalStorage) != 0 {
		t.Fatalf("Config().LocalStorage = %#v, want empty", config.LocalStorage)
	}
	if len(config.SessionStorage) != 0 {
		t.Fatalf("Config().SessionStorage = %#v, want empty", config.SessionStorage)
	}
	if len(config.MatchMedia) != 0 {
		t.Fatalf("Config().MatchMedia = %#v, want empty", config.MatchMedia)
	}

	s.SetNowMs(10)
	s.ResetTime()

	if err := s.AdvanceTime(5); err == nil {
		t.Fatalf("AdvanceTime(5) error = nil, want session unavailable error")
	}
	if err := s.Click("#cta"); err == nil {
		t.Fatalf("Click(#cta) error = nil, want session unavailable error")
	}
	if err := s.TypeText("#cta", "value"); err == nil {
		t.Fatalf("TypeText(#cta) error = nil, want session unavailable error")
	}
	if err := s.SetChecked("#cta", true); err == nil {
		t.Fatalf("SetChecked(#cta) error = nil, want session unavailable error")
	}
	if err := s.SetSelectValue("#cta", "value"); err == nil {
		t.Fatalf("SetSelectValue(#cta) error = nil, want session unavailable error")
	}
	if _, _, err := s.GetAttribute("#cta", "id"); err == nil {
		t.Fatalf("GetAttribute(#cta) error = nil, want session unavailable error")
	}
	if _, err := s.HasAttribute("#cta", "id"); err == nil {
		t.Fatalf("HasAttribute(#cta) error = nil, want session unavailable error")
	}
	if err := s.SetAttribute("#cta", "id", "x"); err == nil {
		t.Fatalf("SetAttribute(#cta) error = nil, want session unavailable error")
	}
	if err := s.RemoveAttribute("#cta", "id"); err == nil {
		t.Fatalf("RemoveAttribute(#cta) error = nil, want session unavailable error")
	}
	if err := s.Submit("#cta"); err == nil {
		t.Fatalf("Submit(#cta) error = nil, want session unavailable error")
	}
	if err := s.Focus("#cta"); err == nil {
		t.Fatalf("Focus(#cta) error = nil, want session unavailable error")
	}
	if err := s.Blur(); err == nil {
		t.Fatalf("Blur() error = nil, want session unavailable error")
	}
}

func TestSessionTracksFocusAndInteractions(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><button id="cta">Go</button><input id="name"></main>`,
	})

	if err := s.Focus(" #name "); err != nil {
		t.Fatalf("Focus(#name) error = %v", err)
	}
	if got, want := s.FocusedSelector(), "#name"; got != want {
		t.Fatalf("FocusedSelector() after Focus = %q, want %q", got, want)
	}

	if err := s.Click("#cta"); err != nil {
		t.Fatalf("Click(#cta) error = %v", err)
	}
	if got, want := s.FocusedSelector(), "#name"; got != want {
		t.Fatalf("FocusedSelector() after Click = %q, want %q", got, want)
	}

	if err := s.Blur(); err != nil {
		t.Fatalf("Blur() error = %v", err)
	}
	if got := s.FocusedSelector(); got != "" {
		t.Fatalf("FocusedSelector() after Blur = %q, want empty", got)
	}

	log := s.InteractionLog()
	if len(log) != 3 {
		t.Fatalf("InteractionLog() len = %d, want 3", len(log))
	}
	if log[0].Kind != InteractionKindFocus || log[0].Selector != "#name" {
		t.Fatalf("InteractionLog()[0] = %#v, want focus #name", log[0])
	}
	if log[1].Kind != InteractionKindClick || log[1].Selector != "#cta" {
		t.Fatalf("InteractionLog()[1] = %#v, want click #cta", log[1])
	}
	if log[2].Kind != InteractionKindBlur || log[2].Selector != "#name" {
		t.Fatalf("InteractionLog()[2] = %#v, want blur #name", log[2])
	}

	log[0].Selector = "mutated"
	fresh := s.InteractionLog()
	if fresh[0].Selector != "#name" {
		t.Fatalf("fresh InteractionLog()[0].Selector = %q, want %q", fresh[0].Selector, "#name")
	}
}

func TestSessionInteractionsValidateSelectorsAgainstDOM(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><button id="cta">Go</button></main>`,
	})

	if err := s.Click("main + button"); err == nil {
		t.Fatalf("Click(main + button) error = nil, want selector syntax error")
	}
	if err := s.Focus("#missing"); err == nil {
		t.Fatalf("Focus(#missing) error = nil, want missing-element error")
	}
	if got := len(s.InteractionLog()); got != 0 {
		t.Fatalf("InteractionLog() len after rejected interactions = %d, want 0", got)
	}
	if got := s.FocusedSelector(); got != "" {
		t.Fatalf("FocusedSelector() after rejected interactions = %q, want empty", got)
	}
}

func TestSessionActionsSupportBoundedCombinators(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><section><button id="cta">Go</button></section><input id="name"></main>`,
	})

	if err := s.Click("main section > button"); err != nil {
		t.Fatalf("Click(main section > button) error = %v", err)
	}
	if err := s.Focus("main > input"); err != nil {
		t.Fatalf("Focus(main > input) error = %v", err)
	}

	if got, want := s.FocusedSelector(), "main > input"; got != want {
		t.Fatalf("FocusedSelector() = %q, want %q", got, want)
	}

	log := s.InteractionLog()
	if len(log) != 2 {
		t.Fatalf("InteractionLog() len = %d, want 2", len(log))
	}
	if log[0].Kind != InteractionKindClick || log[0].Selector != "main section > button" {
		t.Fatalf("InteractionLog()[0] = %#v, want click main section > button", log[0])
	}
	if log[1].Kind != InteractionKindFocus || log[1].Selector != "main > input" {
		t.Fatalf("InteractionLog()[1] = %#v, want focus main > input", log[1])
	}
}

func TestSessionInteractionsReportDOMBootstrapErrors(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div><span></div>`,
	})

	if err := s.Click("span"); err == nil {
		t.Fatalf("Click(span) error = nil, want HTML bootstrap error")
	}
	if err := s.Focus("span"); err == nil {
		t.Fatalf("Focus(span) error = nil, want cached HTML bootstrap error")
	}
	if got := len(s.InteractionLog()); got != 0 {
		t.Fatalf("InteractionLog() len = %d, want 0", got)
	}
}

func TestSessionFormControlsUpdateLiveDomAndLog(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><input id="name"><input id="flag" type="checkbox"><textarea id="bio">Base</textarea><select id="mode"><option value="a" selected>A</option><option>B</option><option value="c">C</option></select><form id="profile"><button id="submit" type="submit">Save</button></form></main>`,
	})

	if err := s.TypeText("#name", "Ada"); err != nil {
		t.Fatalf("TypeText(#name) error = %v", err)
	}
	if err := s.SetChecked("#flag", true); err != nil {
		t.Fatalf("SetChecked(#flag) error = %v", err)
	}
	if err := s.SetSelectValue("#mode", "B"); err != nil {
		t.Fatalf("SetSelectValue(#mode) error = %v", err)
	}
	if err := s.Submit("#profile"); err != nil {
		t.Fatalf("Submit(#profile) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><input id="name" value="Ada"><input id="flag" type="checkbox" checked><textarea id="bio">Base</textarea><select id="mode"><option value="a">A</option><option selected>B</option><option value="c">C</option></select><form id="profile"><button id="submit" type="submit">Save</button></form></main>`; got != want {
		t.Fatalf("DumpDOM() = %q, want %q", got, want)
	}

	log := s.InteractionLog()
	if len(log) != 4 {
		t.Fatalf("InteractionLog() len = %d, want 4", len(log))
	}
	if log[0].Kind != InteractionKindTypeText || log[0].Selector != "#name" {
		t.Fatalf("InteractionLog()[0] = %#v, want type_text #name", log[0])
	}
	if log[1].Kind != InteractionKindSetChecked || log[1].Selector != "#flag" {
		t.Fatalf("InteractionLog()[1] = %#v, want set_checked #flag", log[1])
	}
	if log[2].Kind != InteractionKindSetSelectValue || log[2].Selector != "#mode" {
		t.Fatalf("InteractionLog()[2] = %#v, want set_select_value #mode", log[2])
	}
	if log[3].Kind != InteractionKindSubmit || log[3].Selector != "#profile" {
		t.Fatalf("InteractionLog()[3] = %#v, want submit #profile", log[3])
	}
}

func TestSessionClickAppliesDefaultActions(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<form id="profile"><input id="agree" type="checkbox"><button id="submit" type="submit">Save</button></form>`,
	})

	if err := s.Click("#agree"); err != nil {
		t.Fatalf("Click(#agree) error = %v", err)
	}
	if err := s.Click("#submit"); err != nil {
		t.Fatalf("Click(#submit) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<form id="profile"><input id="agree" type="checkbox" checked><button id="submit" type="submit">Save</button></form>`; got != want {
		t.Fatalf("DumpDOM() = %q, want %q", got, want)
	}

	log := s.InteractionLog()
	if len(log) != 3 {
		t.Fatalf("InteractionLog() len = %d, want 3", len(log))
	}
	if log[0].Kind != InteractionKindClick || log[0].Selector != "#agree" {
		t.Fatalf("InteractionLog()[0] = %#v, want click #agree", log[0])
	}
	if log[1].Kind != InteractionKindClick || log[1].Selector != "#submit" {
		t.Fatalf("InteractionLog()[1] = %#v, want click #submit", log[1])
	}
	if log[2].Kind != InteractionKindSubmit || log[2].Selector != "#submit" {
		t.Fatalf("InteractionLog()[2] = %#v, want submit #submit", log[2])
	}
}

func TestSessionClickAppliesHyperlinkDefaultActions(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/base/",
		HTML: `<main><a id="nav" href="/next">Go</a><map name="hot"><area id="popup" href="https://example.test/popup" target="_blank" alt="Open"></map><a id="download" href="https://example.test/files/report.csv" download="report.csv">Download</a></main>`,
	})

	if err := s.Click("#nav"); err != nil {
		t.Fatalf("Click(#nav) error = %v", err)
	}
	if got, want := s.URL(), "https://example.test/next"; got != want {
		t.Fatalf("URL() after anchor click = %q, want %q", got, want)
	}
	if got := s.Registry().Location().Navigations(); len(got) != 1 || got[0] != "https://example.test/next" {
		t.Fatalf("Location().Navigations() = %#v, want one navigation to https://example.test/next", got)
	}

	if err := s.Click("#popup"); err != nil {
		t.Fatalf("Click(#popup) error = %v", err)
	}
	if got, want := s.URL(), "https://example.test/next"; got != want {
		t.Fatalf("URL() after target=_blank click = %q, want %q", got, want)
	}
	if got := s.Registry().Open().Calls(); len(got) != 1 || got[0].URL != "https://example.test/popup" {
		t.Fatalf("Open().Calls() = %#v, want one open call to popup", got)
	}

	if err := s.Click("#download"); err != nil {
		t.Fatalf("Click(#download) error = %v", err)
	}
	if got, want := s.URL(), "https://example.test/next"; got != want {
		t.Fatalf("URL() after download click = %q, want %q", got, want)
	}
	downloads := s.Registry().Downloads().Artifacts()
	if len(downloads) != 1 || downloads[0].FileName != "report.csv" || string(downloads[0].Bytes) != "https://example.test/files/report.csv" {
		t.Fatalf("Downloads().Artifacts() = %#v, want one captured download", downloads)
	}
}

func TestSessionClickAppliesResetDefaultAction(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<form id="profile"><input id="name"><input id="flag" type="checkbox"><input id="radio-a" type="radio" name="size" checked><input id="radio-b" type="radio" name="size"><textarea id="bio">Base</textarea><select id="mode"><option value="a" selected>A</option><option>B</option><option value="c">C</option></select><button id="reset" type="reset">Reset</button></form>`,
	})

	if err := s.TypeText("#name", "Ada"); err != nil {
		t.Fatalf("TypeText(#name) error = %v", err)
	}
	if err := s.SetChecked("#flag", true); err != nil {
		t.Fatalf("SetChecked(#flag) error = %v", err)
	}
	if err := s.SetChecked("#radio-b", true); err != nil {
		t.Fatalf("SetChecked(#radio-b) error = %v", err)
	}
	if err := s.TypeText("#bio", "Line 1\nLine 2"); err != nil {
		t.Fatalf("TypeText(#bio) error = %v", err)
	}
	if err := s.SetSelectValue("#mode", "B"); err != nil {
		t.Fatalf("SetSelectValue(#mode) error = %v", err)
	}

	if err := s.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<form id="profile"><input id="name"><input id="flag" type="checkbox"><input id="radio-a" type="radio" name="size" checked><input id="radio-b" type="radio" name="size"><textarea id="bio">Base</textarea><select id="mode"><option value="a" selected>A</option><option>B</option><option value="c">C</option></select><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("DumpDOM() after reset click = %q, want %q", got, want)
	}

	log := s.InteractionLog()
	if len(log) != 6 {
		t.Fatalf("InteractionLog() len = %d, want 6", len(log))
	}
	if log[5].Kind != InteractionKindClick || log[5].Selector != "#reset" {
		t.Fatalf("InteractionLog()[5] = %#v, want click #reset", log[5])
	}
}

func TestSessionDispatchesRegisteredClickAndSubmitListeners(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><form id="profile"><button id="submit" type="submit">Save</button></form><div id="out"></div><script>host:addEventListener("#submit", "click", 'host:setInnerHTML("#out", "clicked")'); host:addEventListener("#profile", "submit", 'host:setInnerHTML("#out", "submitted")')</script></main>`,
	})

	if err := s.Click("#submit"); err != nil {
		t.Fatalf("Click(#submit) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><form id="profile"><button id="submit" type="submit">Save</button></form><div id="out">submitted</div><script>host:addEventListener("#submit", "click", 'host:setInnerHTML("#out", "clicked")'); host:addEventListener("#profile", "submit", 'host:setInnerHTML("#out", "submitted")')</script></main>`; got != want {
		t.Fatalf("DumpDOM() after click+submit listeners = %q, want %q", got, want)
	}
}

func TestSessionDispatchesChangeListenersFromSetChecked(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><input id="agree" type="checkbox"><div id="out"></div><script>host:addEventListener("#agree", "change", 'host:setInnerHTML("#out", "changed")')</script></main>`,
	})

	if err := s.SetChecked("#agree", true); err != nil {
		t.Fatalf("SetChecked(#agree) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><input id="agree" type="checkbox" checked><div id="out">changed</div><script>host:addEventListener("#agree", "change", 'host:setInnerHTML("#out", "changed")')</script></main>`; got != want {
		t.Fatalf("DumpDOM() after change listener = %q, want %q", got, want)
	}
}

func TestSessionDispatchesInputListenersFromTypeText(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><input id="name"><div id="out"></div><script>host:addEventListener("#name", "input", 'host:setInnerHTML("#out", "typed")')</script></main>`,
	})

	if err := s.TypeText("#name", "Ada"); err != nil {
		t.Fatalf("TypeText(#name) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><input id="name" value="Ada"><div id="out">typed</div><script>host:addEventListener("#name", "input", 'host:setInnerHTML("#out", "typed")')</script></main>`; got != want {
		t.Fatalf("DumpDOM() after input listener = %q, want %q", got, want)
	}
}

func TestSessionFormControlsRejectUnsupportedTargets(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><input id="name"><input id="flag" type="checkbox"><select id="mode"><option>A</option></select><div id="box"></div></main>`,
	})

	if err := s.TypeText("#flag", "Ada"); err == nil {
		t.Fatalf("TypeText(#flag) error = nil, want unsupported control error")
	}
	if err := s.SetChecked("#name", true); err == nil {
		t.Fatalf("SetChecked(#name) error = nil, want unsupported control error")
	}
	if err := s.SetSelectValue("#name", "A"); err == nil {
		t.Fatalf("SetSelectValue(#name) error = nil, want unsupported control error")
	}
	if err := s.Submit("#box"); err == nil {
		t.Fatalf("Submit(#box) error = nil, want unsupported target error")
	}
}
