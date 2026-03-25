package browsertester

import (
	rt "browsertester/internal/runtime"
)

type DebugView struct {
	session *rt.Session
}

func (v DebugView) URL() string {
	if v.session == nil {
		return ""
	}
	return v.session.URL()
}

func (v DebugView) LocationOrigin() string {
	if v.session == nil {
		return ""
	}
	value, err := v.session.LocationOrigin()
	if err != nil {
		return ""
	}
	return value
}

func (v DebugView) LocationProtocol() string {
	if v.session == nil {
		return ""
	}
	value, err := v.session.LocationProtocol()
	if err != nil {
		return ""
	}
	return value
}

func (v DebugView) LocationHost() string {
	if v.session == nil {
		return ""
	}
	value, err := v.session.LocationHost()
	if err != nil {
		return ""
	}
	return value
}

func (v DebugView) LocationHostname() string {
	if v.session == nil {
		return ""
	}
	value, err := v.session.LocationHostname()
	if err != nil {
		return ""
	}
	return value
}

func (v DebugView) LocationPort() string {
	if v.session == nil {
		return ""
	}
	value, err := v.session.LocationPort()
	if err != nil {
		return ""
	}
	return value
}

func (v DebugView) LocationPathname() string {
	if v.session == nil {
		return ""
	}
	value, err := v.session.LocationPathname()
	if err != nil {
		return ""
	}
	return value
}

func (v DebugView) LocationSearch() string {
	if v.session == nil {
		return ""
	}
	value, err := v.session.LocationSearch()
	if err != nil {
		return ""
	}
	return value
}

func (v DebugView) LocationHash() string {
	if v.session == nil {
		return ""
	}
	value, err := v.session.LocationHash()
	if err != nil {
		return ""
	}
	return value
}

func (v DebugView) HTML() string {
	if v.session == nil {
		return ""
	}
	return v.session.HTML()
}

func (v DebugView) InitialHTML() string {
	if v.session == nil {
		return ""
	}
	return v.session.InitialHTML()
}

func (v DebugView) DumpDOM() string {
	if v.session == nil {
		return ""
	}
	return v.session.DumpDOM()
}

func (v DebugView) NodeCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.NodeCount()
}

func (v DebugView) ScriptCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.ScriptCount()
}

func (v DebugView) ImageCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.ImageCount()
}

func (v DebugView) FormCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.FormCount()
}

func (v DebugView) SelectCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.SelectCount()
}

func (v DebugView) TemplateCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.TemplateCount()
}

func (v DebugView) TableCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.TableCount()
}

func (v DebugView) ButtonCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.ButtonCount()
}

func (v DebugView) TextAreaCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.TextAreaCount()
}

func (v DebugView) InputCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.InputCount()
}

func (v DebugView) FieldsetCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.FieldsetCount()
}

func (v DebugView) LegendCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.LegendCount()
}

func (v DebugView) OutputCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.OutputCount()
}

func (v DebugView) LabelCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.LabelCount()
}

func (v DebugView) ProgressCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.ProgressCount()
}

func (v DebugView) MeterCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.MeterCount()
}

func (v DebugView) AudioCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.AudioCount()
}

func (v DebugView) VideoCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.VideoCount()
}

func (v DebugView) IframeCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.IframeCount()
}

func (v DebugView) EmbedCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.EmbedCount()
}

func (v DebugView) TrackCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.TrackCount()
}

func (v DebugView) PictureCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.PictureCount()
}

func (v DebugView) SourceCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.SourceCount()
}

func (v DebugView) DialogCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.DialogCount()
}

func (v DebugView) DetailsCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.DetailsCount()
}

func (v DebugView) SummaryCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.SummaryCount()
}

func (v DebugView) SectionCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.SectionCount()
}

func (v DebugView) MainCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.MainCount()
}

func (v DebugView) ArticleCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.ArticleCount()
}

func (v DebugView) NavCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.NavCount()
}

func (v DebugView) AsideCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.AsideCount()
}

func (v DebugView) FigureCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.FigureCount()
}

func (v DebugView) FigcaptionCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.FigcaptionCount()
}

func (v DebugView) HeaderCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.HeaderCount()
}

func (v DebugView) FooterCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.FooterCount()
}

func (v DebugView) AddressCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.AddressCount()
}

func (v DebugView) BlockquoteCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.BlockquoteCount()
}

func (v DebugView) ParagraphCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.ParagraphCount()
}

func (v DebugView) PreCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.PreCount()
}

func (v DebugView) MarkCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.MarkCount()
}

func (v DebugView) QCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.QCount()
}

func (v DebugView) CiteCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.CiteCount()
}

func (v DebugView) AbbrCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.AbbrCount()
}

