package mocks

import (
	"fmt"
	"sort"
	"strings"
)

type Registry struct {
	fetch      FetchFamily
	dialogs    DialogFamily
	clipboard  ClipboardFamily
	location   LocationFamily
	open       OpenFamily
	close      CloseFamily
	print      PrintFamily
	scroll     ScrollFamily
	matchMedia MatchMediaFamily
	downloads  DownloadFamily
	fileInput  FileInputFamily
	storage    StorageFamily
}

func NewRegistry() *Registry {
	return &Registry{}
}

func (r *Registry) Fetch() *FetchFamily {
	if r == nil {
		return nil
	}
	return &r.fetch
}

func (r *Registry) Dialogs() *DialogFamily {
	if r == nil {
		return nil
	}
	return &r.dialogs
}

func (r *Registry) Clipboard() *ClipboardFamily {
	if r == nil {
		return nil
	}
	return &r.clipboard
}

func (r *Registry) Location() *LocationFamily {
	if r == nil {
		return nil
	}
	return &r.location
}

func (r *Registry) Open() *OpenFamily {
	if r == nil {
		return nil
	}
	return &r.open
}

func (r *Registry) Close() *CloseFamily {
	if r == nil {
		return nil
	}
	return &r.close
}

func (r *Registry) Print() *PrintFamily {
	if r == nil {
		return nil
	}
	return &r.print
}

func (r *Registry) Scroll() *ScrollFamily {
	if r == nil {
		return nil
	}
	return &r.scroll
}

func (r *Registry) MatchMedia() *MatchMediaFamily {
	if r == nil {
		return nil
	}
	return &r.matchMedia
}

func (r *Registry) Downloads() *DownloadFamily {
	if r == nil {
		return nil
	}
	return &r.downloads
}

func (r *Registry) FileInput() *FileInputFamily {
	if r == nil {
		return nil
	}
	return &r.fileInput
}

func (r *Registry) Storage() *StorageFamily {
	if r == nil {
		return nil
	}
	return &r.storage
}

