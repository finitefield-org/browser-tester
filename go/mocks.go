package browsertester

import imocks "browsertester/internal/mocks"

type FetchResponse struct {
	URL    string
	Status int
	Body   string
}

type FetchCall struct {
	URL string
}

type FetchResponseRule struct {
	URL    string
	Status int
	Body   string
}

type FetchErrorRule struct {
	URL     string
	Message string
}

type OpenCall struct {
	URL string
}

type CloseCall struct{}

type PrintCall struct{}

type ScrollMethod string

const (
	ScrollMethodTo ScrollMethod = "to"
	ScrollMethodBy ScrollMethod = "by"
)

type ScrollCall struct {
	Method ScrollMethod
	X      int64
	Y      int64
}

type HistoryEntry struct {
	URL      string
	State    string
	HasState bool
}

type TimerSnapshot struct {
	ID         int64
	Source     string
	DueAtMs    int64
	Repeat     bool
	IntervalMs int64
}

type AnimationFrameSnapshot struct {
	ID     int64
	Source string
}

type MatchMediaCall struct {
	Query string
}

type MatchMediaListenerCall struct {
	Query  string
	Method string
}

type MatchMediaRule struct {
	Query   string
	Matches bool
}

type DownloadCapture struct {
	FileName string
	Bytes    []byte
}

type FileInputSelection struct {
	Selector string
	Files    []string
}

type StorageEvent struct {
	Scope string
	Op    string
	Key   string
	Value string
}

type MockRegistryView struct {
	registry *imocks.Registry
}

func (v MockRegistryView) Fetch() *FetchMocks {
	if v.registry == nil {
		return nil
	}
	return &FetchMocks{family: v.registry.Fetch()}
}

func (v MockRegistryView) Dialogs() *DialogMocks {
	if v.registry == nil {
		return nil
	}
	return &DialogMocks{family: v.registry.Dialogs()}
}

func (v MockRegistryView) Clipboard() *ClipboardMocks {
	if v.registry == nil {
		return nil
	}
	return &ClipboardMocks{family: v.registry.Clipboard()}
}

func (v MockRegistryView) Location() *LocationMocks {
	if v.registry == nil {
		return nil
	}
	return &LocationMocks{family: v.registry.Location()}
}

func (v MockRegistryView) Open() *OpenMocks {
	if v.registry == nil {
		return nil
	}
	return &OpenMocks{family: v.registry.Open()}
}

func (v MockRegistryView) Close() *CloseMocks {
	if v.registry == nil {
		return nil
	}
	return &CloseMocks{family: v.registry.Close()}
}

func (v MockRegistryView) Print() *PrintMocks {
	if v.registry == nil {
		return nil
	}
	return &PrintMocks{family: v.registry.Print()}
}

func (v MockRegistryView) Scroll() *ScrollMocks {
	if v.registry == nil {
		return nil
	}
	return &ScrollMocks{family: v.registry.Scroll()}
}

func (v MockRegistryView) MatchMedia() *MatchMediaMocks {
	if v.registry == nil {
		return nil
	}
	return &MatchMediaMocks{family: v.registry.MatchMedia()}
}

func (v MockRegistryView) Downloads() *DownloadMocks {
	if v.registry == nil {
		return nil
	}
	return &DownloadMocks{family: v.registry.Downloads()}
}

func (v MockRegistryView) FileInput() *FileInputMocks {
	if v.registry == nil {
		return nil
	}
	return &FileInputMocks{family: v.registry.FileInput()}
}

func (v MockRegistryView) Storage() *StorageSeeds {
	if v.registry == nil {
		return nil
	}
	return &StorageSeeds{family: v.registry.Storage()}
}

func (v MockRegistryView) ResetAll() {
	if v.registry == nil {
		return
	}
	v.registry.ResetAll()
}

type FetchMocks struct {
	family *imocks.FetchFamily
}

func (m *FetchMocks) RespondText(url string, status int, body string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.RespondText(url, status, body)
}

func (m *FetchMocks) Fail(url string, message string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.Fail(url, message)
}

func (m *FetchMocks) Calls() []FetchCall {
	if m == nil || m.family == nil {
		return nil
	}
	calls := m.family.Calls()
	out := make([]FetchCall, len(calls))
	for i := range calls {
		out[i] = FetchCall{URL: calls[i].URL}
	}
	return out
}