func (v DebugView) StrongCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.StrongCount()
}

func (v DebugView) SpanCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.SpanCount()
}

func (v DebugView) DataCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.DataCount()
}

func (v DebugView) DfnCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.DfnCount()
}

func (v DebugView) KbdCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.KbdCount()
}

func (v DebugView) SampCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.SampCount()
}

func (v DebugView) RubyCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.RubyCount()
}

func (v DebugView) RtCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.RtCount()
}

func (v DebugView) VarCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.VarCount()
}

func (v DebugView) CodeCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.CodeCount()
}

func (v DebugView) SmallCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.SmallCount()
}

func (v DebugView) TimeCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.TimeCount()
}

func (v DebugView) OptionCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.OptionCount()
}

func (v DebugView) SelectedOptionCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.SelectedOptionCount()
}

func (v DebugView) OptgroupCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.OptgroupCount()
}

func (v DebugView) LinkCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.LinkCount()
}

func (v DebugView) AnchorCount() int {
	if v.session == nil {
		return 0
	}
	return v.session.AnchorCount()
}

func (v DebugView) OptionLabels() []OptionLabel {
	if v.session == nil {
		return nil
	}
	labels := v.session.OptionLabels()
	out := make([]OptionLabel, len(labels))
	for i := range labels {
		out[i] = OptionLabel{
			NodeID: labels[i].NodeID,
			Label:  labels[i].Label,
		}
	}
	return out
}

func (v DebugView) SelectedOptionLabels() []OptionLabel {
	if v.session == nil {
		return nil
	}
	labels := v.session.SelectedOptionLabels()
	out := make([]OptionLabel, len(labels))
	for i := range labels {
		out[i] = OptionLabel{
			NodeID: labels[i].NodeID,
			Label:  labels[i].Label,
		}
	}
	return out
}

func (v DebugView) OptgroupLabels() []OptgroupLabel {
	if v.session == nil {
		return nil
	}
	labels := v.session.OptgroupLabels()
	out := make([]OptgroupLabel, len(labels))
	for i := range labels {
		out[i] = OptgroupLabel{
			NodeID: labels[i].NodeID,
			Label:  labels[i].Label,
		}
	}
	return out
}

func (v DebugView) OptionValues() []OptionValue {
	if v.session == nil {
		return nil
	}
	values := v.session.OptionValues()
	out := make([]OptionValue, len(values))
	for i := range values {
		out[i] = OptionValue{
			NodeID: values[i].NodeID,
			Value:  values[i].Value,
		}
	}
	return out
}

func (v DebugView) SelectedOptionValues() []OptionValue {
	if v.session == nil {
		return nil
	}
	values := v.session.SelectedOptionValues()
	out := make([]OptionValue, len(values))
	for i := range values {
		out[i] = OptionValue{
			NodeID: values[i].NodeID,
			Value:  values[i].Value,
		}
	}
	return out
}

func (v DebugView) LastInlineScriptHTML() string {
	if v.session == nil {
		return ""
	}
	return v.session.LastInlineScriptHTML()
}

func (v DebugView) DOMReady() bool {
	if v.session == nil {
		return false
	}
	return v.session.DOMReady()
}

func (v DebugView) DOMError() string {
	if v.session == nil {
		return ""
	}
	return v.session.DOMError()
}

func (v DebugView) NowMs() int64 {
	if v.session == nil {
		return 0
	}
	return v.session.NowMs()
}

func (v DebugView) FocusedSelector() string {
	if v.session == nil {
		return ""
	}
	return v.session.FocusedSelector()
}

func (v DebugView) FocusedNodeID() int64 {
	if v.session == nil {
		return 0
	}
	return v.session.FocusedNodeID()
}

func (v DebugView) TargetNodeID() int64 {
	if v.session == nil {
		return 0
	}
	return v.session.TargetNodeID()
}

func (v DebugView) HistoryLength() int {
	if v.session == nil {
		return 0
	}
	return v.session.HistoryLength()
}

func (v DebugView) HistoryState() (string, bool) {
	if v.session == nil {
		return "null", false
	}
	return v.session.HistoryState()
}

func (v DebugView) HistoryScrollRestoration() string {
	if v.session == nil {
		return "auto"
	}
	return v.session.HistoryScrollRestoration()
}

func (v DebugView) ScrollPosition() (int64, int64) {
	if v.session == nil {
		return 0, 0
	}
	return v.session.ScrollPosition()
}

func (v DebugView) WindowName() string {
	if v.session == nil {
		return ""
	}
	return v.session.WindowName()
}

func (v DebugView) DocumentCookie() string {
	if v.session == nil {
		return ""
	}
	return v.session.DocumentCookie()
}