func (r *Registry) ResetAll() {
	if r == nil {
		return
	}
	r.fetch.Reset()
	r.dialogs.Reset()
	r.clipboard.Reset()
	r.location.Reset()
	r.open.Reset()
	r.close.Reset()
	r.print.Reset()
	r.scroll.Reset()
	r.matchMedia.Reset()
	r.downloads.Reset()
	r.fileInput.Reset()
	r.storage.Reset()
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

type FetchFamily struct {
	responseRules []FetchResponseRule
	errorRules    []FetchErrorRule
	calls         []FetchCall
}

func (f *FetchFamily) RespondText(url string, status int, body string) {
	if f == nil {
		return
	}
	url = strings.TrimSpace(url)
	f.responseRules = append(f.responseRules, FetchResponseRule{
		URL:    url,
		Status: status,
		Body:   body,
	})
}

func (f *FetchFamily) Fail(url string, message string) {
	if f == nil {
		return
	}
	url = strings.TrimSpace(url)
	f.errorRules = append(f.errorRules, FetchErrorRule{
		URL:     url,
		Message: message,
	})
}

func (f *FetchFamily) RecordCall(url string) {
	if f == nil {
		return
	}
	f.calls = append(f.calls, FetchCall{URL: strings.TrimSpace(url)})
}

func (f *FetchFamily) Resolve(url string) (int, string, error) {
	if f == nil {
		return 0, "", fmt.Errorf("fetch mock registry is unavailable")
	}

	url = strings.TrimSpace(url)
	if url == "" {
		return 0, "", fmt.Errorf("fetch() requires a non-empty URL")
	}

	f.RecordCall(url)

	for i := len(f.errorRules) - 1; i >= 0; i-- {
		rule := f.errorRules[i]
		if rule.URL == url {
			return 0, "", fmt.Errorf("%s", rule.Message)
		}
	}

	for i := len(f.responseRules) - 1; i >= 0; i-- {
		rule := f.responseRules[i]
		if rule.URL == url {
			return rule.Status, rule.Body, nil
		}
	}

	return 0, "", fmt.Errorf("no fetch mock configured for `%s`", url)
}

func (f *FetchFamily) Calls() []FetchCall {
	if f == nil {
		return nil
	}
	out := make([]FetchCall, len(f.calls))
	copy(out, f.calls)
	return out
}

func (f *FetchFamily) TakeCalls() []FetchCall {
	if f == nil {
		return nil
	}
	out := make([]FetchCall, len(f.calls))
	copy(out, f.calls)
	f.calls = nil
	return out
}

func (f *FetchFamily) ResponseRules() []FetchResponseRule {
	if f == nil {
		return nil
	}
	out := make([]FetchResponseRule, len(f.responseRules))
	copy(out, f.responseRules)
	return out
}

func (f *FetchFamily) ErrorRules() []FetchErrorRule {
	if f == nil {
		return nil
	}
	out := make([]FetchErrorRule, len(f.errorRules))
	copy(out, f.errorRules)
	return out
}

func (f *FetchFamily) Reset() {
	if f == nil {
		return
	}
	f.responseRules = nil
	f.errorRules = nil
	f.calls = nil
}

type DialogFamily struct {
	confirmQueue  []bool
	promptQueue   []*string
	alertMessages []string
	confirmMsgs   []string
	promptMsgs    []string
}

func (f *DialogFamily) QueueConfirm(value bool) {
	if f == nil {
		return
	}
	f.confirmQueue = append(f.confirmQueue, value)
}

func (f *DialogFamily) QueuePrompt(value *string) {
	if f == nil {
		return
	}
	if value == nil {
		f.promptQueue = append(f.promptQueue, nil)
		return
	}
	seeded := *value
	f.promptQueue = append(f.promptQueue, &seeded)
}

func (f *DialogFamily) QueuePromptText(value string) {
	if f == nil {
		return
	}
	seeded := value
	f.promptQueue = append(f.promptQueue, &seeded)
}

func (f *DialogFamily) QueuePromptCancel() {
	if f == nil {
		return
	}
	f.promptQueue = append(f.promptQueue, nil)
}

func (f *DialogFamily) RecordAlert(message string) {
	if f == nil {
		return
	}
	f.alertMessages = append(f.alertMessages, message)
}

func (f *DialogFamily) RecordConfirm(message string) {
	if f == nil {
		return
	}
	f.confirmMsgs = append(f.confirmMsgs, message)
}

func (f *DialogFamily) RecordPrompt(message string) {
	if f == nil {
		return
	}
	f.promptMsgs = append(f.promptMsgs, message)
}

func (f *DialogFamily) TakeConfirm() (bool, bool) {
	if f == nil || len(f.confirmQueue) == 0 {
		return false, false
	}
	value := f.confirmQueue[0]
	f.confirmQueue = f.confirmQueue[1:]
	return value, true
}

func (f *DialogFamily) TakePrompt() (string, bool, bool) {
	if f == nil || len(f.promptQueue) == 0 {
		return "", false, false
	}
	value := f.promptQueue[0]
	f.promptQueue = f.promptQueue[1:]
	if value == nil {
		return "", false, true
	}
	return *value, true, true
}

func (f *DialogFamily) Alerts() []string {
	if f == nil {
		return nil
	}
	out := make([]string, len(f.alertMessages))
	copy(out, f.alertMessages)
	return out
}

func (f *DialogFamily) TakeAlerts() []string {
	if f == nil {
		return nil
	}
	out := make([]string, len(f.alertMessages))
	copy(out, f.alertMessages)
	f.alertMessages = nil
	return out
}

func (f *DialogFamily) ConfirmMessages() []string {
	if f == nil {
		return nil
	}
	out := make([]string, len(f.confirmMsgs))
	copy(out, f.confirmMsgs)
	return out
}

func (f *DialogFamily) TakeConfirmMessages() []string {
	if f == nil {
		return nil
	}
	out := make([]string, len(f.confirmMsgs))
	copy(out, f.confirmMsgs)
	f.confirmMsgs = nil
	return out
}

func (f *DialogFamily) PromptMessages() []string {
	if f == nil {
		return nil
	}
	out := make([]string, len(f.promptMsgs))
	copy(out, f.promptMsgs)
	return out
}

func (f *DialogFamily) TakePromptMessages() []string {
	if f == nil {
		return nil
	}
	out := make([]string, len(f.promptMsgs))
	copy(out, f.promptMsgs)
	f.promptMsgs = nil
	return out
}

func (f *DialogFamily) Reset() {
	if f == nil {
		return
	}
	f.confirmQueue = nil
	f.promptQueue = nil
	f.alertMessages = nil
	f.confirmMsgs = nil
	f.promptMsgs = nil
}

type ClipboardFamily struct {
	seededText *string
	writes     []string
}

func (f *ClipboardFamily) SeedText(value string) {
	if f == nil {
		return
	}
	seeded := value
	f.seededText = &seeded
}

func (f *ClipboardFamily) SeededText() (string, bool) {
	if f == nil || f.seededText == nil {
		return "", false
	}
	return *f.seededText, true
}

func (f *ClipboardFamily) Writes() []string {
	if f == nil {
		return nil
	}
	out := make([]string, len(f.writes))
	copy(out, f.writes)
	return out
}

func (f *ClipboardFamily) RecordWrite(value string) {
	if f == nil {
		return
	}
	f.writes = append(f.writes, value)
	f.SeedText(value)
}

func (f *ClipboardFamily) Reset() {
	if f == nil {
		return
	}
	f.seededText = nil
	f.writes = nil
}

type LocationFamily struct {
	currentURL  *string
	navigations []string
}

func (f *LocationFamily) SetCurrentURL(url string) {
	if f == nil {
		return
	}
	current := url
	f.currentURL = &current
}

func (f *LocationFamily) CurrentURL() (string, bool) {
	if f == nil || f.currentURL == nil {
		return "", false
	}
	return *f.currentURL, true
}

func (f *LocationFamily) RecordNavigation(url string) {
	if f == nil {
		return
	}
	f.navigations = append(f.navigations, url)
	f.SetCurrentURL(url)
}

func (f *LocationFamily) Navigations() []string {
	if f == nil {
		return nil
	}
	out := make([]string, len(f.navigations))
	copy(out, f.navigations)
	return out
}

func (f *LocationFamily) TakeNavigations() []string {
	if f == nil {
		return nil
	}
	out := make([]string, len(f.navigations))
	copy(out, f.navigations)
	f.navigations = nil
	return out
}

func (f *LocationFamily) Reset() {
	if f == nil {
		return
	}
	f.currentURL = nil
	f.navigations = nil
}

type OpenCall struct {
	URL string
}

type OpenFamily struct {
	failure *string
	calls   []OpenCall
}

func (f *OpenFamily) Fail(message string) {
	if f == nil {
		return
	}
	failure := message
	f.failure = &failure
}

func (f *OpenFamily) ClearFailure() {
	if f == nil {
		return
	}
	f.failure = nil
}

func (f *OpenFamily) Invoke(url string) error {
	if f == nil {
		return fmt.Errorf("open mock registry is unavailable")
	}
	f.calls = append(f.calls, OpenCall{URL: url})
	if f.failure != nil {
		return fmt.Errorf("%s", *f.failure)
	}
	return nil
}

func (f *OpenFamily) Calls() []OpenCall {
	if f == nil {
		return nil
	}
	out := make([]OpenCall, len(f.calls))
	copy(out, f.calls)
	return out
}

func (f *OpenFamily) TakeCalls() []OpenCall {
	if f == nil {
		return nil
	}
	out := make([]OpenCall, len(f.calls))
	copy(out, f.calls)
	f.calls = nil
	return out
}

func (f *OpenFamily) Reset() {
	if f == nil {
		return
	}
	f.failure = nil
	f.calls = nil
}

type CloseCall struct{}

type CloseFamily struct {
	failure *string
	calls   []CloseCall
}

func (f *CloseFamily) Fail(message string) {
	if f == nil {
		return
	}
	failure := message
	f.failure = &failure
}

func (f *CloseFamily) ClearFailure() {
	if f == nil {
		return
	}
	f.failure = nil
}

func (f *CloseFamily) Invoke() error {
	if f == nil {
		return fmt.Errorf("close mock registry is unavailable")
	}
	f.calls = append(f.calls, CloseCall{})
	if f.failure != nil {
		return fmt.Errorf("%s", *f.failure)
	}
	return nil
}

func (f *CloseFamily) Calls() []CloseCall {
	if f == nil {
		return nil
	}
	out := make([]CloseCall, len(f.calls))
	copy(out, f.calls)
	return out
}

func (f *CloseFamily) TakeCalls() []CloseCall {
	if f == nil {
		return nil
	}
	out := make([]CloseCall, len(f.calls))
	copy(out, f.calls)
	f.calls = nil
	return out
}

func (f *CloseFamily) Reset() {
	if f == nil {
		return
	}
	f.failure = nil
	f.calls = nil
}

type PrintCall struct{}

type PrintFamily struct {
	failure *string
	calls   []PrintCall
}

func (f *PrintFamily) Fail(message string) {
	if f == nil {
		return
	}
	failure := message
	f.failure = &failure
}

func (f *PrintFamily) ClearFailure() {
	if f == nil {
		return
	}
	f.failure = nil
}

func (f *PrintFamily) Invoke() error {
	if f == nil {
		return fmt.Errorf("print mock registry is unavailable")
	}
	f.calls = append(f.calls, PrintCall{})
	if f.failure != nil {
		return fmt.Errorf("%s", *f.failure)
	}
	return nil
}

func (f *PrintFamily) Calls() []PrintCall {
	if f == nil {
		return nil
	}
	out := make([]PrintCall, len(f.calls))
	copy(out, f.calls)
	return out
}

func (f *PrintFamily) Take() []PrintCall {
	if f == nil {
		return nil
	}
	out := make([]PrintCall, len(f.calls))
	copy(out, f.calls)
	f.calls = nil
	return out
}

func (f *PrintFamily) Reset() {
	if f == nil {
		return
	}
	f.failure = nil
	f.calls = nil
}

type ScrollCall struct {
	Method string
	X      int64
	Y      int64
}

type ScrollFamily struct {
	failure *string
	calls   []ScrollCall
}

func (f *ScrollFamily) Fail(message string) {
	if f == nil {
		return
	}
	failure := message
	f.failure = &failure
}

func (f *ScrollFamily) ClearFailure() {
	if f == nil {
		return
	}
	f.failure = nil
}

func (f *ScrollFamily) Invoke(method string, x, y int64) error {
	if f == nil {
		return fmt.Errorf("scroll mock registry is unavailable")
	}
	f.calls = append(f.calls, ScrollCall{Method: method, X: x, Y: y})
	if f.failure != nil {
		return fmt.Errorf("%s", *f.failure)
	}
	return nil
}

func (f *ScrollFamily) Calls() []ScrollCall {
	if f == nil {
		return nil
	}
	out := make([]ScrollCall, len(f.calls))
	copy(out, f.calls)
	return out
}

func (f *ScrollFamily) TakeCalls() []ScrollCall {
	if f == nil {
		return nil
	}
	out := make([]ScrollCall, len(f.calls))
	copy(out, f.calls)
	f.calls = nil
	return out
}

func (f *ScrollFamily) Reset() {
	if f == nil {
		return
	}
	f.failure = nil
	f.calls = nil
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

type MatchMediaFamily struct {
	rules         []MatchMediaRule
	calls         []MatchMediaCall
	listenerCalls []MatchMediaListenerCall
}

func (f *MatchMediaFamily) RespondMatches(query string, matches bool) {
	if f == nil {
		return
	}
	query = strings.TrimSpace(query)
	f.rules = append(f.rules, MatchMediaRule{
		Query:   query,
		Matches: matches,
	})
}

func (f *MatchMediaFamily) RecordCall(query string) {
	if f == nil {
		return
	}
	f.calls = append(f.calls, MatchMediaCall{Query: strings.TrimSpace(query)})
}

func (f *MatchMediaFamily) RecordListenerCall(query, method string) {
	if f == nil {
		return
	}
	f.listenerCalls = append(f.listenerCalls, MatchMediaListenerCall{
		Query:  strings.TrimSpace(query),
		Method: method,
	})
}

func (f *MatchMediaFamily) Rules() []MatchMediaRule {
	if f == nil {
		return nil
	}
	out := make([]MatchMediaRule, len(f.rules))
	copy(out, f.rules)
	return out
}

func (f *MatchMediaFamily) Resolve(query string) (bool, error) {
	if f == nil {
		return false, fmt.Errorf("matchMedia mock registry is unavailable")
	}

	query = strings.TrimSpace(query)
	if query == "" {
		return false, fmt.Errorf("matchMedia() requires a non-empty media query")
	}

	f.RecordCall(query)

	for i := len(f.rules) - 1; i >= 0; i-- {
		rule := f.rules[i]
		if rule.Query == query {
			return rule.Matches, nil
		}
	}

	return false, fmt.Errorf("no matchMedia mock configured for `%s`", query)
}

func (f *MatchMediaFamily) Calls() []MatchMediaCall {
	if f == nil {
		return nil
	}
	out := make([]MatchMediaCall, len(f.calls))
	copy(out, f.calls)
	return out
}

func (f *MatchMediaFamily) TakeCalls() []MatchMediaCall {
	if f == nil {
		return nil
	}
	out := make([]MatchMediaCall, len(f.calls))
	copy(out, f.calls)
	f.calls = nil
	return out
}

func (f *MatchMediaFamily) ListenerCalls() []MatchMediaListenerCall {
	if f == nil {
		return nil
	}
	out := make([]MatchMediaListenerCall, len(f.listenerCalls))
	copy(out, f.listenerCalls)
	return out
}

func (f *MatchMediaFamily) TakeListenerCalls() []MatchMediaListenerCall {
	if f == nil {
		return nil
	}
	out := make([]MatchMediaListenerCall, len(f.listenerCalls))
	copy(out, f.listenerCalls)
	f.listenerCalls = nil
	return out
}

func (f *MatchMediaFamily) Reset() {
	if f == nil {
		return
	}
	f.rules = nil
	f.calls = nil
	f.listenerCalls = nil
}

type DownloadCapture struct {
	FileName string
	Bytes    []byte
}

type DownloadFamily struct {
	artifacts []DownloadCapture
}

func (f *DownloadFamily) Capture(fileName string, bytes []byte) {
	if f == nil {
		return
	}
	copied := make([]byte, len(bytes))
	copy(copied, bytes)
	f.artifacts = append(f.artifacts, DownloadCapture{
		FileName: fileName,
		Bytes:    copied,
	})
}

func (f *DownloadFamily) Artifacts() []DownloadCapture {
	if f == nil {
		return nil
	}
	out := make([]DownloadCapture, len(f.artifacts))
	for i := range f.artifacts {
		item := f.artifacts[i]
		bytes := make([]byte, len(item.Bytes))
		copy(bytes, item.Bytes)
		out[i] = DownloadCapture{FileName: item.FileName, Bytes: bytes}
	}
	return out
}

func (f *DownloadFamily) Take() []DownloadCapture {
	if f == nil {
		return nil
	}
	out := f.Artifacts()
	f.artifacts = nil
	return out
}

func (f *DownloadFamily) Reset() {
	if f == nil {
		return
	}
	f.artifacts = nil
}

type FileInputSelection struct {
	Selector string
	Files    []string
}

type FileInputFamily struct {
	selections []FileInputSelection
}

func (f *FileInputFamily) SetFiles(selector string, files []string) {
	if f == nil {
		return
	}
	copied := make([]string, len(files))
	copy(copied, files)
	f.selections = append(f.selections, FileInputSelection{
		Selector: selector,
		Files:    copied,
	})
}

func (f *FileInputFamily) Selections() []FileInputSelection {
	if f == nil {
		return nil
	}
	out := make([]FileInputSelection, len(f.selections))
	for i := range f.selections {
		item := f.selections[i]
		files := make([]string, len(item.Files))
		copy(files, item.Files)
		out[i] = FileInputSelection{
			Selector: item.Selector,
			Files:    files,
		}
	}
	return out
}

func (f *FileInputFamily) TakeSelections() []FileInputSelection {
	if f == nil {
		return nil
	}
	out := f.Selections()
	f.selections = nil
	return out
}

func (f *FileInputFamily) Reset() {
	if f == nil {
		return
	}
	f.selections = nil
}

type StorageFamily struct {
	local   map[string]string
	session map[string]string
	events  []StorageEvent
}

type StorageEvent struct {
	Scope string
	Op    string
	Key   string
	Value string
}

func (f *StorageFamily) storageMap(scope string, create bool) (map[string]string, bool) {
	switch strings.ToLower(strings.TrimSpace(scope)) {
	case "local":
		if create {
			f.ensureLocal()
		}
		return f.local, true
	case "session":
		if create {
			f.ensureSession()
		}
		return f.session, true
	default:
		return nil, false
	}
}

func (f *StorageFamily) storageKeys(scope string) ([]string, bool) {
	m, ok := f.storageMap(scope, false)
	if !ok || len(m) == 0 {
		return nil, ok
	}
	keys := make([]string, 0, len(m))
	for key := range m {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	return keys, true
}

func (f *StorageFamily) ensureLocal() {
	if f.local == nil {
		f.local = map[string]string{}
	}
}

func (f *StorageFamily) ensureSession() {
	if f.session == nil {
		f.session = map[string]string{}
	}
}

func (f *StorageFamily) SeedLocal(key, value string) {
	if f == nil {
		return
	}
	f.ensureLocal()
	f.local[key] = value
	f.events = append(f.events, StorageEvent{Scope: "local", Op: "seed", Key: key, Value: value})
}

func (f *StorageFamily) SeedSession(key, value string) {
	if f == nil {
		return
	}
	f.ensureSession()
	f.session[key] = value
	f.events = append(f.events, StorageEvent{Scope: "session", Op: "seed", Key: key, Value: value})
}

func (f *StorageFamily) Get(scope, key string) (string, bool) {
	if f == nil {
		return "", false
	}
	m, ok := f.storageMap(scope, false)
	if !ok {
		return "", false
	}
	value, exists := m[key]
	return value, exists
}

func (f *StorageFamily) Set(scope, key, value string) bool {
	if f == nil {
		return false
	}
	normalized := strings.ToLower(strings.TrimSpace(scope))
	m, ok := f.storageMap(normalized, true)
	if !ok {
		return false
	}
	if current, exists := m[key]; exists && current == value {
		return true
	}
	m[key] = value
	f.events = append(f.events, StorageEvent{Scope: normalized, Op: "set", Key: key, Value: value})
	return true
}

func (f *StorageFamily) Remove(scope, key string) bool {
	if f == nil {
		return false
	}
	normalized := strings.ToLower(strings.TrimSpace(scope))
	m, ok := f.storageMap(normalized, false)
	if !ok {
		return false
	}
	if _, exists := m[key]; !exists {
		return true
	}
	delete(m, key)
	f.events = append(f.events, StorageEvent{Scope: normalized, Op: "remove", Key: key})
	return true
}

func (f *StorageFamily) Clear(scope string) bool {
	if f == nil {
		return false
	}
	normalized := strings.ToLower(strings.TrimSpace(scope))
	m, ok := f.storageMap(normalized, false)
	if !ok {
		return false
	}
	if len(m) == 0 {
		return true
	}
	for key := range m {
		delete(m, key)
	}
	f.events = append(f.events, StorageEvent{Scope: normalized, Op: "clear"})
	return true
}

func (f *StorageFamily) Local() map[string]string {
	if f == nil || len(f.local) == 0 {
		return map[string]string{}
	}
	out := make(map[string]string, len(f.local))
	for key, value := range f.local {
		out[key] = value
	}
	return out
}

func (f *StorageFamily) Session() map[string]string {
	if f == nil || len(f.session) == 0 {
		return map[string]string{}
	}
	out := make(map[string]string, len(f.session))
	for key, value := range f.session {
		out[key] = value
	}
	return out
}

func (f *StorageFamily) Length(scope string) (int, bool) {
	if f == nil {
		return 0, false
	}
	m, ok := f.storageMap(scope, false)
	if !ok {
		return 0, false
	}
	return len(m), true
}

func (f *StorageFamily) Key(scope string, index int) (string, bool) {
	if f == nil || index < 0 {
		return "", false
	}
	keys, ok := f.storageKeys(scope)
	if !ok || index >= len(keys) {
		return "", false
	}
	return keys[index], true
}

func (f *StorageFamily) Events() []StorageEvent {
	if f == nil {
		return nil
	}
	out := make([]StorageEvent, len(f.events))
	copy(out, f.events)
	return out
}

func cloneStorageMap(input map[string]string) map[string]string {
	if len(input) == 0 {
		return nil
	}
	out := make(map[string]string, len(input))
	for key, value := range input {
		out[key] = value
	}
	return out
}

func cloneStorageEvents(input []StorageEvent) []StorageEvent {
	if len(input) == 0 {
		return nil
	}
	out := make([]StorageEvent, len(input))
	copy(out, input)
	return out
}

func (f *StorageFamily) Restore(local, session map[string]string, events []StorageEvent) {
	if f == nil {
		return
	}
	f.local = cloneStorageMap(local)
	f.session = cloneStorageMap(session)
	f.events = cloneStorageEvents(events)
}

func (f *StorageFamily) Reset() {
	if f == nil {
		return
	}
	f.local = nil
	f.session = nil
	f.events = nil
}
