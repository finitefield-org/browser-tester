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

func TestSessionReportsMatchMediaRules(t *testing.T) {
	s := NewSession(SessionConfig{
		MatchMedia: map[string]bool{"(prefers-reduced-motion: reduce)": true},
	})

	rules := s.MatchMediaRules()
	if got, want := rules["(prefers-reduced-motion: reduce)"], true; got != want {
		t.Fatalf("MatchMediaRules()[prefers-reduced-motion] = %v, want %v", got, want)
	}
	rules["(prefers-reduced-motion: reduce)"] = false
	if got, want := s.MatchMediaRules()["(prefers-reduced-motion: reduce)"], true; got != want {
		t.Fatalf("MatchMediaRules() reread = %v, want %v", got, want)
	}
}

func TestSessionReportsFetchCalls(t *testing.T) {
	s := NewSession(DefaultSessionConfig())
	s.Registry().Fetch().RespondText("https://example.test/api/message", 200, "ok")
	s.Registry().Fetch().Fail("https://example.test/api/broken", "boom")

	if _, _, _, err := s.Fetch("https://example.test/api/message"); err != nil {
		t.Fatalf("Fetch() error = %v", err)
	}
	if _, _, _, err := s.Fetch("https://example.test/api/broken"); err == nil {
		t.Fatalf("Fetch() broken error = nil, want failure")
	}

	calls := s.FetchCalls()
	if len(calls) != 2 || calls[0].URL != "https://example.test/api/message" || calls[1].URL != "https://example.test/api/broken" {
		t.Fatalf("FetchCalls() = %#v, want two captured requests", calls)
	}

	calls[0].URL = "mutated"
	if fresh := s.FetchCalls(); len(fresh) != 2 || fresh[0].URL != "https://example.test/api/message" || fresh[1].URL != "https://example.test/api/broken" {
		t.Fatalf("FetchCalls() reread = %#v, want original request", fresh)
	}

	responses := s.FetchResponseRules()
	if len(responses) != 1 || responses[0].URL != "https://example.test/api/message" || responses[0].Status != 200 || responses[0].Body != "ok" {
		t.Fatalf("FetchResponseRules() = %#v, want one response rule", responses)
	}
	responses[0].URL = "mutated"
	if fresh := s.FetchResponseRules(); len(fresh) != 1 || fresh[0].URL != "https://example.test/api/message" || fresh[0].Body != "ok" {
		t.Fatalf("FetchResponseRules() reread = %#v, want original response rule", fresh)
	}

	errors := s.FetchErrorRules()
	if len(errors) != 1 || errors[0].URL != "https://example.test/api/broken" || errors[0].Message != "boom" {
		t.Fatalf("FetchErrorRules() = %#v, want one error rule", errors)
	}
	errors[0].URL = "mutated"
	if fresh := s.FetchErrorRules(); len(fresh) != 1 || fresh[0].URL != "https://example.test/api/broken" || fresh[0].Message != "boom" {
		t.Fatalf("FetchErrorRules() reread = %#v, want original error rule", fresh)
	}
}