func (m *FetchMocks) Responses() []FetchResponseRule {
	if m == nil || m.family == nil {
		return nil
	}
	rules := m.family.ResponseRules()
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

func (m *FetchMocks) Errors() []FetchErrorRule {
	if m == nil || m.family == nil {
		return nil
	}
	rules := m.family.ErrorRules()
	out := make([]FetchErrorRule, len(rules))
	for i := range rules {
		out[i] = FetchErrorRule{
			URL:     rules[i].URL,
			Message: rules[i].Message,
		}
	}
	return out
}

func (m *FetchMocks) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}

type DialogMocks struct {
	family *imocks.DialogFamily
}

func (m *DialogMocks) QueueConfirm(value bool) {
	if m == nil || m.family == nil {
		return
	}
	m.family.QueueConfirm(value)
}

func (m *DialogMocks) QueuePrompt(value *string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.QueuePrompt(value)
}

func (m *DialogMocks) QueuePromptText(value string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.QueuePromptText(value)
}

func (m *DialogMocks) QueuePromptCancel() {
	if m == nil || m.family == nil {
		return
	}
	m.family.QueuePromptCancel()
}

func (m *DialogMocks) Alerts() []string {
	if m == nil || m.family == nil {
		return nil
	}
	return m.family.Alerts()
}

func (m *DialogMocks) ConfirmMessages() []string {
	if m == nil || m.family == nil {
		return nil
	}
	return m.family.ConfirmMessages()
}

func (m *DialogMocks) PromptMessages() []string {
	if m == nil || m.family == nil {
		return nil
	}
	return m.family.PromptMessages()
}

func (m *DialogMocks) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}

type ClipboardMocks struct {
	family *imocks.ClipboardFamily
}

func (m *ClipboardMocks) SeedText(value string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.SeedText(value)
}

func (m *ClipboardMocks) SeededText() (string, bool) {
	if m == nil || m.family == nil {
		return "", false
	}
	return m.family.SeededText()
}

func (m *ClipboardMocks) Writes() []string {
	if m == nil || m.family == nil {
		return nil
	}
	return m.family.Writes()
}

func (m *ClipboardMocks) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}

type LocationMocks struct {
	family *imocks.LocationFamily
}

func (m *LocationMocks) CurrentURL() string {
	if m == nil || m.family == nil {
		return ""
	}
	if current, ok := m.family.CurrentURL(); ok {
		return current
	}
	return ""
}

func (m *LocationMocks) Navigations() []string {
	if m == nil || m.family == nil {
		return nil
	}
	return m.family.Navigations()
}

func (m *LocationMocks) SetCurrentURL(url string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.SetCurrentURL(url)
}

func (m *LocationMocks) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}

type OpenMocks struct {
	family *imocks.OpenFamily
}

func (m *OpenMocks) Fail(message string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.Fail(message)
}

func (m *OpenMocks) ClearFailure() {
	if m == nil || m.family == nil {
		return
	}
	m.family.ClearFailure()
}

func (m *OpenMocks) Calls() []OpenCall {
	if m == nil || m.family == nil {
		return nil
	}
	calls := m.family.Calls()
	out := make([]OpenCall, len(calls))
	for i := range calls {
		out[i] = OpenCall{URL: calls[i].URL}
	}
	return out
}

func (m *OpenMocks) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}

type CloseMocks struct {
	family *imocks.CloseFamily
}

func (m *CloseMocks) Fail(message string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.Fail(message)
}

func (m *CloseMocks) ClearFailure() {
	if m == nil || m.family == nil {
		return
	}
	m.family.ClearFailure()
}

func (m *CloseMocks) Calls() []CloseCall {
	if m == nil || m.family == nil {
		return nil
	}
	calls := m.family.Calls()
	out := make([]CloseCall, len(calls))
	for i := range calls {
		out[i] = CloseCall{}
	}
	return out
}

func (m *CloseMocks) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}

type PrintMocks struct {
	family *imocks.PrintFamily
}

func (m *PrintMocks) Fail(message string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.Fail(message)
}

func (m *PrintMocks) ClearFailure() {
	if m == nil || m.family == nil {
		return
	}
	m.family.ClearFailure()
}

func (m *PrintMocks) Calls() []PrintCall {
	if m == nil || m.family == nil {
		return nil
	}
	calls := m.family.Calls()
	out := make([]PrintCall, len(calls))
	for i := range calls {
		out[i] = PrintCall{}
	}
	return out
}