func (v DebugView) CookieJar() map[string]string {
	if v.session == nil {
		return nil
	}
	return v.session.CookieJar()
}

func (v DebugView) Clipboard() string {
	if v.session == nil {
		return ""
	}
	return v.session.Clipboard()
}

func (v DebugView) ClipboardWrites() []string {
	if v.session == nil {
		return nil
	}
	writes := v.session.ClipboardWrites()
	out := make([]string, len(writes))
	copy(out, writes)
	return out
}

func (v DebugView) FetchCalls() []FetchCall {
	if v.session == nil {
		return nil
	}
	calls := v.session.FetchCalls()
	out := make([]FetchCall, len(calls))
	for i := range calls {
		out[i] = FetchCall{URL: calls[i].URL}
	}
	return out
}

func (v DebugView) FetchResponseRules() []FetchResponseRule {
	if v.session == nil {
		return nil
	}
	rules := v.session.FetchResponseRules()
	out := make([]FetchResponseRule, len(rules))
	for i := range rules {
		out[i] = FetchResponseRule{
			URL:    rules[i].URL,
			Status: rules[i].Status,
			Body:   rules[i].Body,
		}
	}
	return out
}

func (v DebugView) FetchErrorRules() []FetchErrorRule {
	if v.session == nil {
		return nil
	}
	rules := v.session.FetchErrorRules()
	out := make([]FetchErrorRule, len(rules))
	for i := range rules {
		out[i] = FetchErrorRule{
			URL:     rules[i].URL,
			Message: rules[i].Message,
		}
	}
	return out
}

func (v DebugView) OpenCalls() []OpenCall {
	if v.session == nil {
		return nil
	}
	calls := v.session.OpenCalls()
	out := make([]OpenCall, len(calls))
	for i := range calls {
		out[i] = OpenCall{URL: calls[i].URL}
	}
	return out
}

func (v DebugView) CloseCalls() []CloseCall {
	if v.session == nil {
		return nil
	}
	calls := v.session.CloseCalls()
	out := make([]CloseCall, len(calls))
	for i := range calls {
		out[i] = CloseCall{}
	}
	return out
}

func (v DebugView) PrintCalls() []PrintCall {
	if v.session == nil {
		return nil
	}
	calls := v.session.PrintCalls()
	out := make([]PrintCall, len(calls))
	for i := range calls {
		out[i] = PrintCall{}
	}
	return out
}

func (v DebugView) ScrollCalls() []ScrollCall {
	if v.session == nil {
		return nil
	}
	calls := v.session.ScrollCalls()
	out := make([]ScrollCall, len(calls))
	for i := range calls {
		out[i] = ScrollCall{
			Method: ScrollMethod(calls[i].Method),
			X:      calls[i].X,
			Y:      calls[i].Y,
		}
	}
	return out
}

func (v DebugView) MatchMediaCalls() []MatchMediaCall {
	if v.session == nil {
		return nil
	}
	calls := v.session.MatchMediaCalls()
	out := make([]MatchMediaCall, len(calls))
	for i := range calls {
		out[i] = MatchMediaCall{Query: calls[i].Query}
	}
	return out
}

func (v DebugView) MatchMediaListenerCalls() []MatchMediaListenerCall {
	if v.session == nil {
		return nil
	}
	calls := v.session.MatchMediaListenerCalls()
	out := make([]MatchMediaListenerCall, len(calls))
	for i := range calls {
		out[i] = MatchMediaListenerCall{
			Query:  calls[i].Query,
			Method: calls[i].Method,
		}
	}
	return out
}

func (v DebugView) DownloadArtifacts() []DownloadCapture {
	if v.session == nil {
		return nil
	}
	artifacts := v.session.DownloadArtifacts()
	out := make([]DownloadCapture, len(artifacts))
	for i := range artifacts {
		bytes := make([]byte, len(artifacts[i].Bytes))
		copy(bytes, artifacts[i].Bytes)
		out[i] = DownloadCapture{
			FileName: artifacts[i].FileName,
			Bytes:    bytes,
		}
	}
	return out
}

func (v DebugView) FileInputSelections() []FileInputSelection {
	if v.session == nil {
		return nil
	}
	selections := v.session.FileInputSelections()
	out := make([]FileInputSelection, len(selections))
	for i := range selections {
		files := make([]string, len(selections[i].Files))
		copy(files, selections[i].Files)
		out[i] = FileInputSelection{
			Selector: selections[i].Selector,
			Files:    files,
		}
	}
	return out
}

func (v DebugView) StorageEvents() []StorageEvent {
	if v.session == nil {
		return nil
	}
	events := v.session.StorageEvents()
	out := make([]StorageEvent, len(events))
	for i := range events {
		out[i] = StorageEvent{
			Scope: events[i].Scope,
			Op:    events[i].Op,
			Key:   events[i].Key,
			Value: events[i].Value,
		}
	}
	return out
}