func TestSessionReportsActionCalls(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main></main>`,
		MatchMedia: map[string]bool{
			"(prefers-reduced-motion: reduce)": true,
		},
	})

	if err := s.Open("https://example.test/popup"); err != nil {
		t.Fatalf("Open() error = %v", err)
	}
	if err := s.Close(); err != nil {
		t.Fatalf("Close() error = %v", err)
	}
	if err := s.Print(); err != nil {
		t.Fatalf("Print() error = %v", err)
	}
	if err := s.ScrollTo(4, 5); err != nil {
		t.Fatalf("ScrollTo() error = %v", err)
	}
	if err := s.ScrollBy(2, -1); err != nil {
		t.Fatalf("ScrollBy() error = %v", err)
	}
	if _, err := s.MatchMedia("(prefers-reduced-motion: reduce)"); err != nil {
		t.Fatalf("MatchMedia() error = %v", err)
	}

	openCalls := s.OpenCalls()
	if len(openCalls) != 1 || openCalls[0].URL != "https://example.test/popup" {
		t.Fatalf("OpenCalls() = %#v, want one open call", openCalls)
	}
	openCalls[0].URL = "mutated"
	if fresh := s.OpenCalls(); len(fresh) != 1 || fresh[0].URL != "https://example.test/popup" {
		t.Fatalf("OpenCalls() reread = %#v, want original open call", fresh)
	}

	if closeCalls := s.CloseCalls(); len(closeCalls) != 1 {
		t.Fatalf("CloseCalls() = %#v, want one close call", closeCalls)
	}
	if printCalls := s.PrintCalls(); len(printCalls) != 1 {
		t.Fatalf("PrintCalls() = %#v, want one print call", printCalls)
	}
	scrollCalls := s.ScrollCalls()
	if len(scrollCalls) != 2 {
		t.Fatalf("ScrollCalls() = %#v, want two scroll calls", scrollCalls)
	}
	scrollCalls[0].X = 99
	if fresh := s.ScrollCalls(); len(fresh) != 2 || fresh[0].X != 4 || fresh[1].Method != "by" {
		t.Fatalf("ScrollCalls() reread = %#v, want original scroll calls", fresh)
	}
	if matchMediaCalls := s.MatchMediaCalls(); len(matchMediaCalls) != 1 || matchMediaCalls[0].Query != "(prefers-reduced-motion: reduce)" {
		t.Fatalf("MatchMediaCalls() = %#v, want one query call", matchMediaCalls)
	}
	if fresh := s.MatchMediaCalls(); len(fresh) != 1 || fresh[0].Query != "(prefers-reduced-motion: reduce)" {
		t.Fatalf("MatchMediaCalls() reread = %#v, want original query call", fresh)
	}

	s.Registry().MatchMedia().RecordListenerCall("(prefers-reduced-motion: reduce)", "change")
	listeners := s.MatchMediaListenerCalls()
	if len(listeners) != 1 || listeners[0].Query != "(prefers-reduced-motion: reduce)" || listeners[0].Method != "change" {
		t.Fatalf("MatchMediaListenerCalls() = %#v, want one listener call", listeners)
	}
	listeners[0].Method = "mutated"
	if fresh := s.MatchMediaListenerCalls(); len(fresh) != 1 || fresh[0].Method != "change" {
		t.Fatalf("MatchMediaListenerCalls() reread = %#v, want original listener call", fresh)
	}
}

func TestSessionReportsDialogMessages(t *testing.T) {
	s := NewSession(DefaultSessionConfig())
	s.Registry().Dialogs().QueueConfirm(true)
	s.Registry().Dialogs().QueuePromptText("Ada")

	if err := s.Alert("hello"); err != nil {
		t.Fatalf("Alert() error = %v", err)
	}
	if _, err := s.Confirm("Continue?"); err != nil {
		t.Fatalf("Confirm() error = %v", err)
	}
	if _, _, err := s.Prompt("Your name?"); err != nil {
		t.Fatalf("Prompt() error = %v", err)
	}

	alerts := s.DialogAlerts()
	if len(alerts) != 1 || alerts[0] != "hello" {
		t.Fatalf("DialogAlerts() = %#v, want one alert", alerts)
	}
	alerts[0] = "mutated"
	if fresh := s.DialogAlerts(); len(fresh) != 1 || fresh[0] != "hello" {
		t.Fatalf("DialogAlerts() reread = %#v, want original alert", fresh)
	}

	confirms := s.DialogConfirmMessages()
	if len(confirms) != 1 || confirms[0] != "Continue?" {
		t.Fatalf("DialogConfirmMessages() = %#v, want one confirm message", confirms)
	}
	confirms[0] = "mutated"
	if fresh := s.DialogConfirmMessages(); len(fresh) != 1 || fresh[0] != "Continue?" {
		t.Fatalf("DialogConfirmMessages() reread = %#v, want original confirm message", fresh)
	}

	prompts := s.DialogPromptMessages()
	if len(prompts) != 1 || prompts[0] != "Your name?" {
		t.Fatalf("DialogPromptMessages() = %#v, want one prompt message", prompts)
	}
	prompts[0] = "mutated"
	if fresh := s.DialogPromptMessages(); len(fresh) != 1 || fresh[0] != "Your name?" {
		t.Fatalf("DialogPromptMessages() reread = %#v, want original prompt message", fresh)
	}
}

func TestSessionReportsDownloadAndFileInputCaptures(t *testing.T) {
	s := NewSession(DefaultSessionConfig())

	if err := s.CaptureDownload("report.csv", []byte("downloaded bytes")); err != nil {
		t.Fatalf("CaptureDownload() error = %v", err)
	}
	if err := s.SetFiles("#upload", []string{"report.csv", "archive.zip"}); err != nil {
		t.Fatalf("SetFiles() error = %v", err)
	}

	artifacts := s.DownloadArtifacts()
	if len(artifacts) != 1 || artifacts[0].FileName != "report.csv" || string(artifacts[0].Bytes) != "downloaded bytes" {
		t.Fatalf("DownloadArtifacts() = %#v, want one download capture", artifacts)
	}
	artifacts[0].FileName = "mutated"
	artifacts[0].Bytes[0] = 'X'
	if fresh := s.DownloadArtifacts(); len(fresh) != 1 || fresh[0].FileName != "report.csv" || string(fresh[0].Bytes) != "downloaded bytes" {
		t.Fatalf("DownloadArtifacts() reread = %#v, want original capture", fresh)
	}

	selections := s.FileInputSelections()
	if len(selections) != 1 || selections[0].Selector != "#upload" || len(selections[0].Files) != 2 {
		t.Fatalf("FileInputSelections() = %#v, want one file-input selection", selections)
	}
	selections[0].Selector = "mutated"
	selections[0].Files[0] = "mutated"
	if fresh := s.FileInputSelections(); len(fresh) != 1 || fresh[0].Selector != "#upload" || fresh[0].Files[0] != "report.csv" {
		t.Fatalf("FileInputSelections() reread = %#v, want original selection", fresh)
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

func TestSessionUserValidityLifecycle(t *testing.T) {
	cfg := DefaultSessionConfig()
	cfg.HTML = `<main><form id="profile"><input id="name" type="text" required><input id="agree" type="checkbox" required checked><select id="mode" required><option value="a">A</option><option value="b" selected>B</option></select><button id="reset" type="reset">Reset</button></form></main>`
	s := NewSession(cfg)

	if err := s.TypeText("#name", "Ada"); err != nil {
		t.Fatalf("TypeText(#name) error = %v", err)
	}
	if err := s.SetChecked("#agree", false); err != nil {
		t.Fatalf("SetChecked(#agree) error = %v", err)
	}
	if err := s.SetSelectValue("#mode", "a"); err != nil {
		t.Fatalf("SetSelectValue(#mode) error = %v", err)
	}

	store, err := s.ensureDOM()
	if err != nil {
		t.Fatalf("ensureDOM() error = %v", err)
	}

	nameNodes, err := store.Select("#name")
	if err != nil {
		t.Fatalf("Select(#name) error = %v", err)
	}
	if len(nameNodes) != 1 {
		t.Fatalf("Select(#name) len = %d, want 1", len(nameNodes))
	}
	agreeNodes, err := store.Select("#agree")
	if err != nil {
		t.Fatalf("Select(#agree) error = %v", err)
	}
	if len(agreeNodes) != 1 {
		t.Fatalf("Select(#agree) len = %d, want 1", len(agreeNodes))
	}
	modeNodes, err := store.Select("#mode")
	if err != nil {
		t.Fatalf("Select(#mode) error = %v", err)
	}
	if len(modeNodes) != 1 {
		t.Fatalf("Select(#mode) len = %d, want 1", len(modeNodes))
	}

	if node := store.Node(nameNodes[0]); node == nil || !node.UserValidity {
		t.Fatalf("node(#name).UserValidity = %v, want true", node)
	}
	if node := store.Node(agreeNodes[0]); node == nil || !node.UserValidity {
		t.Fatalf("node(#agree).UserValidity = %v, want true", node)
	}
	if node := store.Node(modeNodes[0]); node == nil || !node.UserValidity {
		t.Fatalf("node(#mode).UserValidity = %v, want true", node)
	}

	if err := s.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) error = %v", err)
	}

	if node := store.Node(nameNodes[0]); node == nil || node.UserValidity {
		t.Fatalf("node(#name).UserValidity after reset = %v, want false", node)
	}
	if node := store.Node(agreeNodes[0]); node == nil || node.UserValidity {
		t.Fatalf("node(#agree).UserValidity after reset = %v, want false", node)
	}
	if node := store.Node(modeNodes[0]); node == nil || node.UserValidity {
		t.Fatalf("node(#mode).UserValidity after reset = %v, want false", node)
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
	if got, want := s.windowHistoryLength(), 2; got != want {
		t.Fatalf("windowHistoryLength() after Navigate = %d, want %d", got, want)
	}
	if got := s.Registry().Location().Navigations(); len(got) != 1 || got[0] != "https://example.test/next" {
		t.Fatalf("Location().Navigations() = %#v, want one navigation to example.test/next", got)
	}
	if gotX, gotY := s.scrollX, s.scrollY; gotX != 0 || gotY != 0 {
		t.Fatalf("scroll state after Navigate = (%d, %d), want (0, 0)", gotX, gotY)
	}
	if gotX, gotY := s.ScrollPosition(); gotX != 0 || gotY != 0 {
		t.Fatalf("ScrollPosition() after Navigate = (%d, %d), want (0, 0)", gotX, gotY)
	}
}

func TestSessionTracksTargetFromURLFragments(t *testing.T) {
	cfg := DefaultSessionConfig()
	cfg.URL = "https://example.test/page#legacy"
	cfg.HTML = `<main id="root"><a name="legacy">legacy</a><div id="space target">space</div><p id="tail">tail</p></main>`
	s := NewSession(cfg)

	store, err := s.ensureDOM()
	if err != nil {
		t.Fatalf("ensureDOM() error = %v", err)
	}

	legacyNodes, err := store.Select("a")
	if err != nil {
		t.Fatalf("Select(a) error = %v", err)
	}
	if len(legacyNodes) != 1 {
		t.Fatalf("Select(a) len = %d, want 1", len(legacyNodes))
	}
	spaceNodes, err := store.Select("div")
	if err != nil {
		t.Fatalf("Select(div) error = %v", err)
	}
	if len(spaceNodes) != 1 {
		t.Fatalf("Select(div) len = %d, want 1", len(spaceNodes))
	}
	legacyID := legacyNodes[0]
	spaceID := spaceNodes[0]

	if got := store.TargetNodeID(); got != legacyID {
		t.Fatalf("TargetNodeID() after bootstrap = %d, want %d", got, legacyID)
	}
	if err := s.AssertText("a:target", "legacy"); err != nil {
		t.Fatalf("AssertText(a:target) error = %v", err)
	}

	if err := s.ReplaceLocation("#space%20target"); err != nil {
		t.Fatalf("ReplaceLocation(#space%%20target) error = %v", err)
	}
	if got := store.TargetNodeID(); got != spaceID {
		t.Fatalf("TargetNodeID() after ReplaceLocation(#space%%20target) = %d, want %d", got, spaceID)
	}
	if err := s.AssertText("div:target", "space"); err != nil {
		t.Fatalf("AssertText(div:target) error = %v", err)
	}

	if err := s.Navigate("#missing"); err != nil {
		t.Fatalf("Navigate(#missing) error = %v", err)
	}
	if got := store.TargetNodeID(); got != 0 {
		t.Fatalf("TargetNodeID() after missing fragment = %d, want 0", got)
	}
	if err := s.AssertExists(":target"); err == nil {
		t.Fatalf("AssertExists(:target) after missing fragment error = nil, want no match")
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

func TestSessionClassListAndDatasetDelegatesToDOM(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><div id="root" class="alpha beta" data-foo-bar="1"></div></main>`,
	})

	classList, err := s.ClassList("#root")
	if err != nil {
		t.Fatalf("ClassList(#root) error = %v", err)
	}
	if got := classList.Values(); len(got) != 2 || got[0] != "alpha" || got[1] != "beta" {
		t.Fatalf("ClassList.Values() = %#v, want [alpha beta]", got)
	}
	if !classList.Contains("beta") {
		t.Fatalf("ClassList.Contains(beta) = false, want true")
	}

	classTokens := classList.Values()
	classTokens[0] = "mutated"
	if got := classList.Values(); got[0] != "alpha" {
		t.Fatalf("ClassList.Values() should return copies, got %#v", got)
	}

	if err := classList.Add("gamma"); err != nil {
		t.Fatalf("ClassList.Add(gamma) error = %v", err)
	}
	if err := classList.Remove("alpha"); err != nil {
		t.Fatalf("ClassList.Remove(alpha) error = %v", err)
	}

	dataset, err := s.Dataset("#root")
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

	if got, want := s.DumpDOM(), `<main><div id="root" class="beta gamma" data-ship-id="92432"></div></main>`; got != want {
		t.Fatalf("DumpDOM() after class/dataset view mutation = %q, want %q", got, want)
	}
}

