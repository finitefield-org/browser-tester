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
	if got, want := harness.Debug().LocationOrigin(), "https://app.local"; got != want {
		t.Fatalf("Debug().LocationOrigin() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationProtocol(), "https:"; got != want {
		t.Fatalf("Debug().LocationProtocol() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationHost(), "app.local"; got != want {
		t.Fatalf("Debug().LocationHost() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationHostname(), "app.local"; got != want {
		t.Fatalf("Debug().LocationHostname() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationPort(), ""; got != want {
		t.Fatalf("Debug().LocationPort() = %q, want empty", got)
	}
	if got, want := harness.Debug().LocationPathname(), "/"; got != want {
		t.Fatalf("Debug().LocationPathname() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationSearch(), ""; got != want {
		t.Fatalf("Debug().LocationSearch() = %q, want empty", got)
	}
	if got, want := harness.Debug().LocationHash(), ""; got != want {
		t.Fatalf("Debug().LocationHash() = %q, want empty", got)
	}
	if got := harness.Debug().DOMReady(); got {
		t.Fatalf("Debug().DOMReady() = %v, want false before DOM access", got)
	}
	if got := harness.Debug().DOMError(); got != "" {
		t.Fatalf("Debug().DOMError() = %q, want empty", got)
	}
	if got := harness.Debug().FocusedNodeID(); got != 0 {
		t.Fatalf("Debug().FocusedNodeID() = %d, want zero", got)
	}
	if got := harness.Debug().ImageCount(); got != 0 {
		t.Fatalf("Debug().ImageCount() = %d, want zero", got)
	}
	if got := harness.Debug().FormCount(); got != 0 {
		t.Fatalf("Debug().FormCount() = %d, want zero", got)
	}
	if got := harness.Debug().SelectCount(); got != 0 {
		t.Fatalf("Debug().SelectCount() = %d, want zero", got)
	}
	if got := harness.Debug().TemplateCount(); got != 0 {
		t.Fatalf("Debug().TemplateCount() = %d, want zero", got)
	}
	if got := harness.Debug().TableCount(); got != 0 {
		t.Fatalf("Debug().TableCount() = %d, want zero", got)
	}
	if got := harness.Debug().ButtonCount(); got != 0 {
		t.Fatalf("Debug().ButtonCount() = %d, want zero", got)
	}
	if got := harness.Debug().TextAreaCount(); got != 0 {
		t.Fatalf("Debug().TextAreaCount() = %d, want zero", got)
	}
	if got := harness.Debug().InputCount(); got != 0 {
		t.Fatalf("Debug().InputCount() = %d, want zero", got)
	}
	if got := harness.Debug().FieldsetCount(); got != 0 {
		t.Fatalf("Debug().FieldsetCount() = %d, want zero", got)
	}
	if got := harness.Debug().LegendCount(); got != 0 {
		t.Fatalf("Debug().LegendCount() = %d, want zero", got)
	}
	if got := harness.Debug().OutputCount(); got != 0 {
		t.Fatalf("Debug().OutputCount() = %d, want zero", got)
	}
	if got := harness.Debug().LabelCount(); got != 0 {
		t.Fatalf("Debug().LabelCount() = %d, want zero", got)
	}
	if got := harness.Debug().ProgressCount(); got != 0 {
		t.Fatalf("Debug().ProgressCount() = %d, want zero", got)
	}
	if got := harness.Debug().MeterCount(); got != 0 {
		t.Fatalf("Debug().MeterCount() = %d, want zero", got)
	}
	if got := harness.Debug().AudioCount(); got != 0 {
		t.Fatalf("Debug().AudioCount() = %d, want zero", got)
	}
	if got := harness.Debug().VideoCount(); got != 0 {
		t.Fatalf("Debug().VideoCount() = %d, want zero", got)
	}
	if got := harness.Debug().IframeCount(); got != 0 {
		t.Fatalf("Debug().IframeCount() = %d, want zero", got)
	}
	if got := harness.Debug().EmbedCount(); got != 0 {
		t.Fatalf("Debug().EmbedCount() = %d, want zero", got)
	}
	if got := harness.Debug().TrackCount(); got != 0 {
		t.Fatalf("Debug().TrackCount() = %d, want zero", got)
	}
	if got := harness.Debug().PictureCount(); got != 0 {
		t.Fatalf("Debug().PictureCount() = %d, want zero", got)
	}
	if got := harness.Debug().SourceCount(); got != 0 {
		t.Fatalf("Debug().SourceCount() = %d, want zero", got)
	}
	if got := harness.Debug().DialogCount(); got != 0 {
		t.Fatalf("Debug().DialogCount() = %d, want zero", got)
	}
	if got := harness.Debug().DetailsCount(); got != 0 {
		t.Fatalf("Debug().DetailsCount() = %d, want zero", got)
	}
	if got := harness.Debug().SummaryCount(); got != 0 {
		t.Fatalf("Debug().SummaryCount() = %d, want zero", got)
	}
	if got := harness.Debug().SectionCount(); got != 0 {
		t.Fatalf("Debug().SectionCount() = %d, want zero", got)
	}
	if got := harness.Debug().MainCount(); got != 0 {
		t.Fatalf("Debug().MainCount() = %d, want zero", got)
	}
	if got := harness.Debug().ArticleCount(); got != 0 {
		t.Fatalf("Debug().ArticleCount() = %d, want zero", got)
	}
	if got := harness.Debug().NavCount(); got != 0 {
		t.Fatalf("Debug().NavCount() = %d, want zero", got)
	}
	if got := harness.Debug().AsideCount(); got != 0 {
		t.Fatalf("Debug().AsideCount() = %d, want zero", got)
	}
	if got := harness.Debug().FigureCount(); got != 0 {
		t.Fatalf("Debug().FigureCount() = %d, want zero", got)
	}
	if got := harness.Debug().FigcaptionCount(); got != 0 {
		t.Fatalf("Debug().FigcaptionCount() = %d, want zero", got)
	}
	if got := harness.Debug().HeaderCount(); got != 0 {
		t.Fatalf("Debug().HeaderCount() = %d, want zero", got)
	}
	if got := harness.Debug().FooterCount(); got != 0 {
		t.Fatalf("Debug().FooterCount() = %d, want zero", got)
	}
	if got := harness.Debug().AddressCount(); got != 0 {
		t.Fatalf("Debug().AddressCount() = %d, want zero", got)
	}
	if got := harness.Debug().BlockquoteCount(); got != 0 {
		t.Fatalf("Debug().BlockquoteCount() = %d, want zero", got)
	}
	if got := harness.Debug().ParagraphCount(); got != 0 {
		t.Fatalf("Debug().ParagraphCount() = %d, want zero", got)
	}
	if got := harness.Debug().PreCount(); got != 0 {
		t.Fatalf("Debug().PreCount() = %d, want zero", got)
	}
	if got := harness.Debug().MarkCount(); got != 0 {
		t.Fatalf("Debug().MarkCount() = %d, want zero", got)
	}
	if got := harness.Debug().QCount(); got != 0 {
		t.Fatalf("Debug().QCount() = %d, want zero", got)
	}
	if got := harness.Debug().CiteCount(); got != 0 {
		t.Fatalf("Debug().CiteCount() = %d, want zero", got)
	}
	if got := harness.Debug().AbbrCount(); got != 0 {
		t.Fatalf("Debug().AbbrCount() = %d, want zero", got)
	}
	if got := harness.Debug().StrongCount(); got != 0 {
		t.Fatalf("Debug().StrongCount() = %d, want zero", got)
	}
	if got := harness.Debug().SpanCount(); got != 0 {
		t.Fatalf("Debug().SpanCount() = %d, want zero", got)
	}
	if got := harness.Debug().DataCount(); got != 0 {
		t.Fatalf("Debug().DataCount() = %d, want zero", got)
	}
	if got := harness.Debug().DfnCount(); got != 0 {
		t.Fatalf("Debug().DfnCount() = %d, want zero", got)
	}
	if got := harness.Debug().KbdCount(); got != 0 {
		t.Fatalf("Debug().KbdCount() = %d, want zero", got)
	}
	if got := harness.Debug().SampCount(); got != 0 {
		t.Fatalf("Debug().SampCount() = %d, want zero", got)
	}
	if got := harness.Debug().RubyCount(); got != 0 {
		t.Fatalf("Debug().RubyCount() = %d, want zero", got)
	}
	if got := harness.Debug().RtCount(); got != 0 {
		t.Fatalf("Debug().RtCount() = %d, want zero", got)
	}
	if got := harness.Debug().VarCount(); got != 0 {
		t.Fatalf("Debug().VarCount() = %d, want zero", got)
	}
	if got := harness.Debug().CodeCount(); got != 0 {
		t.Fatalf("Debug().CodeCount() = %d, want zero", got)
	}
	if got := harness.Debug().SmallCount(); got != 0 {
		t.Fatalf("Debug().SmallCount() = %d, want zero", got)
	}
	if got := harness.Debug().TimeCount(); got != 0 {
		t.Fatalf("Debug().TimeCount() = %d, want zero", got)
	}
	if got := harness.Debug().OptionCount(); got != 0 {
		t.Fatalf("Debug().OptionCount() = %d, want zero", got)
	}
	if got := harness.Debug().SelectedOptionCount(); got != 0 {
		t.Fatalf("Debug().SelectedOptionCount() = %d, want zero", got)
	}
	if got := harness.Debug().OptgroupCount(); got != 0 {
		t.Fatalf("Debug().OptgroupCount() = %d, want zero", got)
	}
	if got := harness.Debug().LinkCount(); got != 0 {
		t.Fatalf("Debug().LinkCount() = %d, want zero", got)
	}
	if got := harness.Debug().AnchorCount(); got != 0 {
		t.Fatalf("Debug().AnchorCount() = %d, want zero", got)
	}
	if got := harness.Debug().OptionLabels(); len(got) != 0 {
		t.Fatalf("Debug().OptionLabels() = %#v, want empty", got)
	}
	if got := harness.Debug().SelectedOptionLabels(); len(got) != 0 {
		t.Fatalf("Debug().SelectedOptionLabels() = %#v, want empty", got)
	}
	if got := harness.Debug().OptionValues(); len(got) != 0 {
		t.Fatalf("Debug().OptionValues() = %#v, want empty", got)
	}
	if got := harness.Debug().SelectedOptionValues(); len(got) != 0 {
		t.Fatalf("Debug().SelectedOptionValues() = %#v, want empty", got)
	}
	if got := harness.Debug().OptgroupLabels(); len(got) != 0 {
		t.Fatalf("Debug().OptgroupLabels() = %#v, want empty", got)
	}
	if got, want := harness.NowMs(), int64(0); got != want {
		t.Fatalf("NowMs() = %d, want %d", got, want)
	}
	emptyDOMHarness, err := NewHarnessBuilder().Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}
	if got := emptyDOMHarness.Debug().LastInlineScriptHTML(); got != "" {
		t.Fatalf("Debug().LastInlineScriptHTML() = %q, want empty", got)
	}
	if got := harness.Debug().WindowName(); got != "" {
		t.Fatalf("Debug().WindowName() = %q, want empty", got)
	}
	if got := harness.Debug().DocumentCookie(); got != "" {
		t.Fatalf("Debug().DocumentCookie() = %q, want empty", got)
	}
	if got := harness.Debug().CookieJar(); len(got) != 0 {
		t.Fatalf("Debug().CookieJar() = %#v, want empty", got)
	}
	if got := harness.Debug().Clipboard(); got != "" {
		t.Fatalf("Debug().Clipboard() = %q, want empty", got)
	}
	if got := harness.Debug().ClipboardWrites(); len(got) != 0 {
		t.Fatalf("Debug().ClipboardWrites() = %#v, want empty", got)
	}
	if got := harness.Debug().FetchResponseRules(); len(got) != 0 {
		t.Fatalf("Debug().FetchResponseRules() = %#v, want empty", got)
	}
	if got := harness.Debug().FetchErrorRules(); len(got) != 0 {
		t.Fatalf("Debug().FetchErrorRules() = %#v, want empty", got)
	}
	if got := harness.Debug().DialogAlerts(); len(got) != 0 {
		t.Fatalf("Debug().DialogAlerts() = %#v, want empty", got)
	}
	if got := harness.Debug().DialogConfirmMessages(); len(got) != 0 {
		t.Fatalf("Debug().DialogConfirmMessages() = %#v, want empty", got)
	}
	if got := harness.Debug().DialogPromptMessages(); len(got) != 0 {
		t.Fatalf("Debug().DialogPromptMessages() = %#v, want empty", got)
	}
	if got := harness.Debug().DownloadArtifacts(); len(got) != 0 {
		t.Fatalf("Debug().DownloadArtifacts() = %#v, want empty", got)
	}
	if got := harness.Debug().FileInputSelections(); len(got) != 0 {
		t.Fatalf("Debug().FileInputSelections() = %#v, want empty", got)
	}
	if got := harness.Debug().StorageEvents(); len(got) != 0 {
		t.Fatalf("Debug().StorageEvents() = %#v, want empty", got)
	}
	if got := harness.Debug().OpenCalls(); len(got) != 0 {
		t.Fatalf("Debug().OpenCalls() = %#v, want empty", got)
	}
	if got := harness.Debug().CloseCalls(); len(got) != 0 {
		t.Fatalf("Debug().CloseCalls() = %#v, want empty", got)
	}
	if got := harness.Debug().PrintCalls(); len(got) != 0 {
		t.Fatalf("Debug().PrintCalls() = %#v, want empty", got)
	}
	if got := harness.Debug().ScrollCalls(); len(got) != 0 {
		t.Fatalf("Debug().ScrollCalls() = %#v, want empty", got)
	}
	if got := harness.Debug().MatchMediaCalls(); len(got) != 0 {
		t.Fatalf("Debug().MatchMediaCalls() = %#v, want empty", got)
	}
	if got := harness.Debug().MatchMediaListenerCalls(); len(got) != 0 {
		t.Fatalf("Debug().MatchMediaListenerCalls() = %#v, want empty", got)
	}
	if got := harness.Debug().EventListeners(); len(got) != 0 {
		t.Fatalf("Debug().EventListeners() = %#v, want empty", got)
	}
	if got := harness.Debug().MatchMediaRules(); len(got) != 0 {
		t.Fatalf("Debug().MatchMediaRules() = %#v, want empty", got)
	}
	if got := harness.Debug().OpenFailure(); got != "" {
		t.Fatalf("Debug().OpenFailure() = %q, want empty", got)
	}
	if got := harness.Debug().CloseFailure(); got != "" {
		t.Fatalf("Debug().CloseFailure() = %q, want empty", got)
	}
	if got := harness.Debug().PrintFailure(); got != "" {
		t.Fatalf("Debug().PrintFailure() = %q, want empty", got)
	}
	if got := harness.Debug().ScrollFailure(); got != "" {
		t.Fatalf("Debug().ScrollFailure() = %q, want empty", got)
	}
	if got := harness.Debug().LocalStorage(); len(got) != 0 {
		t.Fatalf("Debug().LocalStorage() = %#v, want empty", got)
	}
	if got := harness.Debug().SessionStorage(); len(got) != 0 {
		t.Fatalf("Debug().SessionStorage() = %#v, want empty", got)
	}
	if got := harness.Debug().TargetNodeID(); got != 0 {
		t.Fatalf("Debug().TargetNodeID() = %d, want zero", got)
	}
	if got := harness.Debug().HistoryLength(); got != 1 {
		t.Fatalf("Debug().HistoryLength() = %d, want 1", got)
	}
	if got, ok := harness.Debug().HistoryState(); ok || got != "null" {
		t.Fatalf("Debug().HistoryState() = (%q, %v), want (\"null\", false)", got, ok)
	}
	if got := harness.Debug().HistoryEntries(); len(got) != 1 || got[0].URL != "https://app.local/" || got[0].HasState {
		t.Fatalf("Debug().HistoryEntries() = %#v, want one initial entry without state", got)
	}
	if got := harness.Debug().HistoryIndex(); got != 0 {
		t.Fatalf("Debug().HistoryIndex() = %d, want zero", got)
	}
	if got := harness.Debug().VisitedURLs(); len(got) != 1 || got[0] != "https://app.local/" {
		t.Fatalf("Debug().VisitedURLs() = %#v, want one current URL snapshot", got)
	}
	if got := harness.Debug().PendingTimers(); len(got) != 0 {
		t.Fatalf("Debug().PendingTimers() = %#v, want empty", got)
	}
	if got := harness.Debug().PendingAnimationFrames(); len(got) != 0 {
		t.Fatalf("Debug().PendingAnimationFrames() = %#v, want empty", got)
	}
	if got := harness.Debug().PendingMicrotasks(); len(got) != 0 {
		t.Fatalf("Debug().PendingMicrotasks() = %#v, want empty", got)
	}
	if got := harness.Debug().HistoryScrollRestoration(); got != "auto" {
		t.Fatalf("Debug().HistoryScrollRestoration() = %q, want %q", got, "auto")
	}
	if got := harness.Debug().NavigationLog(); len(got) != 0 {
		t.Fatalf("Debug().NavigationLog() = %#v, want empty", got)
	}
	if got := harness.Debug().FetchCalls(); len(got) != 0 {
		t.Fatalf("Debug().FetchCalls() = %#v, want empty", got)
	}
	if harness.Mocks().Fetch() == nil {
		t.Fatalf("Mocks().Fetch() = nil")
	}
	if got := harness.Mocks().MatchMedia().Rules(); len(got) != 0 {
		t.Fatalf("Mocks().MatchMedia().Rules() = %#v, want empty", got)
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
	if got, want := harness.Debug().InitialHTML(), "<main>ok</main>"; got != want {
		t.Fatalf("Debug().InitialHTML() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().URL(), "https://example.test/"; got != want {
		t.Fatalf("Debug().URL() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationOrigin(), "https://example.test"; got != want {
		t.Fatalf("Debug().LocationOrigin() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationProtocol(), "https:"; got != want {
		t.Fatalf("Debug().LocationProtocol() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationHost(), "example.test"; got != want {
		t.Fatalf("Debug().LocationHost() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationHostname(), "example.test"; got != want {
		t.Fatalf("Debug().LocationHostname() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationPort(), ""; got != want {
		t.Fatalf("Debug().LocationPort() = %q, want empty", got)
	}
	if got, want := harness.Debug().LocationPathname(), "/"; got != want {
		t.Fatalf("Debug().LocationPathname() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationSearch(), ""; got != want {
		t.Fatalf("Debug().LocationSearch() = %q, want empty", got)
	}
	if got, want := harness.Debug().LocationHash(), ""; got != want {
		t.Fatalf("Debug().LocationHash() = %q, want empty", got)
	}
	if got := harness.Debug().MatchMediaRules(); len(got) != 1 || !got["(prefers-reduced-motion: reduce)"] {
		t.Fatalf("Debug().MatchMediaRules() = %#v, want seeded rule", got)
	}
	if got, want := harness.Debug().OpenFailure(), "blocked"; got != want {
		t.Fatalf("Debug().OpenFailure() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().CloseFailure(), ""; got != want {
		t.Fatalf("Debug().CloseFailure() = %q, want empty", got)
	}
	if got, want := harness.Debug().PrintFailure(), ""; got != want {
		t.Fatalf("Debug().PrintFailure() = %q, want empty", got)
	}
	if got, want := harness.Debug().ScrollFailure(), ""; got != want {
		t.Fatalf("Debug().ScrollFailure() = %q, want empty", got)
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
	if got := harness.Mocks().MatchMedia().Rules(); len(got) != 1 || !got[0].Matches {
		t.Fatalf("Mocks().MatchMedia().Rules() = %#v, want seeded rule", got)
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
	if got := harness.Mocks().Storage().Events(); len(got) != 2 || got[0].Scope != "local" || got[0].Op != "seed" || got[0].Key != "token" || got[0].Value != "abc" || got[1].Scope != "session" || got[1].Op != "seed" || got[1].Key != "tab" || got[1].Value != "main" {
		t.Fatalf("Mocks().Storage().Events() = %#v, want builder seed events", got)
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
	if got, want := harness.Debug().LocationPathname(), "/next"; got != want {
		t.Fatalf("Debug().LocationPathname() after Navigate() = %q, want %q", got, want)
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
	if got, want := harness.Debug().LocationPathname(), "/relative"; got != want {
		t.Fatalf("Debug().LocationPathname() after relative Navigate() = %q, want %q", got, want)
	}
	if matches, err := harness.MatchMedia("(prefers-reduced-motion: reduce)"); err != nil || !matches {
		t.Fatalf("MatchMedia() = (%v, %v), want (true, nil)", matches, err)
	}
	harness.Mocks().MatchMedia().RecordListenerCall("(prefers-reduced-motion: reduce)", "change")
	if got := harness.Mocks().MatchMedia().ListenerCalls(); len(got) != 1 || got[0].Query != "(prefers-reduced-motion: reduce)" || got[0].Method != "change" {
		t.Fatalf("MatchMedia().ListenerCalls() = %#v, want one change listener call", got)
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
	if got := harness.Debug().FetchCalls(); len(got) != 1 || got[0].URL != "https://example.test/api/message" {
		t.Fatalf("Debug().FetchCalls() = %#v, want one call to example test URL", got)
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
	harness.Mocks().Storage().SeedLocal("theme", "dark")
	harness.Mocks().Storage().SeedSession("tab", "main")
	if got := harness.Mocks().Storage().Events(); len(got) != 2 || got[0].Scope != "local" || got[0].Op != "seed" || got[0].Key != "theme" || got[0].Value != "dark" || got[1].Scope != "session" || got[1].Op != "seed" || got[1].Key != "tab" || got[1].Value != "main" {
		t.Fatalf("Storage().Events() = %#v, want two storage change events", got)
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

func TestHarnessTextContentRoutesThroughRuntime(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out"><p>Hello</p><span>world</span></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, err := harness.TextContent("#out"); err != nil {
		t.Fatalf("TextContent(#out) error = %v", err)
	} else if want := "Helloworld"; got != want {
		t.Fatalf("TextContent(#out) = %q, want %q", got, want)
	}

	if err := harness.SetTextContent("#out", "plain"); err != nil {
		t.Fatalf("SetTextContent(#out) error = %v", err)
	}
	if got, err := harness.TextContent("#out"); err != nil {
		t.Fatalf("TextContent(#out) after SetTextContent error = %v", err)
	} else if want := "plain"; got != want {
		t.Fatalf("TextContent(#out) after SetTextContent = %q, want %q", got, want)
	}
	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">plain</div></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after SetTextContent = %q, want %q", got, want)
	}
}

func TestHarnessReplaceChildrenRoutesThroughRuntime(t *testing.T) {
	harness, err := FromHTML(`<form id="profile"><textarea id="bio">Base</textarea><button id="reset" type="reset">Reset</button></form>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.ReplaceChildren("#bio", "Draft"); err != nil {
		t.Fatalf("ReplaceChildren(#bio) error = %v", err)
	}
	if got, err := harness.TextContent("#bio"); err != nil {
		t.Fatalf("TextContent(#bio) after ReplaceChildren error = %v", err)
	} else if want := "Draft"; got != want {
		t.Fatalf("TextContent(#bio) after ReplaceChildren = %q, want %q", got, want)
	}
	if err := harness.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) error = %v", err)
	}
	if got, err := harness.TextContent("#bio"); err != nil {
		t.Fatalf("TextContent(#bio) after reset error = %v", err)
	} else if want := "Draft"; got != want {
		t.Fatalf("TextContent(#bio) after reset = %q, want %q", got, want)
	}
	if got, want := harness.Debug().DumpDOM(), `<form id="profile"><textarea id="bio">Draft</textarea><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("Debug().DumpDOM() after ReplaceChildren = %q, want %q", got, want)
	}
}

func TestHarnessCloneNodeRoutesThroughRuntime(t *testing.T) {
	harness, err := FromHTML(`<main><div id="source"><span>text</span></div><p id="tail">tail</p></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.CloneNode("#source", true); err != nil {
		t.Fatalf("CloneNode(#source, true) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<main><div id="source"><span>text</span></div><div id="source"><span>text</span></div><p id="tail">tail</p></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after CloneNode = %q, want %q", got, want)
	}
}