func (v DebugView) DialogAlerts() []string {
	if v.session == nil {
		return nil
	}
	return v.session.DialogAlerts()
}

func (v DebugView) DialogConfirmMessages() []string {
	if v.session == nil {
		return nil
	}
	return v.session.DialogConfirmMessages()
}

func (v DebugView) DialogPromptMessages() []string {
	if v.session == nil {
		return nil
	}
	return v.session.DialogPromptMessages()
}

func (v DebugView) MatchMediaRules() map[string]bool {
	if v.session == nil {
		return nil
	}
	return v.session.MatchMediaRules()
}

func (v DebugView) OpenFailure() string {
	if v.session == nil {
		return ""
	}
	return v.session.OpenFailure()
}

func (v DebugView) CloseFailure() string {
	if v.session == nil {
		return ""
	}
	return v.session.CloseFailure()
}

func (v DebugView) PrintFailure() string {
	if v.session == nil {
		return ""
	}
	return v.session.PrintFailure()
}

func (v DebugView) ScrollFailure() string {
	if v.session == nil {
		return ""
	}
	return v.session.ScrollFailure()
}

func (v DebugView) LocalStorage() map[string]string {
	if v.session == nil {
		return nil
	}
	return v.session.LocalStorage()
}

func (v DebugView) SessionStorage() map[string]string {
	if v.session == nil {
		return nil
	}
	return v.session.SessionStorage()
}

func (v DebugView) NavigationLog() []string {
	if v.session == nil {
		return nil
	}
	return v.session.NavigationLog()
}

func (v DebugView) HistoryEntries() []HistoryEntry {
	if v.session == nil {
		return nil
	}
	entries := v.session.HistoryEntries()
	out := make([]HistoryEntry, len(entries))
	for i := range entries {
		out[i] = HistoryEntry{
			URL:      entries[i].URL,
			State:    entries[i].State,
			HasState: entries[i].HasState,
		}
	}
	return out
}

func (v DebugView) HistoryIndex() int {
	if v.session == nil {
		return 0
	}
	return v.session.HistoryIndex()
}

func (v DebugView) VisitedURLs() []string {
	if v.session == nil {
		return nil
	}
	urls := v.session.VisitedURLs()
	out := make([]string, len(urls))
	copy(out, urls)
	return out
}

func (v DebugView) PendingTimers() []TimerSnapshot {
	if v.session == nil {
		return nil
	}
	timers := v.session.PendingTimers()
	out := make([]TimerSnapshot, len(timers))
	for i := range timers {
		out[i] = TimerSnapshot{
			ID:         timers[i].ID,
			Source:     timers[i].Source,
			DueAtMs:    timers[i].DueAtMs,
			Repeat:     timers[i].Repeat,
			IntervalMs: timers[i].IntervalMs,
		}
	}
	return out
}

func (v DebugView) PendingAnimationFrames() []AnimationFrameSnapshot {
	if v.session == nil {
		return nil
	}
	frames := v.session.PendingAnimationFrames()
	out := make([]AnimationFrameSnapshot, len(frames))
	for i := range frames {
		out[i] = AnimationFrameSnapshot{
			ID:     frames[i].ID,
			Source: frames[i].Source,
		}
	}
	return out
}

func (v DebugView) PendingMicrotasks() []string {
	if v.session == nil {
		return nil
	}
	microtasks := v.session.PendingMicrotasks()
	out := make([]string, len(microtasks))
	copy(out, microtasks)
	return out
}

func (v DebugView) Interactions() []Interaction {
	if v.session == nil {
		return nil
	}
	records := v.session.InteractionLog()
	out := make([]Interaction, len(records))
	for i := range records {
		out[i] = Interaction{
			Kind:     InteractionKind(records[i].Kind),
			Selector: records[i].Selector,
		}
	}
	return out
}

func (v DebugView) EventListeners() []EventListenerRegistration {
	if v.session == nil {
		return nil
	}
	records := v.session.EventListeners()
	out := make([]EventListenerRegistration, len(records))
	for i := range records {
		out[i] = EventListenerRegistration{
			NodeID: records[i].NodeID,
			Event:  records[i].Event,
			Phase:  records[i].Phase,
			Source: records[i].Source,
			Once:   records[i].Once,
		}
	}
	return out
}

func (v DebugView) RandomSeed() (int64, bool) {
	if v.session == nil {
		return 0, false
	}
	config := v.session.Config()
	if !config.HasRandomSeed {
		return 0, false
	}
	return config.RandomSeed, true
}