func TestSessionClassListAndDatasetRejectInvalidTargets(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><div id="root"></div></main>`,
	})

	if _, err := s.ClassList("main[item="); err == nil {
		t.Fatalf("ClassList(main[item=) error = nil, want selector error")
	}
	if _, err := s.Dataset("#missing"); err == nil {
		t.Fatalf("Dataset(#missing) error = nil, want missing-element error")
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

func TestSessionInlineScriptsCanWriteHTMLMidScript(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><div id="out">old</div><script>host:writeHTML('<main><div id="out">new</div></main>'); host:setInnerHTML("#out", "after")</script></main>`,
	})

	if got, want := s.DumpDOM(), `<main><div id="out">after</div></main>`; got != want {
		t.Fatalf("DumpDOM() after writeHTML bootstrap = %q, want %q", got, want)
	}
	if got, want := s.HTML(), `<main><div id="out">new</div></main>`; got != want {
		t.Fatalf("HTML() after writeHTML bootstrap = %q, want %q", got, want)
	}
}

func TestSessionInlineScriptsCanDriveLocationMock(t *testing.T) {
	tests := []struct {
		name           string
		url            string
		html           string
		wantURL        string
		wantNavs       []string
		wantHistoryLen int
	}{
		{
			name:           "assign",
			url:            "https://example.test/start",
			html:           `<main><script>host:locationAssign("/assign")</script></main>`,
			wantURL:        "https://example.test/assign",
			wantHistoryLen: 2,
			wantNavs: []string{
				"https://example.test/assign",
			},
		},
		{
			name:           "replace",
			url:            "https://example.test/start",
			html:           `<main><script>host:locationReplace("replace")</script></main>`,
			wantURL:        "https://example.test/replace",
			wantHistoryLen: 1,
			wantNavs: []string{
				"https://example.test/replace",
			},
		},
		{
			name:           "reload",
			url:            "https://example.test/reload",
			html:           `<main><script>host:locationReload()</script></main>`,
			wantURL:        "https://example.test/reload",
			wantHistoryLen: 1,
			wantNavs: []string{
				"https://example.test/reload",
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			s := NewSession(SessionConfig{
				URL:  tc.url,
				HTML: tc.html,
			})

			if got := s.DumpDOM(); got == "" {
				t.Fatalf("DumpDOM() after location %s bootstrap = empty string, want DOM snapshot", tc.name)
			}
			if got, want := s.URL(), tc.wantURL; got != want {
				t.Fatalf("URL() after location %s bootstrap = %q, want %q", tc.name, got, want)
			}
			if got, want := s.windowHistoryLength(), tc.wantHistoryLen; got != want {
				t.Fatalf("windowHistoryLength() after location %s bootstrap = %d, want %d", tc.name, got, want)
			}
			if got := s.Registry().Location().Navigations(); len(got) != len(tc.wantNavs) {
				t.Fatalf("Location().Navigations() = %#v, want %#v", got, tc.wantNavs)
			} else {
				for i := range got {
					if got[i] != tc.wantNavs[i] {
						t.Fatalf("Location().Navigations()[%d] = %q, want %q", i, got[i], tc.wantNavs[i])
					}
				}
			}
		})
	}
}