func (m *PrintMocks) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}

type ScrollMocks struct {
	family *imocks.ScrollFamily
}

func (m *ScrollMocks) Fail(message string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.Fail(message)
}

func (m *ScrollMocks) ClearFailure() {
	if m == nil || m.family == nil {
		return
	}
	m.family.ClearFailure()
}

func (m *ScrollMocks) Calls() []ScrollCall {
	if m == nil || m.family == nil {
		return nil
	}
	calls := m.family.Calls()
	out := make([]ScrollCall, len(calls))
	for i := range calls {
		method := ScrollMethod(calls[i].Method)
		out[i] = ScrollCall{
			Method: method,
			X:      calls[i].X,
			Y:      calls[i].Y,
		}
	}
	return out
}

func (m *ScrollMocks) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}

type MatchMediaMocks struct {
	family *imocks.MatchMediaFamily
}

func (m *MatchMediaMocks) RespondMatches(query string, matches bool) {
	if m == nil || m.family == nil {
		return
	}
	m.family.RespondMatches(query, matches)
}

func (m *MatchMediaMocks) RecordListenerCall(query, method string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.RecordListenerCall(query, method)
}

func (m *MatchMediaMocks) Rules() []MatchMediaRule {
	if m == nil || m.family == nil {
		return nil
	}
	rules := m.family.Rules()
	out := make([]MatchMediaRule, len(rules))
	for i := range rules {
		out[i] = MatchMediaRule{
			Query:   rules[i].Query,
			Matches: rules[i].Matches,
		}
	}
	return out
}

func (m *MatchMediaMocks) Calls() []MatchMediaCall {
	if m == nil || m.family == nil {
		return nil
	}
	calls := m.family.Calls()
	out := make([]MatchMediaCall, len(calls))
	for i := range calls {
		out[i] = MatchMediaCall{Query: calls[i].Query}
	}
	return out
}

func (m *MatchMediaMocks) ListenerCalls() []MatchMediaListenerCall {
	if m == nil || m.family == nil {
		return nil
	}
	calls := m.family.ListenerCalls()
	out := make([]MatchMediaListenerCall, len(calls))
	for i := range calls {
		out[i] = MatchMediaListenerCall{
			Query:  calls[i].Query,
			Method: calls[i].Method,
		}
	}
	return out
}

func (m *MatchMediaMocks) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}

type DownloadMocks struct {
	family *imocks.DownloadFamily
}

func (m *DownloadMocks) Capture(fileName string, bytes []byte) {
	if m == nil || m.family == nil {
		return
	}
	m.family.Capture(fileName, bytes)
}

func (m *DownloadMocks) Artifacts() []DownloadCapture {
	if m == nil || m.family == nil {
		return nil
	}
	artifacts := m.family.Artifacts()
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

func (m *DownloadMocks) Take() []DownloadCapture {
	if m == nil || m.family == nil {
		return nil
	}
	artifacts := m.family.Take()
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

func (m *DownloadMocks) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}

type FileInputMocks struct {
	family *imocks.FileInputFamily
}

func (m *FileInputMocks) SetFiles(selector string, files []string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.SetFiles(selector, files)
}

func (m *FileInputMocks) Selections() []FileInputSelection {
	if m == nil || m.family == nil {
		return nil
	}
	selections := m.family.Selections()
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

func (m *FileInputMocks) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}

type StorageSeeds struct {
	family *imocks.StorageFamily
}

func (m *StorageSeeds) SeedLocal(key, value string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.SeedLocal(key, value)
}

func (m *StorageSeeds) SeedSession(key, value string) {
	if m == nil || m.family == nil {
		return
	}
	m.family.SeedSession(key, value)
}

func (m *StorageSeeds) Local() map[string]string {
	if m == nil || m.family == nil {
		return map[string]string{}
	}
	return m.family.Local()
}

func (m *StorageSeeds) Session() map[string]string {
	if m == nil || m.family == nil {
		return map[string]string{}
	}
	return m.family.Session()
}

func (m *StorageSeeds) Events() []StorageEvent {
	if m == nil || m.family == nil {
		return nil
	}
	events := m.family.Events()
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

func (m *StorageSeeds) Reset() {
	if m == nil || m.family == nil {
		return
	}
	m.family.Reset()
}
