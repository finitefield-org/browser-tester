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

func TestDebugViewReportsBuilderFailures(t *testing.T) {
	harness, err := NewHarnessBuilder().
		OpenFailure("open blocked").
		CloseFailure("close blocked").
		PrintFailure("print blocked").
		ScrollFailure("scroll blocked").
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	if got, want := harness.Debug().OpenFailure(), "open blocked"; got != want {
		t.Fatalf("Debug().OpenFailure() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().CloseFailure(), "close blocked"; got != want {
		t.Fatalf("Debug().CloseFailure() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().PrintFailure(), "print blocked"; got != want {
		t.Fatalf("Debug().PrintFailure() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().ScrollFailure(), "scroll blocked"; got != want {
		t.Fatalf("Debug().ScrollFailure() = %q, want %q", got, want)
	}
	if got := harness.Debug().OptionLabels(); len(got) != 0 {
		t.Fatalf("Debug().OptionLabels() = %#v, want empty before DOM bootstrap", got)
	}
	if got := harness.Debug().OptionValues(); len(got) != 0 {
		t.Fatalf("Debug().OptionValues() = %#v, want empty before DOM bootstrap", got)
	}
	if got := harness.Debug().SelectCount(); got != 0 {
		t.Fatalf("Debug().SelectCount() = %d, want 0 before DOM bootstrap", got)
	}
	if got := harness.Debug().TemplateCount(); got != 0 {
		t.Fatalf("Debug().TemplateCount() = %d, want 0 before DOM bootstrap", got)
	}
	if got := harness.Debug().TableCount(); got != 0 {
		t.Fatalf("Debug().TableCount() = %d, want 0 before DOM bootstrap", got)
	}
	if got := harness.Debug().ButtonCount(); got != 0 {
		t.Fatalf("Debug().ButtonCount() = %d, want 0 before DOM bootstrap", got)
	}
	if got := harness.Debug().TextAreaCount(); got != 0 {
		t.Fatalf("Debug().TextAreaCount() = %d, want 0 before DOM bootstrap", got)
	}
	if got := harness.Debug().InputCount(); got != 0 {
		t.Fatalf("Debug().InputCount() = %d, want 0 before DOM bootstrap", got)
	}
	if got := harness.Debug().OptionCount(); got != 0 {
		t.Fatalf("Debug().OptionCount() = %d, want 0 before DOM bootstrap", got)
	}
	if got := harness.Debug().SelectedOptionCount(); got != 0 {
		t.Fatalf("Debug().SelectedOptionCount() = %d, want 0 before DOM bootstrap", got)
	}
	if got := harness.Debug().OptgroupCount(); got != 0 {
		t.Fatalf("Debug().OptgroupCount() = %d, want 0 before DOM bootstrap", got)
	}
	if got := harness.Debug().LinkCount(); got != 0 {
		t.Fatalf("Debug().LinkCount() = %d, want 0 before DOM bootstrap", got)
	}
	if got := harness.Debug().AnchorCount(); got != 0 {
		t.Fatalf("Debug().AnchorCount() = %d, want 0 before DOM bootstrap", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().OpenFailure(); got != "" {
		t.Fatalf("nil Debug().OpenFailure() = %q, want empty", got)
	}
	if got := nilHarness.Debug().CloseFailure(); got != "" {
		t.Fatalf("nil Debug().CloseFailure() = %q, want empty", got)
	}
	if got := nilHarness.Debug().PrintFailure(); got != "" {
		t.Fatalf("nil Debug().PrintFailure() = %q, want empty", got)
	}
	if got := nilHarness.Debug().ScrollFailure(); got != "" {
		t.Fatalf("nil Debug().ScrollFailure() = %q, want empty", got)
	}
	if got := nilHarness.Debug().OptionLabels(); got != nil {
		t.Fatalf("nil Debug().OptionLabels() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().SelectedOptionLabels(); got != nil {
		t.Fatalf("nil Debug().SelectedOptionLabels() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().OptionValues(); got != nil {
		t.Fatalf("nil Debug().OptionValues() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().SelectedOptionValues(); got != nil {
		t.Fatalf("nil Debug().SelectedOptionValues() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().SelectCount(); got != 0 {
		t.Fatalf("nil Debug().SelectCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().TemplateCount(); got != 0 {
		t.Fatalf("nil Debug().TemplateCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().TableCount(); got != 0 {
		t.Fatalf("nil Debug().TableCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().ButtonCount(); got != 0 {
		t.Fatalf("nil Debug().ButtonCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().TextAreaCount(); got != 0 {
		t.Fatalf("nil Debug().TextAreaCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().InputCount(); got != 0 {
		t.Fatalf("nil Debug().InputCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().FieldsetCount(); got != 0 {
		t.Fatalf("nil Debug().FieldsetCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().LegendCount(); got != 0 {
		t.Fatalf("nil Debug().LegendCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().OutputCount(); got != 0 {
		t.Fatalf("nil Debug().OutputCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().LabelCount(); got != 0 {
		t.Fatalf("nil Debug().LabelCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().ProgressCount(); got != 0 {
		t.Fatalf("nil Debug().ProgressCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().MeterCount(); got != 0 {
		t.Fatalf("nil Debug().MeterCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().AudioCount(); got != 0 {
		t.Fatalf("nil Debug().AudioCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().VideoCount(); got != 0 {
		t.Fatalf("nil Debug().VideoCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().IframeCount(); got != 0 {
		t.Fatalf("nil Debug().IframeCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().EmbedCount(); got != 0 {
		t.Fatalf("nil Debug().EmbedCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().TrackCount(); got != 0 {
		t.Fatalf("nil Debug().TrackCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().PictureCount(); got != 0 {
		t.Fatalf("nil Debug().PictureCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().SourceCount(); got != 0 {
		t.Fatalf("nil Debug().SourceCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().DialogCount(); got != 0 {
		t.Fatalf("nil Debug().DialogCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().DetailsCount(); got != 0 {
		t.Fatalf("nil Debug().DetailsCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().SummaryCount(); got != 0 {
		t.Fatalf("nil Debug().SummaryCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().SectionCount(); got != 0 {
		t.Fatalf("nil Debug().SectionCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().MainCount(); got != 0 {
		t.Fatalf("nil Debug().MainCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().ArticleCount(); got != 0 {
		t.Fatalf("nil Debug().ArticleCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().NavCount(); got != 0 {
		t.Fatalf("nil Debug().NavCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().AsideCount(); got != 0 {
		t.Fatalf("nil Debug().AsideCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().FigureCount(); got != 0 {
		t.Fatalf("nil Debug().FigureCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().FigcaptionCount(); got != 0 {
		t.Fatalf("nil Debug().FigcaptionCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().HeaderCount(); got != 0 {
		t.Fatalf("nil Debug().HeaderCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().FooterCount(); got != 0 {
		t.Fatalf("nil Debug().FooterCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().AddressCount(); got != 0 {
		t.Fatalf("nil Debug().AddressCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().BlockquoteCount(); got != 0 {
		t.Fatalf("nil Debug().BlockquoteCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().ParagraphCount(); got != 0 {
		t.Fatalf("nil Debug().ParagraphCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().PreCount(); got != 0 {
		t.Fatalf("nil Debug().PreCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().MarkCount(); got != 0 {
		t.Fatalf("nil Debug().MarkCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().QCount(); got != 0 {
		t.Fatalf("nil Debug().QCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().CiteCount(); got != 0 {
		t.Fatalf("nil Debug().CiteCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().AbbrCount(); got != 0 {
		t.Fatalf("nil Debug().AbbrCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().StrongCount(); got != 0 {
		t.Fatalf("nil Debug().StrongCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().SpanCount(); got != 0 {
		t.Fatalf("nil Debug().SpanCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().DataCount(); got != 0 {
		t.Fatalf("nil Debug().DataCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().DfnCount(); got != 0 {
		t.Fatalf("nil Debug().DfnCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().KbdCount(); got != 0 {
		t.Fatalf("nil Debug().KbdCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().SampCount(); got != 0 {
		t.Fatalf("nil Debug().SampCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().RubyCount(); got != 0 {
		t.Fatalf("nil Debug().RubyCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().RtCount(); got != 0 {
		t.Fatalf("nil Debug().RtCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().VarCount(); got != 0 {
		t.Fatalf("nil Debug().VarCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().CodeCount(); got != 0 {
		t.Fatalf("nil Debug().CodeCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().SmallCount(); got != 0 {
		t.Fatalf("nil Debug().SmallCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().TimeCount(); got != 0 {
		t.Fatalf("nil Debug().TimeCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().OptionCount(); got != 0 {
		t.Fatalf("nil Debug().OptionCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().SelectedOptionCount(); got != 0 {
		t.Fatalf("nil Debug().SelectedOptionCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().OptgroupCount(); got != 0 {
		t.Fatalf("nil Debug().OptgroupCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().LinkCount(); got != 0 {
		t.Fatalf("nil Debug().LinkCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().AnchorCount(); got != 0 {
		t.Fatalf("nil Debug().AnchorCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().OptgroupLabels(); got != nil {
		t.Fatalf("nil Debug().OptgroupLabels() = %#v, want nil", got)
	}
}

func TestDebugViewReportsOptionLabels(t *testing.T) {
	harness, err := FromHTML(`<main><select id="mode"><option id="first" label="Display">Fallback</option><option id="second">Text</option><option id="empty" label="">Used text</option></select></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	labels := harness.Debug().OptionLabels()
	if len(labels) != 3 {
		t.Fatalf("Debug().OptionLabels() len = %d, want 3", len(labels))
	}
	if labels[0].Label != "Display" || labels[1].Label != "Text" || labels[2].Label != "Used text" {
		t.Fatalf("Debug().OptionLabels() = %#v, want labels in document order", labels)
	}
	if labels[0].NodeID == 0 || labels[1].NodeID == 0 || labels[2].NodeID == 0 {
		t.Fatalf("Debug().OptionLabels() = %#v, want node IDs", labels)
	}

	labels[0].Label = "mutated"
	labels[1].NodeID = 999
	fresh := harness.Debug().OptionLabels()
	if len(fresh) != 3 || fresh[0].Label != "Display" || fresh[1].Label != "Text" || fresh[2].Label != "Used text" {
		t.Fatalf("Debug().OptionLabels() reread = %#v, want original labels", fresh)
	}
}

func TestDebugViewReportsSelectedOptionLabels(t *testing.T) {
	harness, err := FromHTML(`<main><select id="mode" multiple><option id="first" label="Display" selected>Fallback</option><option id="second">Text</option><option id="third" selected>Third</option></select></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	labels := harness.Debug().SelectedOptionLabels()
	if len(labels) != 2 {
		t.Fatalf("Debug().SelectedOptionLabels() len = %d, want 2", len(labels))
	}
	if labels[0].Label != "Display" || labels[1].Label != "Third" {
		t.Fatalf("Debug().SelectedOptionLabels() = %#v, want selected labels in document order", labels)
	}
	if labels[0].NodeID == 0 || labels[1].NodeID == 0 {
		t.Fatalf("Debug().SelectedOptionLabels() = %#v, want node IDs", labels)
	}

	labels[0].Label = "mutated"
	labels[1].NodeID = 999
	fresh := harness.Debug().SelectedOptionLabels()
	if len(fresh) != 2 || fresh[0].Label != "Display" || fresh[1].Label != "Third" {
		t.Fatalf("Debug().SelectedOptionLabels() reread = %#v, want original labels", fresh)
	}
}

func TestDebugViewReportsOptionValues(t *testing.T) {
	harness, err := FromHTML(`<main><select id="mode"><option id="first" value="Display">Fallback</option><option id="second">Text</option><option id="empty" value="">Used text</option></select></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	values := harness.Debug().OptionValues()
	if len(values) != 3 {
		t.Fatalf("Debug().OptionValues() len = %d, want 3", len(values))
	}
	if values[0].Value != "Display" || values[1].Value != "Text" || values[2].Value != "" {
		t.Fatalf("Debug().OptionValues() = %#v, want values in document order", values)
	}
	if values[0].NodeID == 0 || values[1].NodeID == 0 || values[2].NodeID == 0 {
		t.Fatalf("Debug().OptionValues() = %#v, want node IDs", values)
	}

	values[0].Value = "mutated"
	values[1].NodeID = 999
	fresh := harness.Debug().OptionValues()
	if len(fresh) != 3 || fresh[0].Value != "Display" || fresh[1].Value != "Text" || fresh[2].Value != "" {
		t.Fatalf("Debug().OptionValues() reread = %#v, want original values", fresh)
	}
}

func TestDebugViewReportsSelectedOptionValues(t *testing.T) {
	harness, err := FromHTML(`<main><select id="mode" multiple><option id="first" value="Display" selected>Fallback</option><option id="second">Text</option><option id="third" selected>Third</option></select></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	values := harness.Debug().SelectedOptionValues()
	if len(values) != 2 {
		t.Fatalf("Debug().SelectedOptionValues() len = %d, want 2", len(values))
	}
	if values[0].Value != "Display" || values[1].Value != "Third" {
		t.Fatalf("Debug().SelectedOptionValues() = %#v, want selected values in document order", values)
	}
	if values[0].NodeID == 0 || values[1].NodeID == 0 {
		t.Fatalf("Debug().SelectedOptionValues() = %#v, want node IDs", values)
	}

	values[0].Value = "mutated"
	values[1].NodeID = 999
	fresh := harness.Debug().SelectedOptionValues()
	if len(fresh) != 2 || fresh[0].Value != "Display" || fresh[1].Value != "Third" {
		t.Fatalf("Debug().SelectedOptionValues() reread = %#v, want original values", fresh)
	}
}

func TestDebugViewReportsOptionCount(t *testing.T) {
	harness, err := FromHTML(`<main><select id="mode"><option id="first" value="Display">Fallback</option><option id="second">Text</option><div><option id="third">Ignored</option></div></select></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().OptionCount(), 3; got != want {
		t.Fatalf("Debug().OptionCount() = %d, want %d", got, want)
	}
	if got, want := harness.Debug().SelectedOptionCount(), 0; got != want {
		t.Fatalf("Debug().SelectedOptionCount() = %d, want %d", got, want)
	}
}

func TestDebugViewReportsSelectedOptionCount(t *testing.T) {
	harness, err := FromHTML(`<main><select id="mode" multiple><option id="first" value="Display" selected>Fallback</option><option id="second">Text</option><option id="third" selected>Third</option></select></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().OptionCount(), 3; got != want {
		t.Fatalf("Debug().OptionCount() = %d, want %d", got, want)
	}
	if got, want := harness.Debug().SelectedOptionCount(), 2; got != want {
		t.Fatalf("Debug().SelectedOptionCount() = %d, want %d", got, want)
	}
}

func TestDebugViewReportsOptgroupCount(t *testing.T) {
	harness, err := FromHTML(`<main><select id="mode"><optgroup id="first" label="Group"><option>A</option></optgroup><optgroup id="second"><legend>Legend</legend><option>B</option></optgroup><div><optgroup id="ignored" label="Nope"></optgroup></div></select></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().OptgroupCount(), 3; got != want {
		t.Fatalf("Debug().OptgroupCount() = %d, want %d", got, want)
	}
}

func TestDebugViewReportsLinkAndAnchorCount(t *testing.T) {
	harness, err := FromHTML(`<main><a href="/one">One</a><a name="anchor">Anchor</a><area href="/area" alt="Area"><div><a href="/two">Two</a><a name="inner">Inner</a></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().LinkCount(), 3; got != want {
		t.Fatalf("Debug().LinkCount() = %d, want %d", got, want)
	}
	if got, want := harness.Debug().AnchorCount(), 2; got != want {
		t.Fatalf("Debug().AnchorCount() = %d, want %d", got, want)
	}
	if !harness.Debug().DOMReady() {
		t.Fatalf("Debug().DOMReady() = false, want true after link count bootstrap")
	}
	if got := harness.Debug().DOMError(); got != "" {
		t.Fatalf("Debug().DOMError() = %q, want empty after link count bootstrap", got)
	}
}

func TestDebugViewReportsOptgroupLabels(t *testing.T) {
	harness, err := FromHTML(`<main><select><optgroup id="plain" label="Group"><option>A</option></optgroup><optgroup id="legend"><legend> Legend  Name </legend><option>B</option></optgroup></select></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	labels := harness.Debug().OptgroupLabels()
	if len(labels) != 2 {
		t.Fatalf("Debug().OptgroupLabels() len = %d, want 2", len(labels))
	}
	if labels[0].Label != "Group" || labels[1].Label != "Legend Name" {
		t.Fatalf("Debug().OptgroupLabels() = %#v, want labels in document order", labels)
	}
	if labels[0].NodeID == 0 || labels[1].NodeID == 0 {
		t.Fatalf("Debug().OptgroupLabels() = %#v, want node IDs", labels)
	}

	labels[0].Label = "mutated"
	labels[1].NodeID = 999
	fresh := harness.Debug().OptgroupLabels()
	if len(fresh) != 2 || fresh[0].Label != "Group" || fresh[1].Label != "Legend Name" {
		t.Fatalf("Debug().OptgroupLabels() reread = %#v, want original labels", fresh)
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

	rules := harness.Mocks().MatchMedia().Rules()
	if len(rules) != 1 || rules[0].Query != "(prefers-reduced-motion: reduce)" || !rules[0].Matches {
		t.Fatalf("MatchMedia().Rules() = %#v, want one seeded rule", rules)
	}
	rules[0].Matches = false
	if got := harness.Mocks().MatchMedia().Rules(); len(got) != 1 || got[0].Matches != true {
		t.Fatalf("MatchMedia().Rules() reread = %#v, want original rule", got)
	}
}

func TestMatchMediaListenerCallsReturnCopies(t *testing.T) {
	harness, err := NewHarnessBuilder().
		MatchMedia(map[string]bool{"(prefers-reduced-motion: reduce)": true}).
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	harness.Mocks().MatchMedia().RecordListenerCall("(prefers-reduced-motion: reduce)", "addListener")
	harness.Mocks().MatchMedia().RecordListenerCall("(prefers-reduced-motion: reduce)", "removeListener")

	listeners := harness.Mocks().MatchMedia().ListenerCalls()
	if len(listeners) != 2 || listeners[0].Query != "(prefers-reduced-motion: reduce)" || listeners[0].Method != "addListener" || listeners[1].Method != "removeListener" {
		t.Fatalf("MatchMedia().ListenerCalls() = %#v, want both listener calls", listeners)
	}

	listeners[0].Query = "mutated"
	listeners[1].Method = "mutated"

	fresh := harness.Mocks().MatchMedia().ListenerCalls()
	if len(fresh) != 2 || fresh[0].Query != "(prefers-reduced-motion: reduce)" || fresh[0].Method != "addListener" || fresh[1].Method != "removeListener" {
		t.Fatalf("MatchMedia().ListenerCalls() reread = %#v, want original listener calls", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Mocks().MatchMedia(); got != nil {
		t.Fatalf("nil Harness.Mocks().MatchMedia() = %#v, want nil", got)
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

func TestDebugViewReportsTargetNodeID(t *testing.T) {
	harness, err := FromHTMLWithURL("https://example.test/page#target", `<main><a id="target">Target</a><p id="other">Other</p></main>`)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got := harness.Debug().TargetNodeID(); got == 0 {
		t.Fatalf("Debug().TargetNodeID() = %d, want targeted node id", got)
	}

	if err := harness.Navigate("#missing"); err != nil {
		t.Fatalf("Navigate(#missing) error = %v", err)
	}
	if got := harness.Debug().TargetNodeID(); got != 0 {
		t.Fatalf("Debug().TargetNodeID() after missing fragment = %d, want 0", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().TargetNodeID(); got != 0 {
		t.Fatalf("nil Debug().TargetNodeID() = %d, want 0", got)
	}
}

func TestDebugViewReportsFocusedNodeID(t *testing.T) {
	harness, err := FromHTML(`<main><input id="field" type="text" value="hello"><p id="other">other</p></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Focus("#field"); err != nil {
		t.Fatalf("Focus(#field) error = %v", err)
	}
	if got := harness.Debug().FocusedNodeID(); got == 0 {
		t.Fatalf("Debug().FocusedNodeID() = %d, want focused node id", got)
	}

	if err := harness.Blur(); err != nil {
		t.Fatalf("Blur() error = %v", err)
	}
	if got := harness.Debug().FocusedNodeID(); got != 0 {
		t.Fatalf("Debug().FocusedNodeID() after blur = %d, want 0", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().FocusedNodeID(); got != 0 {
		t.Fatalf("nil Debug().FocusedNodeID() = %d, want 0", got)
	}
}

func TestDebugViewReportsNodeCount(t *testing.T) {
	harness, err := FromHTML(`<main><section><p id="one">one</p><p id="two">two</p></section></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("main"); err != nil {
		t.Fatalf("AssertExists(main) error = %v", err)
	}

	if got := harness.Debug().NodeCount(); got == 0 {
		t.Fatalf("Debug().NodeCount() = %d, want non-zero", got)
	}
	got := harness.Debug().NodeCount()
	if again := harness.Debug().NodeCount(); got != again {
		t.Fatalf("Debug().NodeCount() reread = %d, want stable result %d", again, got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().NodeCount(); got != 0 {
		t.Fatalf("nil Debug().NodeCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsScriptCount(t *testing.T) {
	harness, err := FromHTML(`<main><script></script><div id="host"></div><script>host:setTextContent(#host, changed)</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().ScriptCount(); got != 2 {
		t.Fatalf("Debug().ScriptCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", "<script></script>"); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().ScriptCount(); got != 3 {
		t.Fatalf("Debug().ScriptCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().ScriptCount(); got != 0 {
		t.Fatalf("nil Debug().ScriptCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsImageCount(t *testing.T) {
	harness, err := FromHTML(`<main><img id="first" src="/a"><div id="host"></div><img name="second" src="/b"></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().ImageCount(); got != 2 {
		t.Fatalf("Debug().ImageCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", "<img>"); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().ImageCount(); got != 3 {
		t.Fatalf("Debug().ImageCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().ImageCount(); got != 0 {
		t.Fatalf("nil Debug().ImageCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsFormCount(t *testing.T) {
	harness, err := FromHTML(`<main><form id="first"></form><div id="host"></div><form name="second"></form></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().FormCount(); got != 2 {
		t.Fatalf("Debug().FormCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", "<form></form>"); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().FormCount(); got != 3 {
		t.Fatalf("Debug().FormCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().FormCount(); got != 0 {
		t.Fatalf("nil Debug().FormCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsSelectCount(t *testing.T) {
	harness, err := FromHTML(`<main><select id="first"></select><div id="host"></div><div><select name="second"></select></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().SelectCount(); got != 2 {
		t.Fatalf("Debug().SelectCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<select id="third"></select>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().SelectCount(); got != 3 {
		t.Fatalf("Debug().SelectCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().SelectCount(); got != 0 {
		t.Fatalf("nil Debug().SelectCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsTemplateCount(t *testing.T) {
	harness, err := FromHTML(`<main><template id="first"></template><div id="host"></div><div><template name="second"></template></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().TemplateCount(); got != 2 {
		t.Fatalf("Debug().TemplateCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<template id="third"></template>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().TemplateCount(); got != 3 {
		t.Fatalf("Debug().TemplateCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().TemplateCount(); got != 0 {
		t.Fatalf("nil Debug().TemplateCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsTableCount(t *testing.T) {
	harness, err := FromHTML(`<main><table id="first"></table><div id="host"></div><div><table name="second"></table></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().TableCount(); got != 2 {
		t.Fatalf("Debug().TableCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<table id="third"></table>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().TableCount(); got != 3 {
		t.Fatalf("Debug().TableCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().TableCount(); got != 0 {
		t.Fatalf("nil Debug().TableCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsButtonCount(t *testing.T) {
	harness, err := FromHTML(`<main><button id="first"></button><div id="host"></div><div><button name="second"></button></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().ButtonCount(); got != 2 {
		t.Fatalf("Debug().ButtonCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<button id="third"></button>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().ButtonCount(); got != 3 {
		t.Fatalf("Debug().ButtonCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().ButtonCount(); got != 0 {
		t.Fatalf("nil Debug().ButtonCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsTextAreaCount(t *testing.T) {
	harness, err := FromHTML(`<main><textarea id="first"></textarea><div id="host"></div><div><textarea name="second"></textarea></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().TextAreaCount(); got != 2 {
		t.Fatalf("Debug().TextAreaCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<textarea id="third"></textarea>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().TextAreaCount(); got != 3 {
		t.Fatalf("Debug().TextAreaCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().TextAreaCount(); got != 0 {
		t.Fatalf("nil Debug().TextAreaCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsInputCount(t *testing.T) {
	harness, err := FromHTML(`<main><input id="first"><div id="host"></div><div><input name="second"></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().InputCount(); got != 2 {
		t.Fatalf("Debug().InputCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<input id="third">`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().InputCount(); got != 3 {
		t.Fatalf("Debug().InputCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().InputCount(); got != 0 {
		t.Fatalf("nil Debug().InputCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsFieldsetCount(t *testing.T) {
	harness, err := FromHTML(`<main><fieldset id="first"></fieldset><div id="host"></div><div><fieldset name="second"></fieldset></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().FieldsetCount(); got != 2 {
		t.Fatalf("Debug().FieldsetCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<fieldset id="third"></fieldset>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().FieldsetCount(); got != 3 {
		t.Fatalf("Debug().FieldsetCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().FieldsetCount(); got != 0 {
		t.Fatalf("nil Debug().FieldsetCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsLegendCount(t *testing.T) {
	harness, err := FromHTML(`<main><fieldset><legend id="first"></legend></fieldset><div id="host"></div><div><legend name="second"></legend></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().LegendCount(); got != 2 {
		t.Fatalf("Debug().LegendCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<legend id="third"></legend>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().LegendCount(); got != 3 {
		t.Fatalf("Debug().LegendCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().LegendCount(); got != 0 {
		t.Fatalf("nil Debug().LegendCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsOutputCount(t *testing.T) {
	harness, err := FromHTML(`<main><output id="first"></output><div id="host"></div><div><output name="second"></output></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().OutputCount(); got != 2 {
		t.Fatalf("Debug().OutputCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<output id="third"></output>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().OutputCount(); got != 3 {
		t.Fatalf("Debug().OutputCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().OutputCount(); got != 0 {
		t.Fatalf("nil Debug().OutputCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsLabelCount(t *testing.T) {
	harness, err := FromHTML(`<main><label id="first">A</label><div id="host"></div><div><label name="second">B</label></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().LabelCount(); got != 2 {
		t.Fatalf("Debug().LabelCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<label id="third">C</label>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().LabelCount(); got != 3 {
		t.Fatalf("Debug().LabelCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().LabelCount(); got != 0 {
		t.Fatalf("nil Debug().LabelCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsProgressCount(t *testing.T) {
	harness, err := FromHTML(`<main id="root"><input id="mixed" type="checkbox" indeterminate><input id="radio-a" type="radio" name="size"><input id="radio-b" type="radio" name="size"><progress id="task"></progress><progress id="done" value="42"></progress></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().ProgressCount(); got != 2 {
		t.Fatalf("Debug().ProgressCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#root", `<progress id="third"></progress>`); err != nil {
		t.Fatalf("SetInnerHTML(#root) error = %v", err)
	}
	if got := harness.Debug().ProgressCount(); got != 1 {
		t.Fatalf("Debug().ProgressCount() after SetInnerHTML = %d, want 1", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().ProgressCount(); got != 0 {
		t.Fatalf("nil Debug().ProgressCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsMeterCount(t *testing.T) {
	harness, err := FromHTML(`<main><meter id="first"></meter><div id="host"></div><div><meter name="second" value="3"></meter></div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().MeterCount(); got != 2 {
		t.Fatalf("Debug().MeterCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<meter id="third"></meter>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().MeterCount(); got != 3 {
		t.Fatalf("Debug().MeterCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().MeterCount(); got != 0 {
		t.Fatalf("nil Debug().MeterCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsAudioAndVideoCount(t *testing.T) {
	harness, err := FromHTML(`<main><audio id="first"></audio><video id="second"></video><div id="host"></div><section><audio id="third"></audio><video id="fourth"></video></section></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().AudioCount(); got != 2 {
		t.Fatalf("Debug().AudioCount() = %d, want 2", got)
	}
	if got := harness.Debug().VideoCount(); got != 2 {
		t.Fatalf("Debug().VideoCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<audio id="fifth"></audio><video id="sixth"></video>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().AudioCount(); got != 3 {
		t.Fatalf("Debug().AudioCount() after SetInnerHTML = %d, want 3", got)
	}
	if got := harness.Debug().VideoCount(); got != 3 {
		t.Fatalf("Debug().VideoCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().AudioCount(); got != 0 {
		t.Fatalf("nil Debug().AudioCount() = %d, want 0", got)
	}
	if got := nilHarness.Debug().VideoCount(); got != 0 {
		t.Fatalf("nil Debug().VideoCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsIframeCount(t *testing.T) {
	harness, err := FromHTML(`<main><iframe id="first"></iframe><div id="host"></div><section><iframe id="second"></iframe></section></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().IframeCount(); got != 2 {
		t.Fatalf("Debug().IframeCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<iframe id="third"></iframe>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().IframeCount(); got != 3 {
		t.Fatalf("Debug().IframeCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().IframeCount(); got != 0 {
		t.Fatalf("nil Debug().IframeCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsEmbedCount(t *testing.T) {
	harness, err := FromHTML(`<main><embed id="first"><div id="host"></div><section><embed id="second"></section></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().EmbedCount(); got != 2 {
		t.Fatalf("Debug().EmbedCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<embed id="third">`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().EmbedCount(); got != 3 {
		t.Fatalf("Debug().EmbedCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().EmbedCount(); got != 0 {
		t.Fatalf("nil Debug().EmbedCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsTrackCount(t *testing.T) {
	harness, err := FromHTML(`<main><track id="first"><div id="host"></div><section><track id="second"></section></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().TrackCount(); got != 2 {
		t.Fatalf("Debug().TrackCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<track id="third">`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().TrackCount(); got != 3 {
		t.Fatalf("Debug().TrackCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().TrackCount(); got != 0 {
		t.Fatalf("nil Debug().TrackCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsPictureCount(t *testing.T) {
	harness, err := FromHTML(`<main><picture id="first"></picture><div id="host"></div><section><picture id="second"></picture></section></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().PictureCount(); got != 2 {
		t.Fatalf("Debug().PictureCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<picture id="third"></picture>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().PictureCount(); got != 3 {
		t.Fatalf("Debug().PictureCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().PictureCount(); got != 0 {
		t.Fatalf("nil Debug().PictureCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsSourceCount(t *testing.T) {
	harness, err := FromHTML(`<main><picture><source id="first"><source id="second"></picture><div id="host"></div><audio><source id="third"></audio></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().SourceCount(); got != 3 {
		t.Fatalf("Debug().SourceCount() = %d, want 3", got)
	}

	if err := harness.SetInnerHTML("#host", `<video><source id="fourth"></video>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().SourceCount(); got != 4 {
		t.Fatalf("Debug().SourceCount() after SetInnerHTML = %d, want 4", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().SourceCount(); got != 0 {
		t.Fatalf("nil Debug().SourceCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsDialogCount(t *testing.T) {
	harness, err := FromHTML(`<main><dialog id="first"></dialog><div id="host"></div><section><dialog id="second"></dialog></section></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().DialogCount(); got != 2 {
		t.Fatalf("Debug().DialogCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<dialog id="third"></dialog>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().DialogCount(); got != 3 {
		t.Fatalf("Debug().DialogCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().DialogCount(); got != 0 {
		t.Fatalf("nil Debug().DialogCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsDetailsCount(t *testing.T) {
	harness, err := FromHTML(`<main><details id="first"></details><div id="host"></div><section><details id="second"></details></section></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().DetailsCount(); got != 2 {
		t.Fatalf("Debug().DetailsCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<details id="third"></details>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().DetailsCount(); got != 3 {
		t.Fatalf("Debug().DetailsCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().DetailsCount(); got != 0 {
		t.Fatalf("nil Debug().DetailsCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsSummaryCount(t *testing.T) {
	harness, err := FromHTML(`<main><details id="first"><summary id="one">One</summary></details><div id="host"></div><section><details id="second"><summary id="two">Two</summary></details></section></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().SummaryCount(); got != 2 {
		t.Fatalf("Debug().SummaryCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<details id="third"><summary id="three">Three</summary></details>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().SummaryCount(); got != 3 {
		t.Fatalf("Debug().SummaryCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().SummaryCount(); got != 0 {
		t.Fatalf("nil Debug().SummaryCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsSectionCount(t *testing.T) {
	harness, err := FromHTML(`<main><section id="first"></section><div id="host"></div><article><section id="second"></section></article></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().SectionCount(); got != 2 {
		t.Fatalf("Debug().SectionCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<section id="third"></section>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().SectionCount(); got != 3 {
		t.Fatalf("Debug().SectionCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().SectionCount(); got != 0 {
		t.Fatalf("nil Debug().SectionCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsMainCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><main id="first"></main><div id="host"></div><section><main id="second"></main></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().MainCount(); got != 2 {
		t.Fatalf("Debug().MainCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<main id="third"></main>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().MainCount(); got != 3 {
		t.Fatalf("Debug().MainCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().MainCount(); got != 0 {
		t.Fatalf("nil Debug().MainCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsArticleCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><article id="first"></article><div id="host"></div><section><article id="second"></article></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().ArticleCount(); got != 2 {
		t.Fatalf("Debug().ArticleCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<article id="third"></article>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().ArticleCount(); got != 3 {
		t.Fatalf("Debug().ArticleCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().ArticleCount(); got != 0 {
		t.Fatalf("nil Debug().ArticleCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsNavCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><nav id="first"></nav><div id="host"></div><section><nav id="second"></nav></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().NavCount(); got != 2 {
		t.Fatalf("Debug().NavCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<nav id="third"></nav>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().NavCount(); got != 3 {
		t.Fatalf("Debug().NavCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().NavCount(); got != 0 {
		t.Fatalf("nil Debug().NavCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsAsideCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><aside id="first"></aside><div id="host"></div><section><aside id="second"></aside></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().AsideCount(); got != 2 {
		t.Fatalf("Debug().AsideCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<aside id="third"></aside>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().AsideCount(); got != 3 {
		t.Fatalf("Debug().AsideCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().AsideCount(); got != 0 {
		t.Fatalf("nil Debug().AsideCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsFigureCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><figure id="first"></figure><div id="host"></div><section><figure id="second"></figure></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().FigureCount(); got != 2 {
		t.Fatalf("Debug().FigureCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<figure id="third"></figure>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().FigureCount(); got != 3 {
		t.Fatalf("Debug().FigureCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().FigureCount(); got != 0 {
		t.Fatalf("nil Debug().FigureCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsFigcaptionCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><figure id="first"><figcaption id="caption-one">One</figcaption></figure><div id="host"></div><section><figure id="second"><figcaption id="caption-two">Two</figcaption></figure></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().FigcaptionCount(); got != 2 {
		t.Fatalf("Debug().FigcaptionCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<figure id="third"><figcaption id="caption-three">Three</figcaption></figure>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().FigcaptionCount(); got != 3 {
		t.Fatalf("Debug().FigcaptionCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().FigcaptionCount(); got != 0 {
		t.Fatalf("nil Debug().FigcaptionCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsHeaderCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><header id="first"></header><div id="host"></div><section><header id="second"></header></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().HeaderCount(); got != 2 {
		t.Fatalf("Debug().HeaderCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<header id="third"></header>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().HeaderCount(); got != 3 {
		t.Fatalf("Debug().HeaderCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().HeaderCount(); got != 0 {
		t.Fatalf("nil Debug().HeaderCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsFooterCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><footer id="first"></footer><div id="host"></div><section><footer id="second"></footer></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().FooterCount(); got != 2 {
		t.Fatalf("Debug().FooterCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<footer id="third"></footer>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().FooterCount(); got != 3 {
		t.Fatalf("Debug().FooterCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().FooterCount(); got != 0 {
		t.Fatalf("nil Debug().FooterCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsAddressCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><address id="first"></address><div id="host"></div><section><address id="second"></address></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().AddressCount(); got != 2 {
		t.Fatalf("Debug().AddressCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<address id="third"></address>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().AddressCount(); got != 3 {
		t.Fatalf("Debug().AddressCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().AddressCount(); got != 0 {
		t.Fatalf("nil Debug().AddressCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsBlockquoteCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><blockquote id="first"></blockquote><div id="host"></div><section><blockquote id="second"></blockquote></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().BlockquoteCount(); got != 2 {
		t.Fatalf("Debug().BlockquoteCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<blockquote id="third"></blockquote>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().BlockquoteCount(); got != 3 {
		t.Fatalf("Debug().BlockquoteCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().BlockquoteCount(); got != 0 {
		t.Fatalf("nil Debug().BlockquoteCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsParagraphCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><p id="first"></p><div id="host"></div><section><p id="second"></p></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().ParagraphCount(); got != 2 {
		t.Fatalf("Debug().ParagraphCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<p id="third"></p>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().ParagraphCount(); got != 3 {
		t.Fatalf("Debug().ParagraphCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().ParagraphCount(); got != 0 {
		t.Fatalf("nil Debug().ParagraphCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsPreCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><pre id="first"></pre><div id="host"></div><section><pre id="second"></pre></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().PreCount(); got != 2 {
		t.Fatalf("Debug().PreCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<pre id="third"></pre>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().PreCount(); got != 3 {
		t.Fatalf("Debug().PreCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().PreCount(); got != 0 {
		t.Fatalf("nil Debug().PreCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsMarkCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><mark id="first"></mark><div id="host"></div><section><mark id="second"></mark></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().MarkCount(); got != 2 {
		t.Fatalf("Debug().MarkCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<mark id="third"></mark>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().MarkCount(); got != 3 {
		t.Fatalf("Debug().MarkCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().MarkCount(); got != 0 {
		t.Fatalf("nil Debug().MarkCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsQCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><q id="first"></q><div id="host"></div><section><q id="second"></q></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().QCount(); got != 2 {
		t.Fatalf("Debug().QCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<q id="third"></q>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().QCount(); got != 3 {
		t.Fatalf("Debug().QCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().QCount(); got != 0 {
		t.Fatalf("nil Debug().QCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsCiteCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><cite id="first"></cite><div id="host"></div><section><cite id="second"></cite></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().CiteCount(); got != 2 {
		t.Fatalf("Debug().CiteCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<cite id="third"></cite>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().CiteCount(); got != 3 {
		t.Fatalf("Debug().CiteCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().CiteCount(); got != 0 {
		t.Fatalf("nil Debug().CiteCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsAbbrCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><abbr id="first"></abbr><div id="host"></div><section><abbr id="second"></abbr></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().AbbrCount(); got != 2 {
		t.Fatalf("Debug().AbbrCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<abbr id="third"></abbr>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().AbbrCount(); got != 3 {
		t.Fatalf("Debug().AbbrCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().AbbrCount(); got != 0 {
		t.Fatalf("nil Debug().AbbrCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsStrongCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><strong id="first"></strong><div id="host"></div><section><strong id="second"></strong></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().StrongCount(); got != 2 {
		t.Fatalf("Debug().StrongCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<strong id="third"></strong>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().StrongCount(); got != 3 {
		t.Fatalf("Debug().StrongCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().StrongCount(); got != 0 {
		t.Fatalf("nil Debug().StrongCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsSpanCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><span id="first"></span><div id="host"></div><section><span id="second"></span></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().SpanCount(); got != 2 {
		t.Fatalf("Debug().SpanCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<span id="third"></span>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().SpanCount(); got != 3 {
		t.Fatalf("Debug().SpanCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().SpanCount(); got != 0 {
		t.Fatalf("nil Debug().SpanCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsDataCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><data id="first"></data><div id="host"></div><section><data id="second"></data></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().DataCount(); got != 2 {
		t.Fatalf("Debug().DataCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<data id="third"></data>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().DataCount(); got != 3 {
		t.Fatalf("Debug().DataCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().DataCount(); got != 0 {
		t.Fatalf("nil Debug().DataCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsDfnCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><dfn id="first"></dfn><div id="host"></div><section><dfn id="second"></dfn></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().DfnCount(); got != 2 {
		t.Fatalf("Debug().DfnCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<dfn id="third"></dfn>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().DfnCount(); got != 3 {
		t.Fatalf("Debug().DfnCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().DfnCount(); got != 0 {
		t.Fatalf("nil Debug().DfnCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsKbdCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><kbd id="first"></kbd><div id="host"></div><section><kbd id="second"></kbd></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().KbdCount(); got != 2 {
		t.Fatalf("Debug().KbdCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<kbd id="third"></kbd>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().KbdCount(); got != 3 {
		t.Fatalf("Debug().KbdCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().KbdCount(); got != 0 {
		t.Fatalf("nil Debug().KbdCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsSampCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><samp id="first"></samp><div id="host"></div><section><samp id="second"></samp></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().SampCount(); got != 2 {
		t.Fatalf("Debug().SampCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<samp id="third"></samp>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().SampCount(); got != 3 {
		t.Fatalf("Debug().SampCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().SampCount(); got != 0 {
		t.Fatalf("nil Debug().SampCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsRubyCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><ruby id="first"><rt>one</rt></ruby><div id="host"></div><section><ruby id="second"></ruby></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().RubyCount(); got != 2 {
		t.Fatalf("Debug().RubyCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<ruby id="third"></ruby>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().RubyCount(); got != 3 {
		t.Fatalf("Debug().RubyCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().RubyCount(); got != 0 {
		t.Fatalf("nil Debug().RubyCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsRtCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><ruby id="first"><rt id="first-rt">one</rt></ruby><div id="host"></div><section><ruby id="second"><rt id="second-rt">two</rt></ruby></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().RtCount(); got != 2 {
		t.Fatalf("Debug().RtCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<ruby id="third"><rt id="third-rt">three</rt></ruby>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().RtCount(); got != 3 {
		t.Fatalf("Debug().RtCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().RtCount(); got != 0 {
		t.Fatalf("nil Debug().RtCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsVarCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><var id="first"></var><div id="host"></div><section><var id="second"></var></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().VarCount(); got != 2 {
		t.Fatalf("Debug().VarCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<var id="third"></var>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().VarCount(); got != 3 {
		t.Fatalf("Debug().VarCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().VarCount(); got != 0 {
		t.Fatalf("nil Debug().VarCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsCodeCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><code id="first"></code><div id="host"></div><section><code id="second"></code></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().CodeCount(); got != 2 {
		t.Fatalf("Debug().CodeCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<code id="third"></code>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().CodeCount(); got != 3 {
		t.Fatalf("Debug().CodeCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().CodeCount(); got != 0 {
		t.Fatalf("nil Debug().CodeCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsSmallCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><small id="first"></small><div id="host"></div><section><small id="second"></small></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().SmallCount(); got != 2 {
		t.Fatalf("Debug().SmallCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<small id="third"></small>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().SmallCount(); got != 3 {
		t.Fatalf("Debug().SmallCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().SmallCount(); got != 0 {
		t.Fatalf("nil Debug().SmallCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsTimeCount(t *testing.T) {
	harness, err := FromHTML(`<div id="root"><time id="first"></time><div id="host"></div><section><time id="second"></time></section></div>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().TimeCount(); got != 2 {
		t.Fatalf("Debug().TimeCount() = %d, want 2", got)
	}

	if err := harness.SetInnerHTML("#host", `<time id="third"></time>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := harness.Debug().TimeCount(); got != 3 {
		t.Fatalf("Debug().TimeCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().TimeCount(); got != 0 {
		t.Fatalf("nil Debug().TimeCount() = %d, want 0", got)
	}
}

func TestDebugViewReportsHistoryState(t *testing.T) {
	harness, err := FromHTMLWithURL("https://example.test/page", `<main><script>host:historyPushState("step-1", "", "#step-1")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got := harness.Debug().HistoryLength(); got != 2 {
		t.Fatalf("Debug().HistoryLength() = %d, want 2", got)
	}
	if got, ok := harness.Debug().HistoryState(); !ok || got != "step-1" {
		t.Fatalf("Debug().HistoryState() = (%q, %v), want (\"step-1\", true)", got, ok)
	}

	if err := harness.Navigate("#missing"); err != nil {
		t.Fatalf("Navigate(#missing) error = %v", err)
	}
	if got := harness.Debug().HistoryLength(); got != 3 {
		t.Fatalf("Debug().HistoryLength() after Navigate = %d, want 3", got)
	}
	if got, ok := harness.Debug().HistoryState(); ok || got != "null" {
		t.Fatalf("Debug().HistoryState() after Navigate = (%q, %v), want (\"null\", false)", got, ok)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().HistoryLength(); got != 0 {
		t.Fatalf("nil Debug().HistoryLength() = %d, want 0", got)
	}
	if got, ok := nilHarness.Debug().HistoryState(); ok || got != "null" {
		t.Fatalf("nil Debug().HistoryState() = (%q, %v), want (\"null\", false)", got, ok)
	}
}

func TestDebugViewReportsHistoryEntries(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/app",
		`<main><script>host:historyPushState("step-1", "", "#step-1"); host:historyReplaceState("step-2", "", "#step-2")</script></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	entries := harness.Debug().HistoryEntries()
	if len(entries) != 2 {
		t.Fatalf("Debug().HistoryEntries() = %#v, want 2 entries", entries)
	}
	if entries[0].URL != "https://example.test/app" || entries[0].HasState {
		t.Fatalf("Debug().HistoryEntries()[0] = %#v, want initial entry without state", entries[0])
	}
	if entries[1].URL != "https://example.test/app#step-2" || !entries[1].HasState || entries[1].State != "step-2" {
		t.Fatalf("Debug().HistoryEntries()[1] = %#v, want current entry with step-2 state", entries[1])
	}

	entries[0].URL = "mutated"
	entries[1].State = "mutated"
	if fresh := harness.Debug().HistoryEntries(); len(fresh) != 2 || fresh[0].URL != "https://example.test/app" || fresh[1].State != "step-2" {
		t.Fatalf("Debug().HistoryEntries() reread = %#v, want original history entries", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().HistoryEntries(); got != nil {
		t.Fatalf("nil Debug().HistoryEntries() = %#v, want nil", got)
	}
}

func TestDebugViewReportsHistoryIndex(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/app",
		`<main><script>host:historyPushState("step-1", "", "#step-1")</script></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got := harness.Debug().HistoryIndex(); got != 1 {
		t.Fatalf("Debug().HistoryIndex() = %d, want 1", got)
	}
	if err := harness.Navigate("#other"); err != nil {
		t.Fatalf("Navigate(#other) error = %v", err)
	}
	if got := harness.Debug().HistoryIndex(); got != 2 {
		t.Fatalf("Debug().HistoryIndex() after Navigate = %d, want 2", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().HistoryIndex(); got != 0 {
		t.Fatalf("nil Debug().HistoryIndex() = %d, want 0", got)
	}
}

func TestDebugViewReportsVisitedURLs(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/app",
		`<main><script>host:historyPushState("step-1", "", "#step-1")</script></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	visited := harness.Debug().VisitedURLs()
	if len(visited) != 2 || visited[0] != "https://example.test/app" || visited[1] != "https://example.test/app#step-1" {
		t.Fatalf("Debug().VisitedURLs() = %#v, want history-derived visited URLs", visited)
	}

	visited[0] = "mutated"
	visited[1] = "mutated"
	if fresh := harness.Debug().VisitedURLs(); len(fresh) != 2 || fresh[0] != "https://example.test/app" || fresh[1] != "https://example.test/app#step-1" {
		t.Fatalf("Debug().VisitedURLs() reread = %#v, want original visited URLs", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().VisitedURLs(); got != nil {
		t.Fatalf("nil Debug().VisitedURLs() = %#v, want nil", got)
	}
}

func TestDebugViewReportsPendingTimers(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out"></div><script>host:setTimeout('host:insertAdjacentHTML("#out", "beforeend", "<span>timeout</span>")', 5); host:setInterval('host:insertAdjacentHTML("#out", "beforeend", "<span>interval</span>")', 9); host:requestAnimationFrame('host:insertAdjacentHTML("#out", "beforeend", "<span>frame</span>")')</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	timers := harness.Debug().PendingTimers()
	if len(timers) != 2 {
		t.Fatalf("Debug().PendingTimers() = %#v, want 2 entries", timers)
	}
	if timers[0].DueAtMs != 5 || timers[0].IntervalMs != 5 || timers[0].Repeat {
		t.Fatalf("Debug().PendingTimers()[0] = %#v, want one-shot timer due at 5", timers[0])
	}
	if timers[1].DueAtMs != 9 || timers[1].IntervalMs != 9 || !timers[1].Repeat {
		t.Fatalf("Debug().PendingTimers()[1] = %#v, want repeating timer due at 9", timers[1])
	}
	timers[0].Source = "mutated"
	if fresh := harness.Debug().PendingTimers(); len(fresh) != 2 || fresh[0].Source != `host:insertAdjacentHTML("#out", "beforeend", "<span>timeout</span>")` {
		t.Fatalf("Debug().PendingTimers() reread = %#v, want original timer snapshot", fresh)
	}

	frames := harness.Debug().PendingAnimationFrames()
	if len(frames) != 1 || frames[0].Source != `host:insertAdjacentHTML("#out", "beforeend", "<span>frame</span>")` {
		t.Fatalf("Debug().PendingAnimationFrames() = %#v, want one pending frame", frames)
	}
	frames[0].Source = "mutated"
	if fresh := harness.Debug().PendingAnimationFrames(); len(fresh) != 1 || fresh[0].Source != `host:insertAdjacentHTML("#out", "beforeend", "<span>frame</span>")` {
		t.Fatalf("Debug().PendingAnimationFrames() reread = %#v, want original frame snapshot", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().PendingTimers(); got != nil {
		t.Fatalf("nil Debug().PendingTimers() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().PendingAnimationFrames(); got != nil {
		t.Fatalf("nil Debug().PendingAnimationFrames() = %#v, want nil", got)
	}
}

func TestDebugViewReportsHistoryScrollRestoration(t *testing.T) {
	harness, err := FromHTML(`<main><script>host:historySetScrollRestoration("manual")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().HistoryScrollRestoration(); got != "manual" {
		t.Fatalf("Debug().HistoryScrollRestoration() = %q, want %q", got, "manual")
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().HistoryScrollRestoration(); got != "auto" {
		t.Fatalf("nil Debug().HistoryScrollRestoration() = %q, want %q", got, "auto")
	}
}

func TestDebugViewReportsDocumentCookie(t *testing.T) {
	harness, err := FromHTML(`<main><script>host:setDocumentCookie("theme=dark"); host:setDocumentCookie("lang=en; Path=/")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().DocumentCookie(), "lang=en; theme=dark"; got != want {
		t.Fatalf("Debug().DocumentCookie() = %q, want %q", got, want)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().DocumentCookie(); got != "" {
		t.Fatalf("nil Debug().DocumentCookie() = %q, want empty", got)
	}
}

func TestHarnessBuilderCopiesConfigurationContract(t *testing.T) {
	localStorage := map[string]string{"token": "abc"}
	sessionStorage := map[string]string{"tab": "main"}
	matchMedia := map[string]bool{"(prefers-reduced-motion: reduce)": true}

	harness, err := NewHarnessBuilder().
		URL("https://example.test/").
		HTML("<main>ok</main>").
		LocalStorage(localStorage).
		SessionStorage(sessionStorage).
		MatchMedia(matchMedia).
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
	if got := harness.Debug().MatchMediaRules(); len(got) != 1 || !got["(prefers-reduced-motion: reduce)"] {
		t.Fatalf("Debug().MatchMediaRules() = %#v, want seeded rule", got)
	}
	if got, want := harness.Mocks().Storage().Local()["token"], "abc"; got != want {
		t.Fatalf("Storage().Local()[\"token\"] = %q, want %q", got, want)
	}
	if got, want := harness.Mocks().Storage().Session()["tab"], "main"; got != want {
		t.Fatalf("Storage().Session()[\"tab\"] = %q, want %q", got, want)
	}
	if got := harness.Mocks().MatchMedia().Rules(); len(got) != 1 || got[0].Query != "(prefers-reduced-motion: reduce)" || !got[0].Matches {
		t.Fatalf("MatchMedia().Rules() = %#v, want seeded rule", got)
	}
}

func TestDebugViewReportsClipboard(t *testing.T) {
	harness, err := NewHarnessBuilder().Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	if err := harness.WriteClipboard("copied text"); err != nil {
		t.Fatalf("WriteClipboard() error = %v", err)
	}
	if got, want := harness.Debug().Clipboard(), "copied text"; got != want {
		t.Fatalf("Debug().Clipboard() = %q, want %q", got, want)
	}

	writes := harness.Debug().ClipboardWrites()
	if len(writes) != 1 || writes[0] != "copied text" {
		t.Fatalf("Debug().ClipboardWrites() = %#v, want one write", writes)
	}
	writes[0] = "mutated"
	if fresh := harness.Debug().ClipboardWrites(); len(fresh) != 1 || fresh[0] != "copied text" {
		t.Fatalf("Debug().ClipboardWrites() reread = %#v, want original write", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().Clipboard(); got != "" {
		t.Fatalf("nil Debug().Clipboard() = %q, want empty", got)
	}
	if got := nilHarness.Debug().ClipboardWrites(); got != nil {
		t.Fatalf("nil Debug().ClipboardWrites() = %#v, want nil", got)
	}
}

func TestDebugViewReportsMatchMediaRules(t *testing.T) {
	harness, err := NewHarnessBuilder().
		MatchMedia(map[string]bool{"(prefers-reduced-motion: reduce)": true}).
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	rules := harness.Debug().MatchMediaRules()
	if got, want := rules["(prefers-reduced-motion: reduce)"], true; got != want {
		t.Fatalf("Debug().MatchMediaRules()[prefers-reduced-motion] = %v, want %v", got, want)
	}
	rules["(prefers-reduced-motion: reduce)"] = false
	if got, want := harness.Debug().MatchMediaRules()["(prefers-reduced-motion: reduce)"], true; got != want {
		t.Fatalf("Debug().MatchMediaRules() reread = %v, want %v", got, want)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().MatchMediaRules(); got != nil {
		t.Fatalf("nil Debug().MatchMediaRules() = %#v, want nil", got)
	}
}

func TestMatchMediaRulesReturnCopies(t *testing.T) {
	harness, err := NewHarnessBuilder().
		MatchMedia(map[string]bool{"(prefers-reduced-motion: reduce)": true}).
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	rules := harness.Mocks().MatchMedia().Rules()
	if len(rules) != 1 {
		t.Fatalf("MatchMedia().Rules() = %#v, want one rule", rules)
	}
	rules[0].Query = "mutated"
	if got := harness.Mocks().MatchMedia().Rules(); len(got) != 1 || got[0].Query != "(prefers-reduced-motion: reduce)" {
		t.Fatalf("MatchMedia().Rules() reread = %#v, want original rule", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Mocks().MatchMedia().Rules(); got != nil {
		t.Fatalf("nil Harness.Mocks().MatchMedia().Rules() = %#v, want nil", got)
	}
}

func TestDebugViewReportsWebStorage(t *testing.T) {
	harness, err := NewHarnessBuilder().
		LocalStorage(map[string]string{"theme": "dark"}).
		SessionStorage(map[string]string{"tab": "main"}).
		HTML(`<main><script>host:localStorageSetItem("accent", "blue"); host:sessionStorageRemoveItem("tab")</script></main>`).
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	if got := harness.Debug().DumpDOM(); got == "" {
		t.Fatalf("Debug().DumpDOM() = empty, want bootstrapped DOM")
	}

	local := harness.Debug().LocalStorage()
	if got, want := local["theme"], "dark"; got != want {
		t.Fatalf("Debug().LocalStorage()[theme] = %q, want %q", got, want)
	}
	if got, want := local["accent"], "blue"; got != want {
		t.Fatalf("Debug().LocalStorage()[accent] = %q, want %q", got, want)
	}
	local["theme"] = "mutated"
	if got, want := harness.Debug().LocalStorage()["theme"], "dark"; got != want {
		t.Fatalf("Debug().LocalStorage()[theme] after mutation = %q, want %q", got, want)
	}

	session := harness.Debug().SessionStorage()
	if len(session) != 0 {
		t.Fatalf("Debug().SessionStorage() = %#v, want empty after mutation", session)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().LocalStorage(); got != nil {
		t.Fatalf("nil Debug().LocalStorage() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().SessionStorage(); got != nil {
		t.Fatalf("nil Debug().SessionStorage() = %#v, want nil", got)
	}
}

func TestDebugViewReportsNavigationLog(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/start",
		`<main><script>host:locationAssign("/next"); host:locationReplace("/replace")</script></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got, want := harness.Debug().NavigationLog(), []string{
		"https://example.test/next",
		"https://example.test/replace",
	}; len(got) != len(want) {
		t.Fatalf("Debug().NavigationLog() = %#v, want %#v", got, want)
	} else {
		for i := range want {
			if got[i] != want[i] {
				t.Fatalf("Debug().NavigationLog()[%d] = %q, want %q", i, got[i], want[i])
			}
		}
	}

	logs := harness.Debug().NavigationLog()
	logs[0] = "mutated"
	fresh := harness.Debug().NavigationLog()
	if fresh[0] != "https://example.test/next" {
		t.Fatalf("Debug().NavigationLog()[0] after mutation = %q, want %q", fresh[0], "https://example.test/next")
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().NavigationLog(); got != nil {
		t.Fatalf("nil Debug().NavigationLog() = %#v, want nil", got)
	}
}

func TestDebugViewReportsLocationParts(t *testing.T) {
	harness, err := FromHTMLWithURL("https://example.test:8443/path/name?mode=full#step-1", "<main></main>")
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got, want := harness.Debug().LocationOrigin(), "https://example.test:8443"; got != want {
		t.Fatalf("Debug().LocationOrigin() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationProtocol(), "https:"; got != want {
		t.Fatalf("Debug().LocationProtocol() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationHost(), "example.test:8443"; got != want {
		t.Fatalf("Debug().LocationHost() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationHostname(), "example.test"; got != want {
		t.Fatalf("Debug().LocationHostname() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationPort(), "8443"; got != want {
		t.Fatalf("Debug().LocationPort() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationPathname(), "/path/name"; got != want {
		t.Fatalf("Debug().LocationPathname() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationSearch(), "?mode=full"; got != want {
		t.Fatalf("Debug().LocationSearch() = %q, want %q", got, want)
	}
	if got, want := harness.Debug().LocationHash(), "#step-1"; got != want {
		t.Fatalf("Debug().LocationHash() = %q, want %q", got, want)
	}

	var nilHarness *Harness
	for _, tc := range []struct {
		name string
		got  string
	}{
		{name: "LocationOrigin", got: nilHarness.Debug().LocationOrigin()},
		{name: "LocationProtocol", got: nilHarness.Debug().LocationProtocol()},
		{name: "LocationHost", got: nilHarness.Debug().LocationHost()},
		{name: "LocationHostname", got: nilHarness.Debug().LocationHostname()},
		{name: "LocationPort", got: nilHarness.Debug().LocationPort()},
		{name: "LocationPathname", got: nilHarness.Debug().LocationPathname()},
		{name: "LocationSearch", got: nilHarness.Debug().LocationSearch()},
		{name: "LocationHash", got: nilHarness.Debug().LocationHash()},
	} {
		if tc.got != "" {
			t.Fatalf("nil Debug().%s() = %q, want empty", tc.name, tc.got)
		}
	}
}

func TestLocationMockNavigationsReturnCopies(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/start",
		`<main><script>host:locationAssign("/next"); host:locationReplace("/replace")</script></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got, want := harness.Debug().NavigationLog(), []string{"https://example.test/next", "https://example.test/replace"}; len(got) != len(want) {
		t.Fatalf("Debug().NavigationLog() = %#v, want %#v", got, want)
	} else if got[0] != want[0] || got[1] != want[1] {
		t.Fatalf("Debug().NavigationLog() = %#v, want %#v", got, want)
	}

	navigations := harness.Mocks().Location().Navigations()
	if len(navigations) != 2 {
		t.Fatalf("Location().Navigations() = %#v, want 2 entries", navigations)
	}
	navigations[0] = "mutated"
	navigations = append(navigations, "extra")

	fresh := harness.Mocks().Location().Navigations()
	if len(fresh) != 2 {
		t.Fatalf("Location().Navigations() reread len = %d, want 2", len(fresh))
	}
	if fresh[0] != "https://example.test/next" || fresh[1] != "https://example.test/replace" {
		t.Fatalf("Location().Navigations() reread = %#v, want original entries", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Mocks().Location(); got != nil {
		t.Fatalf("nil Harness.Mocks().Location() = %#v, want nil", got)
	}
}

func TestFetchMockSnapshotsReturnCopies(t *testing.T) {
	harness, err := FromHTML(`<main></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	harness.Mocks().Fetch().RespondText("https://example.test/api/message", 200, "ok")
	harness.Mocks().Fetch().Fail("https://example.test/api/broken", "boom")
	if _, err := harness.Fetch("https://example.test/api/message"); err != nil {
		t.Fatalf("Fetch() error = %v", err)
	}
	if _, err := harness.Fetch("https://example.test/api/broken"); err == nil {
		t.Fatalf("Fetch() broken error = nil, want failure")
	}

	calls := harness.Mocks().Fetch().Calls()
	if len(calls) != 2 {
		t.Fatalf("Fetch().Calls() = %#v, want 2 entries", calls)
	}
	calls[0].URL = "mutated"
	calls = append(calls, FetchCall{URL: "extra"})

	responses := harness.Mocks().Fetch().Responses()
	if len(responses) != 1 {
		t.Fatalf("Fetch().Responses() = %#v, want 1 entry", responses)
	}
	responses[0].URL = "mutated"
	responses[0].Body = "mutated"
	responses = append(responses, FetchResponseRule{URL: "extra"})

	errors := harness.Mocks().Fetch().Errors()
	if len(errors) != 1 {
		t.Fatalf("Fetch().Errors() = %#v, want 1 entry", errors)
	}
	errors[0].URL = "mutated"
	errors[0].Message = "mutated"
	errors = append(errors, FetchErrorRule{URL: "extra"})

	freshCalls := harness.Mocks().Fetch().Calls()
	if len(freshCalls) != 2 || freshCalls[0].URL != "https://example.test/api/message" || freshCalls[1].URL != "https://example.test/api/broken" {
		t.Fatalf("Fetch().Calls() reread = %#v, want original entries", freshCalls)
	}
	freshResponses := harness.Mocks().Fetch().Responses()
	if len(freshResponses) != 1 || freshResponses[0].URL != "https://example.test/api/message" || freshResponses[0].Body != "ok" {
		t.Fatalf("Fetch().Responses() reread = %#v, want original response rule", freshResponses)
	}
	freshErrors := harness.Mocks().Fetch().Errors()
	if len(freshErrors) != 1 || freshErrors[0].URL != "https://example.test/api/broken" || freshErrors[0].Message != "boom" {
		t.Fatalf("Fetch().Errors() reread = %#v, want original error rule", freshErrors)
	}

	var nilHarness *Harness
	if got := nilHarness.Mocks().Fetch(); got != nil {
		t.Fatalf("nil Harness.Mocks().Fetch() = %#v, want nil", got)
	}
}

func TestDebugViewReportsFetchCalls(t *testing.T) {
	harness, err := FromHTML(`<main></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	harness.Mocks().Fetch().RespondText("https://example.test/api/message", 200, "ok")
	harness.Mocks().Fetch().Fail("https://example.test/api/broken", "boom")
	if _, err := harness.Fetch("https://example.test/api/message"); err != nil {
		t.Fatalf("Fetch() error = %v", err)
	}
	if _, err := harness.Fetch("https://example.test/api/broken"); err == nil {
		t.Fatalf("Fetch() broken error = nil, want failure")
	}

	calls := harness.Debug().FetchCalls()
	if len(calls) != 2 || calls[0].URL != "https://example.test/api/message" || calls[1].URL != "https://example.test/api/broken" {
		t.Fatalf("Debug().FetchCalls() = %#v, want two captured requests", calls)
	}

	calls[0].URL = "mutated"
	if fresh := harness.Debug().FetchCalls(); len(fresh) != 2 || fresh[0].URL != "https://example.test/api/message" || fresh[1].URL != "https://example.test/api/broken" {
		t.Fatalf("Debug().FetchCalls() reread = %#v, want original request", fresh)
	}

	responses := harness.Debug().FetchResponseRules()
	if len(responses) != 1 || responses[0].URL != "https://example.test/api/message" || responses[0].Status != 200 || responses[0].Body != "ok" {
		t.Fatalf("Debug().FetchResponseRules() = %#v, want one response rule", responses)
	}
	responses[0].URL = "mutated"
	if fresh := harness.Debug().FetchResponseRules(); len(fresh) != 1 || fresh[0].URL != "https://example.test/api/message" || fresh[0].Body != "ok" {
		t.Fatalf("Debug().FetchResponseRules() reread = %#v, want original response rule", fresh)
	}

	errors := harness.Debug().FetchErrorRules()
	if len(errors) != 1 || errors[0].URL != "https://example.test/api/broken" || errors[0].Message != "boom" {
		t.Fatalf("Debug().FetchErrorRules() = %#v, want one error rule", errors)
	}
	errors[0].URL = "mutated"
	if fresh := harness.Debug().FetchErrorRules(); len(fresh) != 1 || fresh[0].URL != "https://example.test/api/broken" || fresh[0].Message != "boom" {
		t.Fatalf("Debug().FetchErrorRules() reread = %#v, want original error rule", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().FetchCalls(); got != nil {
		t.Fatalf("nil Debug().FetchCalls() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().FetchResponseRules(); got != nil {
		t.Fatalf("nil Debug().FetchResponseRules() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().FetchErrorRules(); got != nil {
		t.Fatalf("nil Debug().FetchErrorRules() = %#v, want nil", got)
	}
}

func TestDebugViewReportsActionCalls(t *testing.T) {
	harness, err := NewHarnessBuilder().
		HTML(`<main></main>`).
		MatchMedia(map[string]bool{"(prefers-reduced-motion: reduce)": true}).
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	if err := harness.Open("https://example.test/popup"); err != nil {
		t.Fatalf("Open() error = %v", err)
	}
	if err := harness.Close(); err != nil {
		t.Fatalf("Close() error = %v", err)
	}
	if err := harness.Print(); err != nil {
		t.Fatalf("Print() error = %v", err)
	}
	if err := harness.ScrollTo(4, 5); err != nil {
		t.Fatalf("ScrollTo() error = %v", err)
	}
	if err := harness.ScrollBy(2, -1); err != nil {
		t.Fatalf("ScrollBy() error = %v", err)
	}
	if got, err := harness.MatchMedia("(prefers-reduced-motion: reduce)"); err != nil || !got {
		t.Fatalf("MatchMedia() = (%v, %v), want (true, nil)", got, err)
	}

	openCalls := harness.Debug().OpenCalls()
	if len(openCalls) != 1 || openCalls[0].URL != "https://example.test/popup" {
		t.Fatalf("Debug().OpenCalls() = %#v, want one open call", openCalls)
	}
	openCalls[0].URL = "mutated"
	if fresh := harness.Debug().OpenCalls(); len(fresh) != 1 || fresh[0].URL != "https://example.test/popup" {
		t.Fatalf("Debug().OpenCalls() reread = %#v, want original open call", fresh)
	}

	if closeCalls := harness.Debug().CloseCalls(); len(closeCalls) != 1 {
		t.Fatalf("Debug().CloseCalls() = %#v, want one close call", closeCalls)
	}
	if printCalls := harness.Debug().PrintCalls(); len(printCalls) != 1 {
		t.Fatalf("Debug().PrintCalls() = %#v, want one print call", printCalls)
	}
	scrollCalls := harness.Debug().ScrollCalls()
	if len(scrollCalls) != 2 {
		t.Fatalf("Debug().ScrollCalls() = %#v, want two scroll calls", scrollCalls)
	}
	scrollCalls[0].X = 99
	if fresh := harness.Debug().ScrollCalls(); len(fresh) != 2 || fresh[0].X != 4 || fresh[1].Method != ScrollMethodBy {
		t.Fatalf("Debug().ScrollCalls() reread = %#v, want original scroll calls", fresh)
	}
	if matchMediaCalls := harness.Debug().MatchMediaCalls(); len(matchMediaCalls) != 1 || matchMediaCalls[0].Query != "(prefers-reduced-motion: reduce)" {
		t.Fatalf("Debug().MatchMediaCalls() = %#v, want one query call", matchMediaCalls)
	}
	if fresh := harness.Debug().MatchMediaCalls(); len(fresh) != 1 || fresh[0].Query != "(prefers-reduced-motion: reduce)" {
		t.Fatalf("Debug().MatchMediaCalls() reread = %#v, want original query call", fresh)
	}

	harness.Mocks().MatchMedia().RecordListenerCall("(prefers-reduced-motion: reduce)", "change")
	listeners := harness.Debug().MatchMediaListenerCalls()
	if len(listeners) != 1 || listeners[0].Query != "(prefers-reduced-motion: reduce)" || listeners[0].Method != "change" {
		t.Fatalf("Debug().MatchMediaListenerCalls() = %#v, want one listener call", listeners)
	}
	listeners[0].Method = "mutated"
	if fresh := harness.Debug().MatchMediaListenerCalls(); len(fresh) != 1 || fresh[0].Method != "change" {
		t.Fatalf("Debug().MatchMediaListenerCalls() reread = %#v, want original listener call", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().OpenCalls(); got != nil {
		t.Fatalf("nil Debug().OpenCalls() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().CloseCalls(); got != nil {
		t.Fatalf("nil Debug().CloseCalls() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().PrintCalls(); got != nil {
		t.Fatalf("nil Debug().PrintCalls() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().ScrollCalls(); got != nil {
		t.Fatalf("nil Debug().ScrollCalls() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().MatchMediaCalls(); got != nil {
		t.Fatalf("nil Debug().MatchMediaCalls() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().MatchMediaListenerCalls(); got != nil {
		t.Fatalf("nil Debug().MatchMediaListenerCalls() = %#v, want nil", got)
	}
}

func TestDebugViewReportsDialogMessages(t *testing.T) {
	harness, err := FromHTML(`<main></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	harness.Mocks().Dialogs().QueueConfirm(true)
	harness.Mocks().Dialogs().QueuePromptText("Ada")

	if err := harness.Alert("hello"); err != nil {
		t.Fatalf("Alert() error = %v", err)
	}
	if _, err := harness.Confirm("Continue?"); err != nil {
		t.Fatalf("Confirm() error = %v", err)
	}
	if _, _, err := harness.Prompt("Your name?"); err != nil {
		t.Fatalf("Prompt() error = %v", err)
	}

	alerts := harness.Debug().DialogAlerts()
	if len(alerts) != 1 || alerts[0] != "hello" {
		t.Fatalf("Debug().DialogAlerts() = %#v, want one alert", alerts)
	}
	alerts[0] = "mutated"
	if fresh := harness.Debug().DialogAlerts(); len(fresh) != 1 || fresh[0] != "hello" {
		t.Fatalf("Debug().DialogAlerts() reread = %#v, want original alert", fresh)
	}

	confirms := harness.Debug().DialogConfirmMessages()
	if len(confirms) != 1 || confirms[0] != "Continue?" {
		t.Fatalf("Debug().DialogConfirmMessages() = %#v, want one confirm message", confirms)
	}
	confirms[0] = "mutated"
	if fresh := harness.Debug().DialogConfirmMessages(); len(fresh) != 1 || fresh[0] != "Continue?" {
		t.Fatalf("Debug().DialogConfirmMessages() reread = %#v, want original confirm message", fresh)
	}

	prompts := harness.Debug().DialogPromptMessages()
	if len(prompts) != 1 || prompts[0] != "Your name?" {
		t.Fatalf("Debug().DialogPromptMessages() = %#v, want one prompt message", prompts)
	}
	prompts[0] = "mutated"
	if fresh := harness.Debug().DialogPromptMessages(); len(fresh) != 1 || fresh[0] != "Your name?" {
		t.Fatalf("Debug().DialogPromptMessages() reread = %#v, want original prompt message", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().DialogAlerts(); got != nil {
		t.Fatalf("nil Debug().DialogAlerts() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().DialogConfirmMessages(); got != nil {
		t.Fatalf("nil Debug().DialogConfirmMessages() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().DialogPromptMessages(); got != nil {
		t.Fatalf("nil Debug().DialogPromptMessages() = %#v, want nil", got)
	}
}

func TestDebugViewReportsDownloadAndFileInputCaptures(t *testing.T) {
	harness, err := FromHTML(`<main><input id="upload" type="file"></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.CaptureDownload("report.csv", []byte("downloaded bytes")); err != nil {
		t.Fatalf("CaptureDownload() error = %v", err)
	}
	if err := harness.SetFiles("#upload", []string{"report.csv", "archive.zip"}); err != nil {
		t.Fatalf("SetFiles() error = %v", err)
	}

	artifacts := harness.Debug().DownloadArtifacts()
	if len(artifacts) != 1 || artifacts[0].FileName != "report.csv" || string(artifacts[0].Bytes) != "downloaded bytes" {
		t.Fatalf("Debug().DownloadArtifacts() = %#v, want one download capture", artifacts)
	}
	artifacts[0].FileName = "mutated"
	artifacts[0].Bytes[0] = 'X'
	if fresh := harness.Debug().DownloadArtifacts(); len(fresh) != 1 || fresh[0].FileName != "report.csv" || string(fresh[0].Bytes) != "downloaded bytes" {
		t.Fatalf("Debug().DownloadArtifacts() reread = %#v, want original capture", fresh)
	}

	selections := harness.Debug().FileInputSelections()
	if len(selections) != 1 || selections[0].Selector != "#upload" || len(selections[0].Files) != 2 {
		t.Fatalf("Debug().FileInputSelections() = %#v, want one file-input selection", selections)
	}
	selections[0].Selector = "mutated"
	selections[0].Files[0] = "mutated"
	if fresh := harness.Debug().FileInputSelections(); len(fresh) != 1 || fresh[0].Selector != "#upload" || fresh[0].Files[0] != "report.csv" {
		t.Fatalf("Debug().FileInputSelections() reread = %#v, want original selection", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().DownloadArtifacts(); got != nil {
		t.Fatalf("nil Debug().DownloadArtifacts() = %#v, want nil", got)
	}
	if got := nilHarness.Debug().FileInputSelections(); got != nil {
		t.Fatalf("nil Debug().FileInputSelections() = %#v, want nil", got)
	}
}

func TestDebugViewReportsStorageEvents(t *testing.T) {
	harness, err := NewHarnessBuilder().
		HTML(`<main><script>host:localStorageSetItem("accent", "blue"); host:sessionStorageRemoveItem("tab"); host:localStorageClear()</script></main>`).
		LocalStorage(map[string]string{"theme": "dark"}).
		SessionStorage(map[string]string{"tab": "main"}).
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	if got := harness.Debug().DumpDOM(); got == "" {
		t.Fatalf("Debug().DumpDOM() = %q, want non-empty", got)
	}

	events := harness.Debug().StorageEvents()
	if len(events) != 5 {
		t.Fatalf("Debug().StorageEvents() = %#v, want five events", events)
	}
	if events[0].Scope != "local" || events[0].Op != "seed" || events[0].Key != "theme" || events[0].Value != "dark" {
		t.Fatalf("Debug().StorageEvents()[0] = %#v, want local seed", events[0])
	}
	if events[1].Scope != "session" || events[1].Op != "seed" || events[1].Key != "tab" || events[1].Value != "main" {
		t.Fatalf("Debug().StorageEvents()[1] = %#v, want session seed", events[1])
	}
	if events[2].Scope != "local" || events[2].Op != "set" || events[2].Key != "accent" || events[2].Value != "blue" {
		t.Fatalf("Debug().StorageEvents()[2] = %#v, want local set", events[2])
	}
	if events[3].Scope != "session" || events[3].Op != "remove" || events[3].Key != "tab" {
		t.Fatalf("Debug().StorageEvents()[3] = %#v, want session remove", events[3])
	}
	if events[4].Scope != "local" || events[4].Op != "clear" {
		t.Fatalf("Debug().StorageEvents()[4] = %#v, want local clear", events[4])
	}

	events[0].Value = "mutated"
	if fresh := harness.Debug().StorageEvents(); len(fresh) != 5 || fresh[0].Value != "dark" {
		t.Fatalf("Debug().StorageEvents() reread = %#v, want original events", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().StorageEvents(); got != nil {
		t.Fatalf("nil Debug().StorageEvents() = %#v, want nil", got)
	}
}

func TestOtherMockSnapshotsReturnCopies(t *testing.T) {
	harness, err := NewHarnessBuilder().
		HTML(`<main></main>`).
		MatchMedia(map[string]bool{"(prefers-reduced-motion: reduce)": true}).
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	if err := harness.Open("https://example.test/popup"); err != nil {
		t.Fatalf("Open() error = %v", err)
	}
	if err := harness.Close(); err != nil {
		t.Fatalf("Close() error = %v", err)
	}
	if err := harness.Print(); err != nil {
		t.Fatalf("Print() error = %v", err)
	}
	if err := harness.ScrollTo(7, 9); err != nil {
		t.Fatalf("ScrollTo() error = %v", err)
	}
	if err := harness.ScrollBy(2, -1); err != nil {
		t.Fatalf("ScrollBy() error = %v", err)
	}
	if got, err := harness.MatchMedia("(prefers-reduced-motion: reduce)"); err != nil {
		t.Fatalf("MatchMedia() error = %v", err)
	} else if !got {
		t.Fatalf("MatchMedia() = false, want true")
	}

	openCalls := harness.Mocks().Open().Calls()
	if len(openCalls) != 1 {
		t.Fatalf("Open().Calls() = %#v, want 1 entry", openCalls)
	}
	openCalls[0].URL = "mutated"
	openCalls = append(openCalls, OpenCall{URL: "extra"})

	closeCalls := harness.Mocks().Close().Calls()
	if len(closeCalls) != 1 {
		t.Fatalf("Close().Calls() = %#v, want 1 entry", closeCalls)
	}
	closeCalls = append(closeCalls, CloseCall{})

	printCalls := harness.Mocks().Print().Calls()
	if len(printCalls) != 1 {
		t.Fatalf("Print().Calls() = %#v, want 1 entry", printCalls)
	}
	printCalls = append(printCalls, PrintCall{})

	scrollCalls := harness.Mocks().Scroll().Calls()
	if len(scrollCalls) != 2 {
		t.Fatalf("Scroll().Calls() = %#v, want 2 entries", scrollCalls)
	}
	scrollCalls[0].Method = ScrollMethod("mutated")
	scrollCalls[0].X = 100
	scrollCalls[0].Y = 200
	scrollCalls = append(scrollCalls, ScrollCall{Method: ScrollMethod("extra"), X: 3, Y: 4})

	matchMediaCalls := harness.Mocks().MatchMedia().Calls()
	if len(matchMediaCalls) != 1 {
		t.Fatalf("MatchMedia().Calls() = %#v, want 1 entry", matchMediaCalls)
	}
	matchMediaCalls[0].Query = "mutated"
	matchMediaCalls = append(matchMediaCalls, MatchMediaCall{Query: "extra"})

	freshOpenCalls := harness.Mocks().Open().Calls()
	if len(freshOpenCalls) != 1 || freshOpenCalls[0].URL != "https://example.test/popup" {
		t.Fatalf("Open().Calls() reread = %#v, want original open call", freshOpenCalls)
	}
	freshCloseCalls := harness.Mocks().Close().Calls()
	if len(freshCloseCalls) != 1 {
		t.Fatalf("Close().Calls() reread = %#v, want 1 entry", freshCloseCalls)
	}
	freshPrintCalls := harness.Mocks().Print().Calls()
	if len(freshPrintCalls) != 1 {
		t.Fatalf("Print().Calls() reread = %#v, want 1 entry", freshPrintCalls)
	}
	freshScrollCalls := harness.Mocks().Scroll().Calls()
	if len(freshScrollCalls) != 2 || freshScrollCalls[0].Method != ScrollMethodTo || freshScrollCalls[0].X != 7 || freshScrollCalls[0].Y != 9 || freshScrollCalls[1].Method != ScrollMethodBy || freshScrollCalls[1].X != 2 || freshScrollCalls[1].Y != -1 {
		t.Fatalf("Scroll().Calls() reread = %#v, want original scroll calls", freshScrollCalls)
	}
	freshMatchMediaCalls := harness.Mocks().MatchMedia().Calls()
	if len(freshMatchMediaCalls) != 1 || freshMatchMediaCalls[0].Query != "(prefers-reduced-motion: reduce)" {
		t.Fatalf("MatchMedia().Calls() reread = %#v, want original query", freshMatchMediaCalls)
	}

	var nilHarness *Harness
	if got := nilHarness.Mocks().Open(); got != nil {
		t.Fatalf("nil Harness.Mocks().Open() = %#v, want nil", got)
	}
	if got := nilHarness.Mocks().Close(); got != nil {
		t.Fatalf("nil Harness.Mocks().Close() = %#v, want nil", got)
	}
	if got := nilHarness.Mocks().Print(); got != nil {
		t.Fatalf("nil Harness.Mocks().Print() = %#v, want nil", got)
	}
	if got := nilHarness.Mocks().Scroll(); got != nil {
		t.Fatalf("nil Harness.Mocks().Scroll() = %#v, want nil", got)
	}
	if got := nilHarness.Mocks().MatchMedia(); got != nil {
		t.Fatalf("nil Harness.Mocks().MatchMedia() = %#v, want nil", got)
	}
}

func TestClipboardDownloadAndFileInputSnapshotsReturnCopies(t *testing.T) {
	harness, err := FromHTML(`<main><input id="upload" type="file"></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.WriteClipboard("copied text"); err != nil {
		t.Fatalf("WriteClipboard() error = %v", err)
	}
	if err := harness.WriteClipboard("more text"); err != nil {
		t.Fatalf("WriteClipboard() second error = %v", err)
	}
	if err := harness.CaptureDownload("report.csv", []byte("downloaded bytes")); err != nil {
		t.Fatalf("CaptureDownload() error = %v", err)
	}
	if err := harness.SetFiles("#upload", []string{"report.csv", "archive.zip"}); err != nil {
		t.Fatalf("SetFiles(#upload) error = %v", err)
	}

	clipboardWrites := harness.Mocks().Clipboard().Writes()
	if len(clipboardWrites) != 2 {
		t.Fatalf("Clipboard().Writes() = %#v, want 2 entries", clipboardWrites)
	}
	clipboardWrites[0] = "mutated"
	clipboardWrites = append(clipboardWrites, "extra")

	downloadArtifacts := harness.Mocks().Downloads().Artifacts()
	if len(downloadArtifacts) != 1 {
		t.Fatalf("Downloads().Artifacts() = %#v, want 1 entry", downloadArtifacts)
	}
	downloadArtifacts[0].FileName = "mutated.csv"
	if len(downloadArtifacts[0].Bytes) > 0 {
		downloadArtifacts[0].Bytes[0] ^= 0xff
	}
	downloadArtifacts = append(downloadArtifacts, DownloadCapture{FileName: "extra.csv"})

	fileSelections := harness.Mocks().FileInput().Selections()
	if len(fileSelections) != 1 {
		t.Fatalf("FileInput().Selections() = %#v, want 1 entry", fileSelections)
	}
	fileSelections[0].Selector = "mutated"
	fileSelections[0].Files[0] = "mutated.txt"
	fileSelections = append(fileSelections, FileInputSelection{Selector: "extra"})

	freshClipboardWrites := harness.Mocks().Clipboard().Writes()
	if len(freshClipboardWrites) != 2 || freshClipboardWrites[0] != "copied text" || freshClipboardWrites[1] != "more text" {
		t.Fatalf("Clipboard().Writes() reread = %#v, want original writes", freshClipboardWrites)
	}
	freshDownloadArtifacts := harness.Mocks().Downloads().Artifacts()
	if len(freshDownloadArtifacts) != 1 || freshDownloadArtifacts[0].FileName != "report.csv" || string(freshDownloadArtifacts[0].Bytes) != "downloaded bytes" {
		t.Fatalf("Downloads().Artifacts() reread = %#v, want original download capture", freshDownloadArtifacts)
	}
	freshFileSelections := harness.Mocks().FileInput().Selections()
	if len(freshFileSelections) != 1 || freshFileSelections[0].Selector != "#upload" || len(freshFileSelections[0].Files) != 2 || freshFileSelections[0].Files[0] != "report.csv" || freshFileSelections[0].Files[1] != "archive.zip" {
		t.Fatalf("FileInput().Selections() reread = %#v, want original file selection", freshFileSelections)
	}

	var nilHarness *Harness
	if got := nilHarness.Mocks().Clipboard(); got != nil {
		t.Fatalf("nil Harness.Mocks().Clipboard() = %#v, want nil", got)
	}
	if got := nilHarness.Mocks().Downloads(); got != nil {
		t.Fatalf("nil Harness.Mocks().Downloads() = %#v, want nil", got)
	}
	if got := nilHarness.Mocks().FileInput(); got != nil {
		t.Fatalf("nil Harness.Mocks().FileInput() = %#v, want nil", got)
	}
}

func TestDebugViewReportsInteractionsCopy(t *testing.T) {
	harness, err := FromHTML(`<main><button id="cta">Go</button><input id="name"></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Focus("#name"); err != nil {
		t.Fatalf("Focus(#name) error = %v", err)
	}
	if err := harness.Click("#cta"); err != nil {
		t.Fatalf("Click(#cta) error = %v", err)
	}

	interactions := harness.Debug().Interactions()
	if len(interactions) != 2 {
		t.Fatalf("Debug().Interactions() len = %d, want 2", len(interactions))
	}
	if interactions[0].Kind != InteractionKindFocus || interactions[0].Selector != "#name" {
		t.Fatalf("Debug().Interactions()[0] = %#v, want focus #name", interactions[0])
	}
	if interactions[1].Kind != InteractionKindClick || interactions[1].Selector != "#cta" {
		t.Fatalf("Debug().Interactions()[1] = %#v, want click #cta", interactions[1])
	}

	interactions[0].Selector = "mutated"
	interactions[1].Kind = InteractionKindBlur
	fresh := harness.Debug().Interactions()
	if fresh[0].Selector != "#name" {
		t.Fatalf("Debug().Interactions()[0].Selector after mutation = %q, want %q", fresh[0].Selector, "#name")
	}
	if fresh[1].Kind != InteractionKindClick {
		t.Fatalf("Debug().Interactions()[1].Kind after mutation = %q, want %q", fresh[1].Kind, InteractionKindClick)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().Interactions(); got != nil {
		t.Fatalf("nil Debug().Interactions() = %#v, want nil", got)
	}
}

func TestDebugViewReportsEventListenersCopy(t *testing.T) {
	harness, err := FromHTML(`<main><button id="btn">Go</button><div id="out"></div><script>host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#out", "beforeend", "<span>once</span>")', "capture", true); host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#out", "beforeend", "<span>stay</span>")', "bubble")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	listeners := harness.Debug().EventListeners()
	if len(listeners) != 2 {
		t.Fatalf("Debug().EventListeners() len = %d, want 2", len(listeners))
	}
	if listeners[0].NodeID == 0 || listeners[0].NodeID != listeners[1].NodeID {
		t.Fatalf("Debug().EventListeners() node ids = %#v, want same non-zero node id", listeners)
	}
	if listeners[0].Event != "click" || listeners[0].Phase != "capture" || !listeners[0].Once {
		t.Fatalf("Debug().EventListeners()[0] = %#v, want capture once click listener", listeners[0])
	}
	if listeners[1].Event != "click" || listeners[1].Phase != "bubble" || listeners[1].Once {
		t.Fatalf("Debug().EventListeners()[1] = %#v, want bubble persistent click listener", listeners[1])
	}

	listeners[0].Source = "mutated"
	fresh := harness.Debug().EventListeners()
	if fresh[0].Source != `host:insertAdjacentHTML("#out", "beforeend", "<span>once</span>")` {
		t.Fatalf("Debug().EventListeners() reread source = %q, want original source", fresh[0].Source)
	}

	if err := harness.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}
	after := harness.Debug().EventListeners()
	if len(after) != 1 {
		t.Fatalf("Debug().EventListeners() after click len = %d, want 1", len(after))
	}
	if after[0].Phase != "bubble" || after[0].Once {
		t.Fatalf("Debug().EventListeners() after click = %#v, want persistent bubble listener", after[0])
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().EventListeners(); got != nil {
		t.Fatalf("nil Debug().EventListeners() = %#v, want nil", got)
	}
}

func TestDebugViewReportsPendingMicrotasksCopy(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out">seed</div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().PendingMicrotasks(); len(got) != 0 {
		t.Fatalf("Debug().PendingMicrotasks() = %#v, want empty", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().PendingMicrotasks(); got != nil {
		t.Fatalf("nil Debug().PendingMicrotasks() = %#v, want nil", got)
	}
}

func TestDebugViewReportsCookieJarCopy(t *testing.T) {
	harness, err := FromHTML(`<main><script>host:setDocumentCookie("theme=dark"); host:setDocumentCookie("lang=en")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}
	jar := harness.Debug().CookieJar()
	if len(jar) != 2 || jar["theme"] != "dark" || jar["lang"] != "en" {
		t.Fatalf("Debug().CookieJar() = %#v, want theme/lang snapshot", jar)
	}
	jar["theme"] = "mutated"
	fresh := harness.Debug().CookieJar()
	if fresh["theme"] != "dark" {
		t.Fatalf("Debug().CookieJar() reread = %#v, want original cookie jar", fresh)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().CookieJar(); got != nil {
		t.Fatalf("nil Debug().CookieJar() = %#v, want nil", got)
	}
}

func TestDebugViewReportsDOMReadinessAndErrors(t *testing.T) {
	harness, err := FromHTML(`<main><button id="btn">Go</button></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got := harness.Debug().DOMReady(); got {
		t.Fatalf("Debug().DOMReady() before DOM access = %v, want false", got)
	}
	if got := harness.Debug().DOMError(); got != "" {
		t.Fatalf("Debug().DOMError() before DOM access = %q, want empty", got)
	}

	if err := harness.AssertExists("#btn"); err != nil {
		t.Fatalf("AssertExists(#btn) error = %v", err)
	}
	if got := harness.Debug().DOMReady(); !got {
		t.Fatalf("Debug().DOMReady() after DOM access = %v, want true", got)
	}
	if got := harness.Debug().DOMError(); got != "" {
		t.Fatalf("Debug().DOMError() after DOM access = %q, want empty", got)
	}
}

func TestDebugViewReportsDOMFailureDetails(t *testing.T) {
	harness, err := FromHTML(`<main><div id="broken"></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("#broken"); err == nil {
		t.Fatalf("AssertExists(#broken) error = nil, want DOM parse failure")
	}
	if got := harness.Debug().DOMReady(); got {
		t.Fatalf("Debug().DOMReady() after parse failure = %v, want false", got)
	}
	if got := harness.Debug().DOMError(); got == "" {
		t.Fatalf("Debug().DOMError() after parse failure = %q, want parse error", got)
	}
}

func TestDebugViewReportsLastInlineScriptHTML(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out">old</div><script id="boot">host:setInnerHTML("#out", "<span>new</span>")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().LastInlineScriptHTML(), `<script id="boot">host:setInnerHTML("#out", "<span>new</span>")</script>`; got != want {
		t.Fatalf("Debug().LastInlineScriptHTML() = %q, want %q", got, want)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().LastInlineScriptHTML(); got != "" {
		t.Fatalf("nil Debug().LastInlineScriptHTML() = %q, want empty", got)
	}
}

func TestDebugViewReportsInitialHTMLWithoutBootstrapping(t *testing.T) {
	harness, err := FromHTML(`<main><script>host:setInnerHTML("#out", "ok")</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().InitialHTML(), `<main><script>host:setInnerHTML("#out", "ok")</script></main>`; got != want {
		t.Fatalf("Debug().InitialHTML() = %q, want %q", got, want)
	}
	if got := harness.Debug().DOMReady(); got {
		t.Fatalf("Debug().DOMReady() after InitialHTML() = %v, want false", got)
	}

	var nilHarness *Harness
	if got := nilHarness.Debug().InitialHTML(); got != "" {
		t.Fatalf("nil Debug().InitialHTML() = %q, want empty", got)
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

func TestDialogMockSnapshotsReturnCopies(t *testing.T) {
	harness, err := FromHTML("<main></main>")
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	harness.Mocks().Dialogs().QueueConfirm(true)
	harness.Mocks().Dialogs().QueuePromptText("typed")
	harness.Mocks().Dialogs().QueuePromptCancel()

	if err := harness.Alert("alert"); err != nil {
		t.Fatalf("Alert() error = %v", err)
	}
	if got, err := harness.Confirm("confirm?"); err != nil {
		t.Fatalf("Confirm() error = %v", err)
	} else if !got {
		t.Fatalf("Confirm() = false, want true")
	}
	if got, submitted, err := harness.Prompt("prompt?"); err != nil {
		t.Fatalf("Prompt() #1 error = %v", err)
	} else if got != "typed" || !submitted {
		t.Fatalf("Prompt() #1 = (%q, %v), want (%q, true)", got, submitted, "typed")
	}
	if got, submitted, err := harness.Prompt("cancel?"); err != nil {
		t.Fatalf("Prompt() #2 error = %v", err)
	} else if got != "" || submitted {
		t.Fatalf("Prompt() #2 = (%q, %v), want (\"\", false)", got, submitted)
	}

	alerts := harness.Mocks().Dialogs().Alerts()
	if len(alerts) != 1 || alerts[0] != "alert" {
		t.Fatalf("Dialogs().Alerts() = %#v, want [\"alert\"]", alerts)
	}
	alerts[0] = "mutated"
	alerts = append(alerts, "extra")

	confirms := harness.Mocks().Dialogs().ConfirmMessages()
	if len(confirms) != 1 || confirms[0] != "confirm?" {
		t.Fatalf("Dialogs().ConfirmMessages() = %#v, want [\"confirm?\"]", confirms)
	}
	confirms[0] = "mutated"
	confirms = append(confirms, "extra")

	prompts := harness.Mocks().Dialogs().PromptMessages()
	if len(prompts) != 2 || prompts[0] != "prompt?" || prompts[1] != "cancel?" {
		t.Fatalf("Dialogs().PromptMessages() = %#v, want [\"prompt?\", \"cancel?\"]", prompts)
	}
	prompts[0] = "mutated"
	prompts = append(prompts, "extra")

	freshAlerts := harness.Mocks().Dialogs().Alerts()
	if len(freshAlerts) != 1 || freshAlerts[0] != "alert" {
		t.Fatalf("Dialogs().Alerts() reread = %#v, want [\"alert\"]", freshAlerts)
	}
	freshConfirms := harness.Mocks().Dialogs().ConfirmMessages()
	if len(freshConfirms) != 1 || freshConfirms[0] != "confirm?" {
		t.Fatalf("Dialogs().ConfirmMessages() reread = %#v, want [\"confirm?\"]", freshConfirms)
	}
	freshPrompts := harness.Mocks().Dialogs().PromptMessages()
	if len(freshPrompts) != 2 || freshPrompts[0] != "prompt?" || freshPrompts[1] != "cancel?" {
		t.Fatalf("Dialogs().PromptMessages() reread = %#v, want [\"prompt?\", \"cancel?\"]", freshPrompts)
	}

	var nilHarness *Harness
	if got := nilHarness.Mocks().Dialogs(); got != nil {
		t.Fatalf("nil Harness.Mocks().Dialogs() = %#v, want nil", got)
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

func TestInlineScriptsCanDriveWebStorageThroughPublicFacade(t *testing.T) {
	harness, err := NewHarnessBuilder().
		URL("https://example.test/app").
		LocalStorage(map[string]string{"theme": "dark"}).
		SessionStorage(map[string]string{"tab": "main"}).
		HTML(`<main><div id="out"></div><script>host:setTextContent("#out", expr(host:localStorageGetItem("theme"))); host:localStorageSetItem("accent", "blue"); host:sessionStorageRemoveItem("tab"); host:localStorageClear()</script></main>`).
		Build()
	if err != nil {
		t.Fatalf("Build() error = %v", err)
	}

	if err := harness.AssertText("#out", "dark"); err != nil {
		t.Fatalf("AssertText(#out, dark) error = %v", err)
	}
	if got := harness.Mocks().Storage().Local(); len(got) != 0 {
		t.Fatalf("Storage().Local() after bootstrap web storage script = %#v, want empty", got)
	}
	if got := harness.Mocks().Storage().Session(); len(got) != 0 {
		t.Fatalf("Storage().Session() after bootstrap web storage script = %#v, want empty", got)
	}
	events := harness.Mocks().Storage().Events()
	if len(events) != 5 || events[0].Op != "seed" || events[1].Op != "seed" || events[2].Op != "set" || events[3].Op != "remove" || events[4].Op != "clear" {
		t.Fatalf("Storage().Events() after web storage script = %#v, want seed/seed/set/remove/clear", events)
	}
}

func TestInlineScriptsCanUseTreeMutationHelpersThroughPublicFacade(t *testing.T) {
	harness, err := FromHTML(`<main><div id="src"><span>old</span></div><div id="out">before</div><script>host:replaceChildren("#out", "<em>fresh</em>"); host:cloneNode("#src", true)</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="src"><span>old</span></div><div id="src"><span>old</span></div><div id="out"><em>fresh</em></div><script>host:replaceChildren("#out", "<em>fresh</em>"); host:cloneNode("#src", true)</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after tree mutation host helpers = %q, want %q", got, want)
	}
}

func TestInlineScriptsCanReadTextContentThroughPublicFacade(t *testing.T) {
	harness, err := FromHTML(`<main><div id="src">seed</div><div id="out"></div><script>host:setTextContent("#out", expr(host:textContent("#src")))</script></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="src">seed</div><div id="out">seed</div><script>host:setTextContent("#out", expr(host:textContent("#src")))</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after textContent getter = %q, want %q", got, want)
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

	harness.Mocks().Storage().SeedLocal("theme", "dark")
	harness.Mocks().Storage().SeedSession("tab", "main")

	events := harness.Mocks().Storage().Events()
	if len(events) != 4 || events[0].Scope != "local" || events[0].Op != "seed" || events[0].Key != "local" || events[0].Value != "value" || events[1].Scope != "session" || events[1].Op != "seed" || events[1].Key != "session" || events[1].Value != "value" || events[2].Scope != "local" || events[2].Op != "seed" || events[2].Key != "theme" || events[2].Value != "dark" || events[3].Scope != "session" || events[3].Op != "seed" || events[3].Key != "tab" || events[3].Value != "main" {
		t.Fatalf("Storage().Events() = %#v, want four storage events", events)
	}
	events[0].Value = "mutated"
	events = append(events, StorageEvent{Scope: "extra"})

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
	freshEvents := harness.Mocks().Storage().Events()
	if len(freshEvents) != 4 || freshEvents[0].Op != "seed" || freshEvents[0].Value != "value" || freshEvents[1].Op != "seed" || freshEvents[1].Value != "value" || freshEvents[2].Op != "seed" || freshEvents[2].Value != "dark" || freshEvents[3].Op != "seed" || freshEvents[3].Value != "main" {
		t.Fatalf("Storage().Events() reread = %#v, want original storage events", freshEvents)
	}

	var nilHarness *Harness
	if got := nilHarness.Mocks().Storage(); got != nil {
		t.Fatalf("nil Harness.Mocks().Storage() = %#v, want nil", got)
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
	harness, err := FromHTML(`<main><div id="panel" data-kind="panel"><a id="nav" href="/next" data-role="nav">Go</a><input id="name" type="text"><p id="flag" hidden></p><span id="meta" data-tags="alpha beta gamma" data-locale="en-US" data-note="prefix-middle-suffix" data-code="abc123"></span></div></main>`)
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
	if err := harness.AssertExists("span[data-tags~=beta]"); err != nil {
		t.Fatalf("AssertExists(span[data-tags~=beta]) error = %v", err)
	}
	if err := harness.AssertExists("span[data-locale|=en]"); err != nil {
		t.Fatalf("AssertExists(span[data-locale|=en]) error = %v", err)
	}
	if err := harness.AssertExists("span[data-note^=prefix]"); err != nil {
		t.Fatalf("AssertExists(span[data-note^=prefix]) error = %v", err)
	}
	if err := harness.AssertExists("span[data-note$=suffix]"); err != nil {
		t.Fatalf("AssertExists(span[data-note$=suffix]) error = %v", err)
	}
	if err := harness.AssertExists("span[data-note*=middle]"); err != nil {
		t.Fatalf("AssertExists(span[data-note*=middle]) error = %v", err)
	}
	if err := harness.AssertExists("span[data-tags~=BETA i]"); err != nil {
		t.Fatalf("AssertExists(span[data-tags~=BETA i]) error = %v", err)
	}
	if err := harness.AssertExists("input[type=TEXT i]"); err != nil {
		t.Fatalf("AssertExists(input[type=TEXT i]) error = %v", err)
	}
	if err := harness.AssertExists("a[data-role=missing]"); err == nil {
		t.Fatalf("AssertExists(a[data-role=missing]) error = nil, want no match")
	}
	if err := harness.AssertExists("span[data-tags~=delta]"); err == nil {
		t.Fatalf("AssertExists(span[data-tags~=delta]) error = nil, want no match")
	}
	if err := harness.AssertExists("span[data-tags~=BETA s]"); err == nil {
		t.Fatalf("AssertExists(span[data-tags~=BETA s]) error = nil, want no match")
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

func TestInlineScriptsCanSetLocationUsernameAndPasswordThroughPublicActions(t *testing.T) {
	markup := `<main><script>host:locationSet("username", "alice"); host:locationSet("password", "secret")</script></main>`
	harness, err := FromHTMLWithURL("https://example.test/start", markup)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got, want := harness.Debug().DumpDOM(), markup; got != want {
		t.Fatalf("Debug().DumpDOM() after location credential updates = %q, want %q", got, want)
	}
	if got, want := harness.URL(), "https://alice:secret@example.test/start"; got != want {
		t.Fatalf("URL() after location credential updates = %q, want %q", got, want)
	}
	if got, want := harness.Mocks().Location().Navigations(), []string{
		"https://alice@example.test/start",
		"https://alice:secret@example.test/start",
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

func TestInlineScriptsCanReadLocationPropertiesThroughPublicActions(t *testing.T) {
	tests := []struct {
		name   string
		script string
		want   string
	}{
		{
			name:   "href",
			script: `host:setTextContent("#out", expr(host:locationHref()))`,
			want:   "https://example.test:8443/path/name?mode=full#step-1",
		},
		{
			name:   "origin",
			script: `host:setTextContent("#out", expr(host:locationOrigin()))`,
			want:   "https://example.test:8443",
		},
		{
			name:   "protocol",
			script: `host:setTextContent("#out", expr(host:locationProtocol()))`,
			want:   "https:",
		},
		{
			name:   "host",
			script: `host:setTextContent("#out", expr(host:locationHost()))`,
			want:   "example.test:8443",
		},
		{
			name:   "hostname",
			script: `host:setTextContent("#out", expr(host:locationHostname()))`,
			want:   "example.test",
		},
		{
			name:   "port",
			script: `host:setTextContent("#out", expr(host:locationPort()))`,
			want:   "8443",
		},
		{
			name:   "pathname",
			script: `host:setTextContent("#out", expr(host:locationPathname()))`,
			want:   "/path/name",
		},
		{
			name:   "search",
			script: `host:setTextContent("#out", expr(host:locationSearch()))`,
			want:   "?mode=full",
		},
		{
			name:   "hash",
			script: `host:setTextContent("#out", expr(host:locationHash()))`,
			want:   "#step-1",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			markup := `<main><div id="out"></div><script>` + tc.script + `</script></main>`
			harness, err := FromHTMLWithURL("https://example.test:8443/path/name?mode=full#step-1", markup)
			if err != nil {
				t.Fatalf("FromHTMLWithURL() error = %v", err)
			}

			if err := harness.AssertText("#out", tc.want); err != nil {
				t.Fatalf("AssertText(#out, %q) error = %v", tc.want, err)
			}
			if got, want := harness.URL(), "https://example.test:8443/path/name?mode=full#step-1"; got != want {
				t.Fatalf("URL() after %s getter = %q, want %q", tc.name, got, want)
			}
		})
	}
}

func TestInlineScriptsRejectLocationGetterArgumentsThroughPublicActions(t *testing.T) {
	harness, err := FromHTML(`<main><div id="out">old</div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.WriteHTML(`<main><script>host:locationHref("extra")</script></main>`); err == nil {
		t.Fatalf("WriteHTML(locationHref extra) error = nil, want validation failure")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("WriteHTML(locationHref extra) error = %#v, want DOM error", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">old</div></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after rejected locationHref getter = %q, want %q", got, want)
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

func TestWriteHTMLRestoresSessionStateOnHostFailure(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/app#target",
		`<main><button id="btn">Go</button><div id="target">target</div><script>host:setWindowName("alpha"); host:setDocumentCookie("theme=dark"); host:historySetScrollRestoration("manual"); host:historyPushState("step-1", "", "/step-1#target")</script></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if got := harness.Debug().TargetNodeID(); got == 0 {
		t.Fatalf("Debug().TargetNodeID() before failed WriteHTML = %d, want targeted node", got)
	}
	if err := harness.Focus("#btn"); err != nil {
		t.Fatalf("Focus(#btn) error = %v", err)
	}
	if err := harness.ScrollTo(11, 17); err != nil {
		t.Fatalf("ScrollTo() error = %v", err)
	}

	if err := harness.WriteHTML(`<main><script>host:setWindowName("beta"); host:setDocumentCookie("lang=en"); host:historySetScrollRestoration("auto"); host:locationAssign("/next"); host:doesNotExist()</script></main>`); err == nil {
		t.Fatalf("WriteHTML(host failure) error = nil, want host error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("WriteHTML(host failure) error = %#v, want DOM error", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><button id="btn">Go</button><div id="target">target</div><script>host:setWindowName("alpha"); host:setDocumentCookie("theme=dark"); host:historySetScrollRestoration("manual"); host:historyPushState("step-1", "", "/step-1#target")</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after failed WriteHTML = %q, want %q", got, want)
	}
	if got, want := harness.HTML(), `<main><button id="btn">Go</button><div id="target">target</div><script>host:setWindowName("alpha"); host:setDocumentCookie("theme=dark"); host:historySetScrollRestoration("manual"); host:historyPushState("step-1", "", "/step-1#target")</script></main>`; got != want {
		t.Fatalf("HTML() after failed WriteHTML = %q, want %q", got, want)
	}
	if got := harness.Debug().TargetNodeID(); got == 0 {
		t.Fatalf("Debug().TargetNodeID() after failed WriteHTML = %d, want targeted node", got)
	}
	if got := harness.Debug().FocusedSelector(); got != "#btn" {
		t.Fatalf("Debug().FocusedSelector() after failed WriteHTML = %q, want %q", got, "#btn")
	}
	if gotX, gotY := harness.Debug().ScrollPosition(); gotX != 11 || gotY != 17 {
		t.Fatalf("Debug().ScrollPosition() after failed WriteHTML = (%d, %d), want (11, 17)", gotX, gotY)
	}
	if got, want := harness.Debug().WindowName(), "alpha"; got != want {
		t.Fatalf("Debug().WindowName() after failed WriteHTML = %q, want %q", got, want)
	}
	if got, want := harness.Debug().DocumentCookie(), "theme=dark"; got != want {
		t.Fatalf("Debug().DocumentCookie() after failed WriteHTML = %q, want %q", got, want)
	}
	if got, want := harness.Debug().HistoryScrollRestoration(), "manual"; got != want {
		t.Fatalf("Debug().HistoryScrollRestoration() after failed WriteHTML = %q, want %q", got, want)
	}
	if got, want := harness.URL(), "https://example.test/step-1#target"; got != want {
		t.Fatalf("URL() after failed WriteHTML = %q, want %q", got, want)
	}
	if got := harness.Debug().HistoryLength(); got != 2 {
		t.Fatalf("Debug().HistoryLength() after failed WriteHTML = %d, want 2", got)
	}
	if got, ok := harness.Debug().HistoryState(); !ok || got != "step-1" {
		t.Fatalf("Debug().HistoryState() after failed WriteHTML = (%q, %v), want (\"step-1\", true)", got, ok)
	}
	if got, want := harness.Debug().NavigationLog(), []string{"https://example.test/step-1#target"}; len(got) != len(want) {
		t.Fatalf("Debug().NavigationLog() after failed WriteHTML = %#v, want %#v", got, want)
	} else if got[0] != want[0] {
		t.Fatalf("Debug().NavigationLog()[0] after failed WriteHTML = %q, want %q", got[0], want[0])
	}
}

func TestInlineScriptsRejectReadOnlyLocationOriginThroughPublicActions(t *testing.T) {
	harness, err := FromHTMLWithURL("https://example.test/start", `<main><div id="out">old</div></main>`)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if err := harness.WriteHTML(`<main><script>host:locationSet("origin", "https://other.test/")</script></main>`); err == nil {
		t.Fatalf("WriteHTML(locationSet origin) error = nil, want read-only failure")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("WriteHTML(locationSet origin) error = %#v, want DOM error", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><div id="out">old</div></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after rejected origin assignment = %q, want %q", got, want)
	}
	if got, want := harness.HTML(), `<main><div id="out">old</div></main>`; got != want {
		t.Fatalf("HTML() after rejected origin assignment = %q, want %q", got, want)
	}
	if got, want := harness.URL(), "https://example.test/start"; got != want {
		t.Fatalf("URL() after rejected origin assignment = %q, want %q", got, want)
	}
	if got, want := harness.Debug().NavigationLog(), []string{}; len(got) != len(want) {
		t.Fatalf("Debug().NavigationLog() after rejected origin assignment = %#v, want %#v", got, want)
	}
}

func TestInlineScriptsRejectInvalidHistoryScrollRestorationThroughPublicActions(t *testing.T) {
	harness, err := FromHTMLWithURL(
		"https://example.test/app",
		`<main><script>host:setWindowName("alpha"); host:setDocumentCookie("theme=dark"); host:historySetScrollRestoration("manual"); host:historyPushState("step-1", "", "/step-1")</script></main>`,
	)
	if err != nil {
		t.Fatalf("FromHTMLWithURL() error = %v", err)
	}

	if err := harness.WriteHTML(`<main><script>host:historySetScrollRestoration("sideways")</script></main>`); err == nil {
		t.Fatalf("WriteHTML(historySetScrollRestoration sideways) error = nil, want validation failure")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("WriteHTML(historySetScrollRestoration sideways) error = %#v, want DOM error", err)
	}

	if got, want := harness.Debug().DumpDOM(), `<main><script>host:setWindowName("alpha"); host:setDocumentCookie("theme=dark"); host:historySetScrollRestoration("manual"); host:historyPushState("step-1", "", "/step-1")</script></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after rejected history scroll restoration = %q, want %q", got, want)
	}
	if got, want := harness.HTML(), `<main><script>host:setWindowName("alpha"); host:setDocumentCookie("theme=dark"); host:historySetScrollRestoration("manual"); host:historyPushState("step-1", "", "/step-1")</script></main>`; got != want {
		t.Fatalf("HTML() after rejected history scroll restoration = %q, want %q", got, want)
	}
	if got, want := harness.Debug().WindowName(), "alpha"; got != want {
		t.Fatalf("Debug().WindowName() after rejected history scroll restoration = %q, want %q", got, want)
	}
	if got, want := harness.Debug().DocumentCookie(), "theme=dark"; got != want {
		t.Fatalf("Debug().DocumentCookie() after rejected history scroll restoration = %q, want %q", got, want)
	}
	if got, want := harness.Debug().HistoryScrollRestoration(), "manual"; got != want {
		t.Fatalf("Debug().HistoryScrollRestoration() after rejected history scroll restoration = %q, want %q", got, want)
	}
	if got, want := harness.URL(), "https://example.test/step-1"; got != want {
		t.Fatalf("URL() after rejected history scroll restoration = %q, want %q", got, want)
	}
	if got := harness.Debug().HistoryLength(); got != 2 {
		t.Fatalf("Debug().HistoryLength() after rejected history scroll restoration = %d, want 2", got)
	}
	if got, ok := harness.Debug().HistoryState(); !ok || got != "step-1" {
		t.Fatalf("Debug().HistoryState() after rejected history scroll restoration = (%q, %v), want (\"step-1\", true)", got, ok)
	}
	if got, want := harness.Debug().NavigationLog(), []string{"https://example.test/step-1"}; len(got) != len(want) {
		t.Fatalf("Debug().NavigationLog() after rejected history scroll restoration = %#v, want %#v", got, want)
	} else if got[0] != want[0] {
		t.Fatalf("Debug().NavigationLog()[0] after rejected history scroll restoration = %q, want %q", got[0], want[0])
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
	if got, err := harness.TextContent("#target"); err != nil {
		t.Fatalf("TextContent(#target) error = %v", err)
	} else if want := `Helloworld`; got != want {
		t.Fatalf("TextContent(#target) = %q, want %q", got, want)
	}

	if err := harness.SetInnerHTML("#target", `<em id="next">updated</em>tail`); err != nil {
		t.Fatalf("SetInnerHTML(#target) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<section id="wrap"><div id="target"><em id="next">updated</em>tail</div><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("Debug().DumpDOM() after SetInnerHTML = %q, want %q", got, want)
	}
	if err := harness.SetTextContent("#target", `plain <text> & more`); err != nil {
		t.Fatalf("SetTextContent(#target) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<section id="wrap"><div id="target">plain &lt;text&gt; &amp; more</div><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("Debug().DumpDOM() after SetTextContent = %q, want %q", got, want)
	}
	if got, err := harness.TextContent("#target"); err != nil {
		t.Fatalf("TextContent(#target) after SetTextContent error = %v", err)
	} else if want := `plain <text> & more`; got != want {
		t.Fatalf("TextContent(#target) after SetTextContent = %q, want %q", got, want)
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
	if got, want := harness.Debug().DumpDOM(), `<section id="wrap"><a id="bb"></a><div id="target"><i id="ab">a</i>plain &lt;text&gt; &amp; more<i id="be">b</i></div><a id="ae"></a><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("Debug().DumpDOM() after InsertAdjacentHTML = %q, want %q", got, want)
	}

	if err := harness.SetOuterHTML("#target", `<article id="next-outer">n</article><aside id="extra"></aside>`); err != nil {
		t.Fatalf("SetOuterHTML(#target) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<section id="wrap"><a id="bb"></a><article id="next-outer">n</article><aside id="extra"></aside><a id="ae"></a><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("Debug().DumpDOM() after SetOuterHTML = %q, want %q", got, want)
	}
}

func TestMutationContractsCloneNodeDuplicatesSubtree(t *testing.T) {
	harness, err := FromHTML(`<main><div id="source"><span id="child">text</span></div><p id="tail">tail</p></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.CloneNode("#source", true); err != nil {
		t.Fatalf("CloneNode(#source, true) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<main><div id="source"><span id="child">text</span></div><div id="source"><span id="child">text</span></div><p id="tail">tail</p></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after CloneNode = %q, want %q", got, want)
	}
	if err := harness.AssertText("main > div + div > span", "text"); err != nil {
		t.Fatalf("AssertText(main > div + div > span) after CloneNode error = %v", err)
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
	if _, err := harness.TextContent("#missing"); err == nil {
		t.Fatalf("TextContent(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("TextContent(#missing) error = %#v, want DOM error", err)
	}

	if err := harness.ReplaceChildren("#missing", "<p>x</p>"); err == nil {
		t.Fatalf("ReplaceChildren(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("ReplaceChildren(#missing) error = %#v, want DOM error", err)
	}
	if err := harness.SetInnerHTML("#missing", "<p>x</p>"); err == nil {
		t.Fatalf("SetInnerHTML(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("SetInnerHTML(#missing) error = %#v, want DOM error", err)
	}
	if err := harness.SetTextContent("#missing", "x"); err == nil {
		t.Fatalf("SetTextContent(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("SetTextContent(#missing) error = %#v, want DOM error", err)
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
	if err := harness.CloneNode("#missing", true); err == nil {
		t.Fatalf("CloneNode(#missing) error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("CloneNode(#missing) error = %#v, want DOM error", err)
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

func TestMutationContractsSetTextContentPreservesFocusAndClearsTargetDescendants(t *testing.T) {
	harness, err := FromHTML(`<main><div id="target"><span id="child">x</span></div><p id="other">y</p></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.Focus("#target"); err != nil {
		t.Fatalf("Focus(#target) error = %v", err)
	}
	if err := harness.Navigate("#child"); err != nil {
		t.Fatalf("Navigate(#child) error = %v", err)
	}
	if err := harness.SetTextContent("#target", "plain"); err != nil {
		t.Fatalf("SetTextContent(#target) error = %v", err)
	}
	if got, want := harness.Debug().FocusedSelector(), "#target"; got != want {
		t.Fatalf("Debug().FocusedSelector() after SetTextContent = %q, want %q", got, want)
	}
	if err := harness.AssertExists(":target"); err == nil {
		t.Fatalf("AssertExists(:target) after SetTextContent error = nil, want no match")
	}
	if got, want := harness.Debug().DumpDOM(), `<main><div id="target">plain</div><p id="other">y</p></main>`; got != want {
		t.Fatalf("Debug().DumpDOM() after SetTextContent = %q, want %q", got, want)
	}
}

func TestMutationContractsSetTextContentUpdatesTextareaDefaultValue(t *testing.T) {
	harness, err := FromHTML(`<form id="profile"><textarea id="bio">Base</textarea><button id="reset" type="reset">Reset</button></form>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.SetTextContent("#bio", "Draft"); err != nil {
		t.Fatalf("SetTextContent(#bio) error = %v", err)
	}
	if got, want := harness.Debug().DumpDOM(), `<form id="profile"><textarea id="bio">Draft</textarea><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("Debug().DumpDOM() after SetTextContent = %q, want %q", got, want)
	}

	if err := harness.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) error = %v", err)
	}
	if got, err := harness.TextContent("#bio"); err != nil {
		t.Fatalf("TextContent(#bio) after reset click error = %v", err)
	} else if got != "Draft" {
		t.Fatalf("TextContent(#bio) after reset click = %q, want %q", got, "Draft")
	}

	if err := harness.TypeText("#bio", "User"); err != nil {
		t.Fatalf("TypeText(#bio) error = %v", err)
	}
	if err := harness.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) after TypeText error = %v", err)
	}
	if got, err := harness.TextContent("#bio"); err != nil {
		t.Fatalf("TextContent(#bio) after reset click following TypeText error = %v", err)
	} else if got != "Draft" {
		t.Fatalf("TextContent(#bio) after reset click following TypeText = %q, want %q", got, "Draft")
	}
}

func TestMutationContractsTextareaChildMutationsUpdateResetDefaultValue(t *testing.T) {
	harness, err := FromHTML(`<form id="profile"><textarea id="bio">Base</textarea><button id="reset" type="reset">Reset</button></form>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.ReplaceChildren("#bio", "Draft"); err != nil {
		t.Fatalf("ReplaceChildren(#bio) error = %v", err)
	}
	if err := harness.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) after ReplaceChildren error = %v", err)
	}
	if got, err := harness.TextContent("#bio"); err != nil {
		t.Fatalf("TextContent(#bio) after ReplaceChildren reset error = %v", err)
	} else if got != "Draft" {
		t.Fatalf("TextContent(#bio) after ReplaceChildren reset = %q, want %q", got, "Draft")
	}

	if err := harness.SetInnerHTML("#bio", "Fresh"); err != nil {
		t.Fatalf("SetInnerHTML(#bio) second update error = %v", err)
	}
	if err := harness.InsertAdjacentHTML("#bio", "beforeend", `<span id="bang">!</span>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(#bio,beforeend) error = %v", err)
	}
	if err := harness.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) after InsertAdjacentHTML error = %v", err)
	}
	if got, err := harness.TextContent("#bio"); err != nil {
		t.Fatalf("TextContent(#bio) after InsertAdjacentHTML reset error = %v", err)
	} else if got != "Fresh!" {
		t.Fatalf("TextContent(#bio) after InsertAdjacentHTML reset = %q, want %q", got, "Fresh!")
	}

	if err := harness.SetInnerHTML("#bio", "Fresh"); err != nil {
		t.Fatalf("SetInnerHTML(#bio) third update error = %v", err)
	}
	if err := harness.InsertAdjacentHTML("#bio", "beforeend", `<span id="bang">!</span>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(#bio,beforeend) second error = %v", err)
	}
	if err := harness.RemoveNode("#bang"); err != nil {
		t.Fatalf("RemoveNode(#bang) error = %v", err)
	}
	if err := harness.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) after RemoveNode error = %v", err)
	}
	if got, err := harness.TextContent("#bio"); err != nil {
		t.Fatalf("TextContent(#bio) after RemoveNode reset error = %v", err)
	} else if got != "Fresh" {
		t.Fatalf("TextContent(#bio) after RemoveNode reset = %q, want %q", got, "Fresh")
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
	if _, err := nilHarness.TextContent("#target"); err == nil {
		t.Fatalf("nil Harness.TextContent() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.TextContent() error = %#v, want DOM error", err)
	}
	if err := nilHarness.ReplaceChildren("#target", "<p>x</p>"); err == nil {
		t.Fatalf("nil Harness.ReplaceChildren() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.ReplaceChildren() error = %#v, want DOM error", err)
	}
	if err := nilHarness.SetInnerHTML("#target", "<p>x</p>"); err == nil {
		t.Fatalf("nil Harness.SetInnerHTML() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.SetInnerHTML() error = %#v, want DOM error", err)
	}
	if err := nilHarness.SetTextContent("#target", "x"); err == nil {
		t.Fatalf("nil Harness.SetTextContent() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.SetTextContent() error = %#v, want DOM error", err)
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
	if err := nilHarness.CloneNode("#target", true); err == nil {
		t.Fatalf("nil Harness.CloneNode() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("nil Harness.CloneNode() error = %#v, want DOM error", err)
	}

	zeroSessionHarness := &Harness{}
	if _, err := zeroSessionHarness.InnerHTML("#target"); err == nil {
		t.Fatalf("Harness{nil session}.InnerHTML() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("Harness{nil session}.InnerHTML() error = %#v, want DOM error", err)
	}
	if _, err := zeroSessionHarness.TextContent("#target"); err == nil {
		t.Fatalf("Harness{nil session}.TextContent() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("Harness{nil session}.TextContent() error = %#v, want DOM error", err)
	}
	if err := zeroSessionHarness.CloneNode("#target", true); err == nil {
		t.Fatalf("Harness{nil session}.CloneNode() error = nil, want DOM error")
	} else if got, ok := err.(Error); !ok || got.Kind != ErrorKindDOM {
		t.Fatalf("Harness{nil session}.CloneNode() error = %#v, want DOM error", err)
	}
}