func TestSessionInlineScriptsCanSetLocationProperties(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/start?old=1",
		HTML: `<main><script>host:locationSet("hash", "#next"); host:locationSet("pathname", "detail"); host:locationSet("search", "?mode=full")</script></main>`,
	})

	if got := s.DumpDOM(); got == "" {
		t.Fatalf("DumpDOM() after locationSet bootstrap = empty string, want DOM snapshot")
	}
	if got, want := s.URL(), "https://example.test/detail?mode=full#next"; got != want {
		t.Fatalf("URL() after locationSet bootstrap = %q, want %q", got, want)
	}
	if got, want := s.windowHistoryLength(), 4; got != want {
		t.Fatalf("windowHistoryLength() after locationSet bootstrap = %d, want %d", got, want)
	}
	if got := s.Registry().Location().Navigations(); len(got) != 3 || got[0] != "https://example.test/start?old=1#next" || got[1] != "https://example.test/detail?old=1#next" || got[2] != "https://example.test/detail?mode=full#next" {
		t.Fatalf("Location().Navigations() after locationSet bootstrap = %#v, want ordered property updates", got)
	}
}

func TestSessionInlineScriptsCanReadLocationProperties(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test:8443/path/name?mode=full#step-1",
		HTML: `<main><div id="href"></div><div id="origin"></div><div id="protocol"></div><div id="host"></div><div id="hostname"></div><div id="port"></div><div id="pathname"></div><div id="search"></div><div id="hash"></div><script>host:setTextContent("#href", expr(host:locationHref())); host:setTextContent("#origin", expr(host:locationOrigin())); host:setTextContent("#protocol", expr(host:locationProtocol())); host:setTextContent("#host", expr(host:locationHost())); host:setTextContent("#hostname", expr(host:locationHostname())); host:setTextContent("#port", expr(host:locationPort())); host:setTextContent("#pathname", expr(host:locationPathname())); host:setTextContent("#search", expr(host:locationSearch())); host:setTextContent("#hash", expr(host:locationHash()))</script></main>`,
	})

	tests := []struct {
		selector string
		want     string
	}{
		{selector: "#href", want: "https://example.test:8443/path/name?mode=full#step-1"},
		{selector: "#origin", want: "https://example.test:8443"},
		{selector: "#protocol", want: "https:"},
		{selector: "#host", want: "example.test:8443"},
		{selector: "#hostname", want: "example.test"},
		{selector: "#port", want: "8443"},
		{selector: "#pathname", want: "/path/name"},
		{selector: "#search", want: "?mode=full"},
		{selector: "#hash", want: "#step-1"},
	}

	for _, tc := range tests {
		got, err := s.TextContent(tc.selector)
		if err != nil {
			t.Fatalf("TextContent(%s) error = %v", tc.selector, err)
		}
		if got != tc.want {
			t.Fatalf("TextContent(%s) = %q, want %q", tc.selector, got, tc.want)
		}
	}
}

