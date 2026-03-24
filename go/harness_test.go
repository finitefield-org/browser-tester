package browsertester

import "testing"

func TestHarnessBuilderBuildsWithDefaults(t *testing.T) {
	harness, err := NewHarnessBuilder().Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}
	if got, want := harness.URL(), "https://app.local/"; got != want {
		t.Fatalf("URL() = %q, want %q", got, want)
	}
	if got, want := harness.NowMs(), int64(0); got != want {
		t.Fatalf("NowMs() = %d, want %d", got, want)
	}
	if got := harness.Debug().WindowName(); got != "" {
		t.Fatalf("Debug().WindowName() = %q, want empty", got)
	}
	if harness.Mocks().Fetch() == nil {
		t.Fatalf("Mocks().Fetch() = nil")
	}
	if got, want := harness.Mocks().Location().CurrentURL(), "https://app.local/"; got != want {
		t.Fatalf("Mocks().Location().CurrentURL() = %q, want %q", got, want)
	}
	if got := harness.Mocks().Storage().Local(); len(got) != 0 {
		t.Fatalf("Mocks().Storage().Local() = %#v, want empty map", got)
	}
}

func TestHarnessBuilderCopiesConfiguration(t *testing.T) {
	localStorage := map[string]string{"token": "abc"}
	sessionStorage := map[string]string{"tab": "main"}
	matchMedia := map[string]bool{"(prefers-reduced-motion: reduce)": true}

	harness, err := NewHarnessBuilder().
		URL("https://example.test/").
		HTML("<main>ok</main>").
		LocalStorage(localStorage).
		SessionStorage(sessionStorage).
		RandomSeed(42).
		MatchMedia(matchMedia).
		OpenFailure("blocked").
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	localStorage["token"] = "mutated"
	sessionStorage["tab"] = "mutated"
	matchMedia["(prefers-reduced-motion: reduce)"] = false

	if got, want := harness.URL(), "https://example.test/"; got != want {
		t.Fatalf("URL() = %q, want %q", got, want)
	}
	if got, want := harness.HTML(), "<main>ok</main>"; got != want {
		t.Fatalf("HTML() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().URL(), "https://example.test/"; got != want {
		t.Fatalf("Debug().URL() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().HTML(), "<main>ok</main>"; got != want {
		t.Fatalf("Debug().HTML() = %q, want %q", got, want)
	}
	if got := harness.Mocks().Dialogs(); got == nil {
		t.Fatalf("Mocks().Dialogs() = nil")
	}
	if got := harness.Mocks().MatchMedia(); got == nil {
		t.Fatalf("Mocks().MatchMedia() = nil")
	}
	if got, want := harness.Mocks().Location().CurrentURL(), "https://example.test/"; got != want {
		t.Fatalf("Mocks().Location().CurrentURL() = %q, want %q", got, want)
	}
	if got, want := harness.Mocks().Storage().Local()["token"], "abc"; got != want {
		t.Fatalf("Mocks().Storage().Local()[\"token\"] = %q, want %q", got, want)
	}
	if got, want := harness.Mocks().Storage().Session()["tab"], "main"; got != want {
		t.Fatalf("Mocks().Storage().Session()[\"tab\"] = %q, want %q", got, want)
	}
}

func TestFromHTMLHelpers(t *testing.T) {
	harness, err := FromHTMLWithURLAndSessionStorage(
		"https://example.test/",
		"<body>hi</body>",
		map[string]string{"seen": "yes"},
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURLAndSessionStorage() error = %v", err)
	}
	if got, want := harness.URL(), "https://example.test/"; got != want {
		t.Fatalf("URL() = %q, want %q", got, want)
	}
	if got, want := harness.HTML(), "<body>hi</body>"; got != want {
		t.Fatalf("HTML() = %q, want %q", got, want)
	}
}

func TestHarnessActionsRouteThroughMockFamilies(t *testing.T) {
	harness, err := FromHTML("<main></main>")
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	harness.Mocks().Fetch().RespondText("https://example.test/api/message", 200, "ok")
	harness.Mocks().Dialogs().QueueConfirm(true)
	harness.Mocks().Dialogs().QueuePromptText("typed answer")
	harness.Mocks().Clipboard().SeedText("seeded text")
	harness.Mocks().MatchMedia().RespondMatches("(prefers-reduced-motion: reduce)", true)

	if err := harness.Alert("hello"); err != nil {
		t.Fatalf("Alert() error = %v", err)
	}
	confirmed, err := harness.Confirm("Continue?")
	if err != nil {
		t.Fatalf("Confirm() error = %v", err)
	}
	if !confirmed {
		t.Fatalf("Confirm() = %v, want true", confirmed)
	}
	prompted, submitted, err := harness.Prompt("Your name?")
	if err != nil {
		t.Fatalf("Prompt() error = %v", err)
	}
	if prompted != "typed answer" || !submitted {
		t.Fatalf("Prompt() = (%q, %v), want (%q, true)", prompted, submitted, "typed answer")
	}

	resp, err := harness.Fetch("https://example.test/api/message")
	if err != nil {
		t.Fatalf("Fetch() error = %v", err)
	}
	if got, want := resp.URL, "https://example.test/api/message"; got != want {
		t.Fatalf("Fetch() URL = %q, want %q", got, want)
	}
	if got, want := resp.Status, 200; got != want {
		t.Fatalf("Fetch() Status = %d, want %d", got, want)
	}
	if got, want := resp.Body, "ok"; got != want {
		t.Fatalf("Fetch() Body = %q, want %q", got, want)
	}

	if err := harness.Open("https://example.test/new"); err != nil {
		t.Fatalf("Open() error = %v", err)
	}
	if err := harness.Close(); err != nil {
		t.Fatalf("Close() error = %v", err)
	}
	if err := harness.Print(); err != nil {
		t.Fatalf("Print() error = %v", err)
	}
	if err := harness.ScrollTo(10, 20); err != nil {
		t.Fatalf("ScrollTo() error = %v", err)
	}
	if err := harness.ScrollBy(-2, 3); err != nil {
		t.Fatalf("ScrollBy() error = %v", err)
	}
	if err := harness.Navigate("https://example.test/next"); err != nil {
		t.Fatalf("Navigate() error = %v", err)
	}
	if got, want := harness.URL(), "https://example.test/next"; got != want {
		t.Fatalf("URL() after Navigate() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().URL(), "https://example.test/next"; got != want {
		t.Fatalf("Debug().URL() after Navigate() = %q, want %q", got, want)
	}
	if err := harness.Navigate("relative"); err != nil {
		t.Fatalf("Navigate(relative) error = %v", err)
	}
	if got, want := harness.URL(), "https://example.test/relative"; got != want {
		t.Fatalf("URL() after relative Navigate() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().URL(), "https://example.test/relative"; got != want {
		t.Fatalf("Debug().URL() after relative Navigate() = %q, want %q", got, want)
	}
	if matches, err := harness.MatchMedia("(prefers-reduced-motion: reduce)"); err != nil || !matches {
		t.Fatalf("MatchMedia() = (%v, %v), want (true, nil)", matches, err)
	}
	if err := harness.CaptureDownload("report.csv", []byte("downloaded bytes")); err != nil {
		t.Fatalf("CaptureDownload() error = %v", err)
	}
	if err := harness.SetFiles("#upload", []string{"report.csv"}); err != nil {
		t.Fatalf("SetFiles() error = %v", err)
	}

	seeded, ok := harness.Mocks().Clipboard().SeededText()
	if !ok {
		t.Fatalf("Clipboard().SeededText() ok = false, want true")
	}
	if got, want := seeded, "seeded text"; got != want {
		t.Fatalf("Clipboard().SeededText() = %q, want %q", got, want)
	}
	got, err := harness.ReadClipboard()
	if err != nil {
		t.Fatalf("ReadClipboard() error = %v", err)
	}
	if got != "seeded text" {
		t.Fatalf("ReadClipboard() = %q, want %q", got, "seeded text")
	}

	if err := harness.WriteClipboard("copied text"); err != nil {
		t.Fatalf("WriteClipboard() error = %v", err)
	}
	got, err = harness.ReadClipboard()
	if err != nil {
		t.Fatalf("ReadClipboard() after write error = %v", err)
	}
	if got != "copied text" {
		t.Fatalf("ReadClipboard() after write = %q, want %q", got, "copied text")
	}

	writes := harness.Mocks().Clipboard().Writes()
	if len(writes) != 1 || writes[0] != "copied text" {
		t.Fatalf("Writes() = %#v, want [\"copied text\"]", writes)
	}

	if got := harness.Mocks().Fetch().Calls(); len(got) != 1 || got[0].URL != "https://example.test/api/message" {
		t.Fatalf("Fetch().Calls() = %#v, want one call to example test URL", got)
	}
	if got := harness.Mocks().Dialogs().Alerts(); len(got) != 1 || got[0] != "hello" {
		t.Fatalf("Dialogs().Alerts() = %#v, want [\"hello\"]", got)
	}
	if got := harness.Mocks().Dialogs().ConfirmMessages(); len(got) != 1 || got[0] != "Continue?" {
		t.Fatalf("Dialogs().ConfirmMessages() = %#v, want [\"Continue?\"]", got)
	}
	if got := harness.Mocks().Dialogs().PromptMessages(); len(got) != 1 || got[0] != "Your name?" {
		t.Fatalf("Dialogs().PromptMessages() = %#v, want [\"Your name?\"]", got)
	}
	if got := harness.Mocks().Open().Calls(); len(got) != 1 || got[0].URL != "https://example.test/new" {
		t.Fatalf("Open().Calls() = %#v, want one open call", got)
	}
	if got := harness.Mocks().Close().Calls(); len(got) != 1 {
		t.Fatalf("Close().Calls() = %#v, want one close call", got)
	}
	if got := harness.Mocks().Print().Calls(); len(got) != 1 {
		t.Fatalf("Print().Calls() = %#v, want one print call", got)
	}
	if got := harness.Mocks().Scroll().Calls(); len(got) != 2 || got[0].Method != ScrollMethodTo || got[1].Method != ScrollMethodBy {
		t.Fatalf("Scroll().Calls() = %#v, want to/by calls", got)
	}
	if got := harness.Mocks().MatchMedia().Calls(); len(got) != 1 || got[0].Query != "(prefers-reduced-motion: reduce)" {
		t.Fatalf("MatchMedia().Calls() = %#v, want one query call", got)
	}
	if got := harness.Mocks().Location().Navigations(); len(got) != 2 || got[0] != "https://example.test/next" || got[1] != "https://example.test/relative" {
		t.Fatalf("Location().Navigations() = %#v, want [https://example.test/next https://example.test/relative]", got)
	}
	if got := harness.Mocks().Downloads().Artifacts(); len(got) != 1 || got[0].FileName != "report.csv" {
		t.Fatalf("Downloads().Artifacts() = %#v, want one artifact", got)
	}
	if got := harness.Mocks().FileInput().Selections(); len(got) != 1 || got[0].Selector != "#upload" {
		t.Fatalf("FileInput().Selections() = %#v, want one selection", got)
	}
}

func TestHarnessFailurePathsAreReported(t *testing.T) {
	harness, err := NewHarnessBuilder().
		OpenFailure("open blocked").
		CloseFailure("close blocked").
		PrintFailure("print blocked").
		ScrollFailure("scroll blocked").
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	if _, err := harness.Fetch("https://example.test/missing"); err == nil {
		t.Fatalf("Fetch() error = nil, want missing mock error")
	}
	if _, err := harness.Confirm("Continue?"); err == nil {
		t.Fatalf("Confirm() error = nil, want queued response error")
	}
	if _, _, err := harness.Prompt("Continue?"); err == nil {
		t.Fatalf("Prompt() error = nil, want queued response error")
	}
	if err := harness.Open("https://example.test/blocked"); err == nil {
		t.Fatalf("Open() error = nil, want failure seed")
	}
	if err := harness.Close(); err == nil {
		t.Fatalf("Close() error = nil, want failure seed")
	}
	if err := harness.Print(); err == nil {
		t.Fatalf("Print() error = nil, want failure seed")
	}
	if err := harness.ScrollTo(1, 2); err == nil {
		t.Fatalf("ScrollTo() error = nil, want failure seed")
	}
	if err := harness.ScrollBy(1, 2); err == nil {
		t.Fatalf("ScrollBy() error = nil, want failure seed")
	}
	if err := harness.Navigate(""); err == nil {
		t.Fatalf("Navigate() error = nil, want empty URL validation")
	}
	if err := harness.CaptureDownload("", []byte("x")); err == nil {
		t.Fatalf("CaptureDownload() error = nil, want empty file name validation")
	}
	unseededHarness, err := FromHTML("<main></main>")
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}
	if _, err := unseededHarness.ReadClipboard(); err == nil {
		t.Fatalf("ReadClipboard() error = nil, want unseeded clipboard error")
	}
}

func TestHarnessWriteHTMLRoutesThroughRuntime(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out">old</div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.WriteHTML(`<main><div id="out">new</div></main>`); err != nil {
		t.Fatalf("WriteHTML() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">new</div></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after WriteHTML = %q, want %q", got, want)
	}
	if got, want := harness.HTML(), `<main><div id="out">new</div></main>`; got != want {
		t.Fatalf("HTML() after WriteHTML = %q, want %q", got, want)
	}
}