func TestSessionRejectsLocationGetterArgumentsFromInlineScript(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><div id="out">old</div><script>host:locationHref("extra")</script></main>`,
	})

	if _, err := s.ensureDOM(); err == nil {
		t.Fatalf("ensureDOM() error = nil, want location getter validation error")
	}
	if got := s.DumpDOM(); got != "" {
		t.Fatalf("DumpDOM() after rejected location getter args = %q, want empty string", got)
	}
}

func TestSessionInlineScriptsCanSetDocumentCookie(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/start",
		HTML: `<main><script>host:setDocumentCookie("theme=dark"); host:setDocumentCookie("lang=en; Path=/")</script></main>`,
	})

	if got := s.DumpDOM(); got == "" {
		t.Fatalf("DumpDOM() after documentCookie bootstrap = empty string, want DOM snapshot")
	}
	if got, want := s.documentCookie(), "lang=en; theme=dark"; got != want {
		t.Fatalf("documentCookie() after bootstrap = %q, want %q", got, want)
	}
	if got, want := s.navigatorCookieEnabled(), true; got != want {
		t.Fatalf("navigatorCookieEnabled() = %v, want %v", got, want)
	}
}

func TestSessionClipboardTracksSeedAndWrites(t *testing.T) {
	s := NewSession(DefaultSessionConfig())

	if got := s.Clipboard(); got != "" {
		t.Fatalf("Clipboard() without seed = %q, want empty", got)
	}

	if err := s.WriteClipboard("copied text"); err != nil {
		t.Fatalf("WriteClipboard() error = %v", err)
	}
	if got, want := s.Clipboard(), "copied text"; got != want {
		t.Fatalf("Clipboard() after write = %q, want %q", got, want)
	}

	writes := s.ClipboardWrites()
	if len(writes) != 1 || writes[0] != "copied text" {
		t.Fatalf("ClipboardWrites() = %#v, want one write", writes)
	}
	writes[0] = "mutated"
	if fresh := s.ClipboardWrites(); len(fresh) != 1 || fresh[0] != "copied text" {
		t.Fatalf("ClipboardWrites() reread = %#v, want original write", fresh)
	}
}

func TestSessionRejectsMalformedDocumentCookieFromInlineScript(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/start",
		HTML: `<main><script>host:setDocumentCookie("badcookie")</script></main>`,
	})

	if _, err := s.ensureDOM(); err == nil {
		t.Fatalf("ensureDOM() error = nil, want malformed cookie assignment error")
	}
	if got := s.documentCookie(); got != "" {
		t.Fatalf("documentCookie() after rejected bootstrap = %q, want empty string", got)
	}
}

func TestSessionRejectsUnsupportedLocationPropertyFromInlineScript(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/start",
		HTML: `<main><script>host:locationSet("origin", "https://example.test/other")</script></main>`,
	})

	if _, err := s.ensureDOM(); err == nil {
		t.Fatalf("ensureDOM() error = nil, want unsupported location property error")
	}
	if got := s.Registry().Location().Navigations(); len(got) != 0 {
		t.Fatalf("Location().Navigations() after rejected origin assignment = %#v, want empty", got)
	}
	if got, want := s.windowHistoryLength(), 1; got != want {
		t.Fatalf("windowHistoryLength() after rejected origin assignment = %d, want %d", got, want)
	}
	if got, want := s.URL(), "https://example.test/start"; got != want {
		t.Fatalf("URL() after rejected origin assignment = %q, want %q", got, want)
	}
}

func TestSessionRejectsEmptyLocationAssignmentFromInlineScript(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><script>host:locationAssign("")</script></main>`,
	})

	if got := s.DumpDOM(); got != "" {
		t.Fatalf("DumpDOM() after rejected location assignment = %q, want empty string", got)
	}
	if got, want := s.URL(), "https://app.local/"; got != want {
		t.Fatalf("URL() after rejected location assignment = %q, want %q", got, want)
	}
	if got := s.Registry().Location().Navigations(); len(got) != 0 {
		t.Fatalf("Location().Navigations() after rejected location assignment = %#v, want empty", got)
	}
	if got, want := s.windowHistoryLength(), 1; got != want {
		t.Fatalf("windowHistoryLength() after rejected location assignment = %d, want %d", got, want)
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

func TestSessionRunsQueuedMicrotasksDuringBootstrap(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><div id="out">start</div><script>host:queueMicrotask('host:setInnerHTML(#out, micro)')</script></main>`,
	})

	if got, want := s.DumpDOM(), `<main><div id="out">micro</div><script>host:queueMicrotask('host:setInnerHTML(#out, micro)')</script></main>`; got != want {
		t.Fatalf("DumpDOM() after queued bootstrap microtask = %q, want %q", got, want)
	}
}

func TestSessionClickRunsQueuedMicrotasksAfterDispatch(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><button id="btn">Go</button><div id="out">start</div><script>host:addEventListener("#btn", "click", "host:queueMicrotask('host:setInnerHTML(#out, micro)')")</script></main>`,
	})

	if err := s.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><button id="btn">Go</button><div id="out">micro</div><script>host:addEventListener("#btn", "click", "host:queueMicrotask('host:setInnerHTML(#out, micro)')")</script></main>`; got != want {
		t.Fatalf("DumpDOM() after queued click microtask = %q, want %q", got, want)
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
	if _, err := s.ClassList("#cta"); err == nil {
		t.Fatalf("ClassList(#cta) error = nil, want session unavailable error")
	}
	if _, err := s.Dataset("#cta"); err == nil {
		t.Fatalf("Dataset(#cta) error = nil, want session unavailable error")
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
	if err := s.Dispatch("#cta", "custom"); err == nil {
		t.Fatalf("Dispatch(#cta, custom) error = nil, want session unavailable error")
	}
	if err := s.DispatchKeyboard("#cta"); err == nil {
		t.Fatalf("DispatchKeyboard(#cta) error = nil, want session unavailable error")
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
	store, err := s.ensureDOM()
	if err != nil {
		t.Fatalf("ensureDOM() after Focus error = %v", err)
	}
	nameMatches, err := store.Select("#name")
	if err != nil {
		t.Fatalf("Select(#name) after Focus error = %v", err)
	}
	if len(nameMatches) != 1 {
		t.Fatalf("Select(#name) after Focus len = %d, want 1", len(nameMatches))
	}
	nameID := nameMatches[0]
	if got := store.FocusedNodeID(); got != nameID {
		t.Fatalf("FocusedNodeID() after Focus = %d, want %d", got, nameID)
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
	if got := store.FocusedNodeID(); got != 0 {
		t.Fatalf("FocusedNodeID() after Blur = %d, want 0", got)
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

	if err := s.Click("main[item="); err == nil {
		t.Fatalf("Click(main[item=) error = nil, want selector syntax error")
	}
	if err := s.Focus("#missing"); err == nil {
		t.Fatalf("Focus(#missing) error = nil, want missing-element error")
	}
	if err := s.Dispatch("#missing", "custom"); err == nil {
		t.Fatalf("Dispatch(#missing, custom) error = nil, want missing-element error")
	}
	if got := len(s.InteractionLog()); got != 0 {
		t.Fatalf("InteractionLog() len after rejected interactions = %d, want 0", got)
	}
	if got := s.FocusedSelector(); got != "" {
		t.Fatalf("FocusedSelector() after rejected interactions = %q, want empty", got)
	}
}

func TestSessionDispatchesCustomEventListeners(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"></div><script>host:addEventListener("#wrap", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>")', "capture"); host:addEventListener("#btn", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`,
	})

	if err := s.Dispatch("#btn", "custom"); err != nil {
		t.Fatalf("Dispatch(#btn, custom) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"><span>capture</span><span>target</span><span>bubble</span></div><script>host:addEventListener("#wrap", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>")', "capture"); host:addEventListener("#btn", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "custom", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`; got != want {
		t.Fatalf("DumpDOM() after custom dispatch = %q, want %q", got, want)
	}
}

func TestSessionDispatchRejectsBlankEventType(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><button id="btn">Go</button></main>`,
	})

	if err := s.Dispatch("#btn", " "); err == nil {
		t.Fatalf("Dispatch(#btn, blank) error = nil, want blank-event error")
	}
}

func TestSessionDispatchKeyboardFiresKeyboardEventSequence(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><button id="btn">Go</button><div id="log"></div><script>host:addEventListener("#btn", "keydown", 'host:insertAdjacentHTML("#log", "beforeend", "<span>down</span>")'); host:addEventListener("#btn", "keypress", 'host:insertAdjacentHTML("#log", "beforeend", "<span>press</span>")'); host:addEventListener("#btn", "keyup", 'host:insertAdjacentHTML("#log", "beforeend", "<span>up</span>")')</script></main>`,
	})

	if err := s.DispatchKeyboard("#btn"); err != nil {
		t.Fatalf("DispatchKeyboard(#btn) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><button id="btn">Go</button><div id="log"><span>down</span><span>press</span><span>up</span></div><script>host:addEventListener("#btn", "keydown", 'host:insertAdjacentHTML("#log", "beforeend", "<span>down</span>")'); host:addEventListener("#btn", "keypress", 'host:insertAdjacentHTML("#log", "beforeend", "<span>press</span>")'); host:addEventListener("#btn", "keyup", 'host:insertAdjacentHTML("#log", "beforeend", "<span>up</span>")')</script></main>`; got != want {
		t.Fatalf("DumpDOM() after keyboard dispatch = %q, want %q", got, want)
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

func TestSessionActionsSupportBoundedPseudoClasses(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><input id="enabled" type="text"><input id="flag" type="checkbox" checked><input id="off" type="text" disabled><div id="empty"></div><p id="last">two</p></main>`,
	})

	if err := s.Focus("input:first-child"); err != nil {
		t.Fatalf("Focus(input:first-child) error = %v", err)
	}
	if got, want := s.FocusedSelector(), "input:first-child"; got != want {
		t.Fatalf("FocusedSelector() = %q, want %q", got, want)
	}

	if err := s.Click("div:empty"); err != nil {
		t.Fatalf("Click(div:empty) error = %v", err)
	}

	log := s.InteractionLog()
	if len(log) != 2 {
		t.Fatalf("InteractionLog() len = %d, want 2", len(log))
	}
	if log[0].Kind != InteractionKindFocus || log[0].Selector != "input:first-child" {
		t.Fatalf("InteractionLog()[0] = %#v, want focus input:first-child", log[0])
	}
	if log[1].Kind != InteractionKindClick || log[1].Selector != "div:empty" {
		t.Fatalf("InteractionLog()[1] = %#v, want click div:empty", log[1])
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

func TestSessionDispatchesCaptureTargetAndBubbleListeners(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"></div><script>host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>")', "capture"); host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`,
	})

	if err := s.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"><span>capture</span><span>target</span><span>bubble</span></div><script>host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>")', "capture"); host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`; got != want {
		t.Fatalf("DumpDOM() after capture/target/bubble listeners = %q, want %q", got, want)
	}
}

func TestSessionFocusRemainsTargetOnly(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><section id="wrap"><input id="name"></section><div id="log"></div><script>host:addEventListener("#wrap", "focus", 'host:insertAdjacentHTML("#log", "beforeend", "<span>ancestor</span>")', "capture"); host:addEventListener("#name", "focus", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")')</script></main>`,
	})

	if err := s.Focus("#name"); err != nil {
		t.Fatalf("Focus(#name) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><section id="wrap"><input id="name"></section><div id="log"><span>target</span></div><script>host:addEventListener("#wrap", "focus", 'host:insertAdjacentHTML("#log", "beforeend", "<span>ancestor</span>")', "capture"); host:addEventListener("#name", "focus", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")')</script></main>`; got != want {
		t.Fatalf("DumpDOM() after focus target-only dispatch = %q, want %q", got, want)
	}
}

func TestSessionClickHonorsPreventDefaultFromListeners(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/base/",
		HTML: `<main><a id="nav" href="/next">Go</a><div id="out"></div><script>host:addEventListener("#nav", "click", 'host:preventDefault(); host:setInnerHTML("#out", "blocked")')</script></main>`,
	})

	if err := s.Click("#nav"); err != nil {
		t.Fatalf("Click(#nav) error = %v", err)
	}

	if got, want := s.URL(), "https://example.test/base/"; got != want {
		t.Fatalf("URL() after prevented click = %q, want %q", got, want)
	}
	if got := s.Registry().Location().Navigations(); len(got) != 0 {
		t.Fatalf("Location().Navigations() = %#v, want no navigation", got)
	}
	if got, want := s.DumpDOM(), `<main><a id="nav" href="/next">Go</a><div id="out">blocked</div><script>host:addEventListener("#nav", "click", 'host:preventDefault(); host:setInnerHTML("#out", "blocked")')</script></main>`; got != want {
		t.Fatalf("DumpDOM() after prevented click = %q, want %q", got, want)
	}
}

func TestSessionClickHonorsStopPropagationFromCaptureListeners(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"></div><script>host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>"); host:stopPropagation()', "capture"); host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`,
	})

	if err := s.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"><span>capture</span><span>target</span></div><script>host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>capture</span>"); host:stopPropagation()', "capture"); host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>target</span>")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`; got != want {
		t.Fatalf("DumpDOM() after stopPropagation click = %q, want %q", got, want)
	}
}

func TestSessionClickHonorsOnceListeners(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><button id="btn">Go</button><div id="log"></div><script>host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>once</span>")', "target", true)</script></main>`,
	})

	if err := s.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) first error = %v", err)
	}
	if err := s.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) second error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><button id="btn">Go</button><div id="log"><span>once</span></div><script>host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>once</span>")', "target", true)</script></main>`; got != want {
		t.Fatalf("DumpDOM() after once listener = %q, want %q", got, want)
	}
}

func TestSessionClickAllowsCaptureListenersToRemoveLaterTargetListeners(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"></div><script>host:addEventListener("#wrap", "click", 'host:removeEventListener("#btn", "click", host:removeNode("#btn")); host:insertAdjacentHTML("#log", "beforeend", "<span>remover</span>")', "capture"); host:addEventListener("#btn", "click", 'host:removeNode("#btn")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`,
	})

	if err := s.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><section id="wrap"><button id="btn">Go</button></section><div id="log"><span>remover</span><span>bubble</span></div><script>host:addEventListener("#wrap", "click", 'host:removeEventListener("#btn", "click", host:removeNode("#btn")); host:insertAdjacentHTML("#log", "beforeend", "<span>remover</span>")', "capture"); host:addEventListener("#btn", "click", 'host:removeNode("#btn")'); host:addEventListener("#wrap", "click", 'host:insertAdjacentHTML("#log", "beforeend", "<span>bubble</span>")', "bubble")</script></main>`; got != want {
		t.Fatalf("DumpDOM() after listener removal = %q, want %q", got, want)
	}
}

func TestSessionResetListenersCanPreventResetDefaultAction(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<form id="profile"><input id="name"><button id="reset" type="reset">Reset</button><div id="out"></div><script>host:addEventListener("#profile", "reset", 'host:preventDefault(); host:setInnerHTML("#out", "reset-blocked")')</script></form>`,
	})

	if err := s.TypeText("#name", "Ada"); err != nil {
		t.Fatalf("TypeText(#name) error = %v", err)
	}
	if err := s.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) error = %v", err)
	}

	if got, want := s.DumpDOM(), `<form id="profile"><input id="name" value="Ada"><button id="reset" type="reset">Reset</button><div id="out">reset-blocked</div><script>host:addEventListener("#profile", "reset", 'host:preventDefault(); host:setInnerHTML("#out", "reset-blocked")')</script></form>`; got != want {
		t.Fatalf("DumpDOM() after prevented reset = %q, want %q", got, want)
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
