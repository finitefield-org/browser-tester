package runtime

import (
	"fmt"
	"strings"

	"browsertester/internal/dom"
	"browsertester/internal/mocks"
)

type SessionConfig struct {
	URL            string
	HTML           string
	LocalStorage   map[string]string
	SessionStorage map[string]string
	RandomSeed     int64
	HasRandomSeed  bool
	MatchMedia     map[string]bool
	OpenFailure    string
	CloseFailure   string
	PrintFailure   string
	ScrollFailure  string
}

func DefaultSessionConfig() SessionConfig {
	return SessionConfig{
		URL:            "https://app.local/",
		LocalStorage:   map[string]string{},
		SessionStorage: map[string]string{},
		MatchMedia:     map[string]bool{},
	}
}

type Session struct {
	config                   SessionConfig
	scheduler                Scheduler
	scrollX                  int64
	scrollY                  int64
	registry                 *mocks.Registry
	domStore                 *dom.Store
	domReady                 bool
	domErr                   error
	focusedSelector          string
	writingHTML              bool
	interactions             []Interaction
	eventListeners           []eventListenerRecord
	nextEventListenerID      int64
	eventDispatch            *eventDispatchContext
	microtasks               []string
	currentScriptHTML        string
	lastInlineScriptHTML     string
	historyEntries           []historyEntry
	historyIndex             int
	historyScrollRestoration string
	windowName               string
	cookieJar                map[string]string
	timers                   map[int64]timerRecord
	animationFrames          map[int64]animationFrameRecord
	nextTimerID              int64
	nextAnimationFrameID     int64
	runningTimerID           int64
	runningTimerCancelled    bool
}

func NewSession(config SessionConfig) *Session {
	cfg := cloneSessionConfig(config)
	if cfg.URL == "" {
		cfg.URL = DefaultSessionConfig().URL
	}

	session := &Session{
		config:   cfg,
		registry: mocks.NewRegistry(),
	}
	session.applyConfigSeeds()
	return session
}

func (s *Session) applyConfigSeeds() {
	if s == nil {
		return
	}
	registry := s.Registry()
	if registry == nil {
		return
	}

	registry.Location().SetCurrentURL(s.config.URL)

	for key, value := range s.config.LocalStorage {
		registry.Storage().SeedLocal(key, value)
	}

	for key, value := range s.config.SessionStorage {
		registry.Storage().SeedSession(key, value)
	}

	for query, matches := range s.config.MatchMedia {
		registry.MatchMedia().RespondMatches(query, matches)
	}

	if s.config.OpenFailure != "" {
		registry.Open().Fail(s.config.OpenFailure)
	}
	if s.config.CloseFailure != "" {
		registry.Close().Fail(s.config.CloseFailure)
	}
	if s.config.PrintFailure != "" {
		registry.Print().Fail(s.config.PrintFailure)
	}
	if s.config.ScrollFailure != "" {
		registry.Scroll().Fail(s.config.ScrollFailure)
	}
}

func (s *Session) URL() string {
	if s == nil {
		return ""
	}
	if location := s.Registry().Location(); location != nil {
		if current, ok := location.CurrentURL(); ok {
			return current
		}
	}
	return s.config.URL
}

func (s *Session) HTML() string {
	if s == nil {
		return ""
	}
	if s.domStore != nil {
		return s.domStore.SourceHTML()
	}
	return s.config.HTML
}

func (s *Session) NowMs() int64 {
	if s == nil {
		return 0
	}
	return s.scheduler.NowMs()
}

func (s *Session) AdvanceTime(deltaMs int64) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if deltaMs < 0 {
		return fmt.Errorf("advance_time() requires a non-negative delta")
	}
	store, err := s.ensureDOM()
	if err != nil {
		return err
	}
	s.scheduler.Advance(deltaMs)
	return s.settlePendingWork(store)
}

func (s *Session) SetNowMs(nowMs int64) {
	if s == nil {
		return
	}
	s.scheduler.SetNow(nowMs)
}

func (s *Session) ResetTime() {
	if s == nil {
		return
	}
	s.scheduler.Reset()
	s.clearTimers()
	s.clearAnimationFrames()
	s.discardMicrotasks()
}

func (s *Session) Scheduler() *Scheduler {
	if s == nil {
		return nil
	}
	return &s.scheduler
}

func (s *Session) Registry() *mocks.Registry {
	if s == nil {
		return nil
	}
	if s.registry == nil {
		s.registry = mocks.NewRegistry()
		s.applyConfigSeeds()
	}
	return s.registry
}

func (s *Session) Config() SessionConfig {
	if s == nil {
		return DefaultSessionConfig()
	}
	return cloneSessionConfig(s.config)
}

func (s *Session) FocusedSelector() string {
	if s == nil {
		return ""
	}
	return s.focusedSelector
}

func (s *Session) ScrollPosition() (int64, int64) {
	if s == nil {
		return 0, 0
	}
	return s.scrollX, s.scrollY
}

func (s *Session) InteractionLog() []Interaction {
	if s == nil {
		return nil
	}
	out := make([]Interaction, len(s.interactions))
	copy(out, s.interactions)
	return out
}

func (s *Session) ReadClipboard() (string, error) {
	if s == nil {
		return "", fmt.Errorf("session is unavailable")
	}
	if text, ok := s.Registry().Clipboard().SeededText(); ok {
		return text, nil
	}
	return "", fmt.Errorf("clipboard text has not been seeded")
}

func (s *Session) Clipboard() string {
	if s == nil {
		return ""
	}
	if text, ok := s.Registry().Clipboard().SeededText(); ok {
		return text
	}
	return ""
}

func (s *Session) ClipboardWrites() []string {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	return registry.Clipboard().Writes()
}

func (s *Session) FetchCalls() []mocks.FetchCall {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	calls := registry.Fetch().Calls()
	out := make([]mocks.FetchCall, len(calls))
	copy(out, calls)
	return out
}

func (s *Session) FetchResponseRules() []mocks.FetchResponseRule {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	rules := registry.Fetch().ResponseRules()
	out := make([]mocks.FetchResponseRule, len(rules))
	copy(out, rules)
	return out
}

func (s *Session) FetchErrorRules() []mocks.FetchErrorRule {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	rules := registry.Fetch().ErrorRules()
	out := make([]mocks.FetchErrorRule, len(rules))
	copy(out, rules)
	return out
}

func (s *Session) OpenCalls() []mocks.OpenCall {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	calls := registry.Open().Calls()
	out := make([]mocks.OpenCall, len(calls))
	copy(out, calls)
	return out
}

func (s *Session) CloseCalls() []mocks.CloseCall {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	calls := registry.Close().Calls()
	out := make([]mocks.CloseCall, len(calls))
	copy(out, calls)
	return out
}

func (s *Session) PrintCalls() []mocks.PrintCall {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	calls := registry.Print().Calls()
	out := make([]mocks.PrintCall, len(calls))
	copy(out, calls)
	return out
}

func (s *Session) ScrollCalls() []mocks.ScrollCall {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	calls := registry.Scroll().Calls()
	out := make([]mocks.ScrollCall, len(calls))
	copy(out, calls)
	return out
}

func (s *Session) MatchMediaCalls() []mocks.MatchMediaCall {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	calls := registry.MatchMedia().Calls()
	out := make([]mocks.MatchMediaCall, len(calls))
	copy(out, calls)
	return out
}

func (s *Session) MatchMediaListenerCalls() []mocks.MatchMediaListenerCall {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	calls := registry.MatchMedia().ListenerCalls()
	out := make([]mocks.MatchMediaListenerCall, len(calls))
	copy(out, calls)
	return out
}

func (s *Session) DownloadArtifacts() []mocks.DownloadCapture {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	return registry.Downloads().Artifacts()
}

func (s *Session) FileInputSelections() []mocks.FileInputSelection {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	return registry.FileInput().Selections()
}

func (s *Session) DialogAlerts() []string {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	return registry.Dialogs().Alerts()
}

func (s *Session) DialogConfirmMessages() []string {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	return registry.Dialogs().ConfirmMessages()
}

func (s *Session) DialogPromptMessages() []string {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	return registry.Dialogs().PromptMessages()
}

func (s *Session) MatchMediaRules() map[string]bool {
	if s == nil {
		return nil
	}
	registry := s.Registry()
	if registry == nil {
		return nil
	}
	rules := registry.MatchMedia().Rules()
	out := make(map[string]bool, len(rules))
	for i := range rules {
		out[rules[i].Query] = rules[i].Matches
	}
	return out
}

func (s *Session) OpenFailure() string {
	if s == nil {
		return ""
	}
	return s.config.OpenFailure
}

func (s *Session) CloseFailure() string {
	if s == nil {
		return ""
	}
	return s.config.CloseFailure
}

func (s *Session) PrintFailure() string {
	if s == nil {
		return ""
	}
	return s.config.PrintFailure
}

func (s *Session) ScrollFailure() string {
	if s == nil {
		return ""
	}
	return s.config.ScrollFailure
}

func (s *Session) WriteClipboard(text string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	s.Registry().Clipboard().RecordWrite(text)
	return nil
}

func (s *Session) Alert(message string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	s.Registry().Dialogs().RecordAlert(message)
	return nil
}

func (s *Session) Confirm(message string) (bool, error) {
	if s == nil {
		return false, fmt.Errorf("session is unavailable")
	}
	dialogs := s.Registry().Dialogs()
	dialogs.RecordConfirm(message)
	value, ok := dialogs.TakeConfirm()
	if !ok {
		return false, fmt.Errorf("confirm() requires a queued response")
	}
	return value, nil
}

func (s *Session) Prompt(message string) (string, bool, error) {
	if s == nil {
		return "", false, fmt.Errorf("session is unavailable")
	}
	dialogs := s.Registry().Dialogs()
	dialogs.RecordPrompt(message)
	value, submitted, ok := dialogs.TakePrompt()
	if !ok {
		return "", false, fmt.Errorf("prompt() requires a queued response")
	}
	return value, submitted, nil
}

func (s *Session) Fetch(url string) (string, int, string, error) {
	if s == nil {
		return "", 0, "", fmt.Errorf("session is unavailable")
	}
	normalized := strings.TrimSpace(url)
	status, body, err := s.Registry().Fetch().Resolve(normalized)
	if err != nil {
		return "", 0, "", err
	}
	return normalized, status, body, nil
}

func (s *Session) Open(url string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	return s.Registry().Open().Invoke(url)
}

func (s *Session) Close() error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	return s.Registry().Close().Invoke()
}

func (s *Session) Print() error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	return s.Registry().Print().Invoke()
}

func (s *Session) ScrollTo(x, y int64) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if err := s.Registry().Scroll().Invoke("to", x, y); err != nil {
		return err
	}
	s.scrollX = x
	s.scrollY = y
	return nil
}

func (s *Session) ScrollBy(x, y int64) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if err := s.Registry().Scroll().Invoke("by", x, y); err != nil {
		return err
	}
	s.scrollX += x
	s.scrollY += y
	return nil
}

func (s *Session) Navigate(url string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	normalized := strings.TrimSpace(url)
	if normalized == "" {
		return fmt.Errorf("navigate() requires a non-empty URL")
	}
	return s.recordNavigation(resolveHyperlinkURL(s.URL(), normalized))
}

func (s *Session) AssignLocation(url string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	normalized := strings.TrimSpace(url)
	if normalized == "" {
		return fmt.Errorf("location assignment requires a non-empty URL")
	}
	return s.recordNavigation(resolveHyperlinkURL(s.URL(), normalized))
}

func (s *Session) ReplaceLocation(url string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	normalized := strings.TrimSpace(url)
	if normalized == "" {
		return fmt.Errorf("location replacement requires a non-empty URL")
	}
	return s.replaceNavigation(resolveHyperlinkURL(s.URL(), normalized))
}

func (s *Session) ReloadLocation() error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	return s.reloadNavigation()
}

func (s *Session) CaptureDownload(fileName string, bytes []byte) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if strings.TrimSpace(fileName) == "" {
		return fmt.Errorf("capture_download() requires a non-empty file name")
	}
	s.Registry().Downloads().Capture(fileName, bytes)
	return nil
}

func (s *Session) SetFiles(selector string, files []string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, err := s.ensureDOM()
	if err != nil {
		return err
	}
	s.Registry().FileInput().SetFiles(selector, files)
	normalized := strings.TrimSpace(selector)
	if normalized != "" {
		if matches, err := store.Select(normalized); err == nil && len(matches) > 0 {
			if node := store.Node(matches[0]); node != nil && node.Kind == dom.NodeKindElement && node.TagName == "input" && inputType(node) == "file" {
				_ = store.SetUserValidity(matches[0], true)
			}
		}
	}
	return nil
}

func (s *Session) MatchMedia(query string) (bool, error) {
	if s == nil {
		return false, fmt.Errorf("session is unavailable")
	}
	return s.Registry().MatchMedia().Resolve(query)
}

func (s *Session) Click(selector string) (err error) {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, normalized, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	defer func() {
		if err != nil {
			s.discardMicrotasks()
		}
	}()
	s.interactions = append(s.interactions, Interaction{
		Kind:     InteractionKindClick,
		Selector: normalized,
	})
	prevented, err := s.dispatchEventListeners(store, nodeID, "click")
	if err != nil {
		return err
	}
	if s.domStore != nil && s.domStore != store {
		return s.drainMicrotasks(s.domStore)
	}
	if prevented {
		return s.drainMicrotasks(store)
	}
	if err = s.applyClickDefaultAction(normalized); err != nil {
		return err
	}
	return s.drainMicrotasks(store)
}

func (s *Session) Focus(selector string) (err error) {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, normalized, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	defer func() {
		if err != nil {
			s.discardMicrotasks()
		}
	}()
	if err := store.SetFocusedNode(nodeID); err != nil {
		return err
	}
	s.focusedSelector = normalized
	s.interactions = append(s.interactions, Interaction{
		Kind:     InteractionKindFocus,
		Selector: normalized,
	})
	if _, err := s.dispatchTargetEventListeners(store, nodeID, "focus"); err != nil {
		return err
	}
	return s.drainMicrotasks(store)
}

func (s *Session) Blur() (err error) {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	previous := s.focusedSelector
	previousNodeID := dom.NodeID(0)
	if s.domStore != nil {
		previousNodeID = s.domStore.FocusedNodeID()
		s.domStore.ClearFocusedNode()
	}
	s.focusedSelector = ""
	s.interactions = append(s.interactions, Interaction{
		Kind:     InteractionKindBlur,
		Selector: previous,
	})
	if previousNodeID == 0 || s.domStore == nil {
		return nil
	}
	defer func() {
		if err != nil {
			s.discardMicrotasks()
		}
	}()
	if _, err := s.dispatchTargetEventListeners(s.domStore, previousNodeID, "blur"); err != nil {
		return err
	}
	return s.drainMicrotasks(s.domStore)
}

func (s *Session) validateSelector(selector string) (string, error) {
	normalized := strings.TrimSpace(selector)
	if normalized == "" {
		return "", fmt.Errorf("selector must not be empty")
	}
	store, err := s.ensureDOM()
	if err != nil {
		return "", err
	}
	ids, err := store.Select(normalized)
	if err != nil {
		return "", err
	}
	if len(ids) == 0 {
		return "", fmt.Errorf("selector `%s` did not match any element", normalized)
	}
	return normalized, nil
}

func (s *Session) GetAttribute(selector, name string) (string, bool, error) {
	if s == nil {
		return "", false, fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return "", false, err
	}
	return store.GetAttribute(nodeID, name)
}

func (s *Session) HasAttribute(selector, name string) (bool, error) {
	if s == nil {
		return false, fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return false, err
	}
	return store.HasAttribute(nodeID, name)
}

func (s *Session) SetAttribute(selector, name, value string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	return store.SetAttribute(nodeID, name, value)
}

func (s *Session) RemoveAttribute(selector, name string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	return store.RemoveAttribute(nodeID, name)
}

func (s *Session) recordNavigation(url string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	return s.pushHistoryNavigation(url)
}

func (s *Session) ensureDOM() (*dom.Store, error) {
	if s == nil {
		return nil, fmt.Errorf("session is unavailable")
	}
	if s.domReady {
		if s.domErr != nil {
			return nil, s.domErr
		}
		return s.domStore, nil
	}

	store := dom.NewStore()
	if strings.TrimSpace(s.config.HTML) != "" {
		if err := store.BootstrapHTML(s.config.HTML); err != nil {
			s.domErr = err
			s.domReady = true
			return nil, err
		}
	}

	s.domStore = store
	s.domReady = true
	s.syncDocumentState(s.URL())
	if err := s.executeInlineScripts(store); err != nil {
		s.domErr = err
		return nil, err
	}
	return s.domStore, nil
}

func (s *Session) syncDocumentState(url string) {
	if s == nil || s.domStore == nil {
		return
	}
	s.ensureHistoryInitialized()
	s.domStore.SyncTargetFromURL(url)
	s.domStore.SyncCurrentURL(url)
	s.domStore.SyncVisitedURLs(s.visitedHistoryURLs(url))
}

func cloneSessionConfig(config SessionConfig) SessionConfig {
	out := config
	out.LocalStorage = cloneStringMap(config.LocalStorage)
	out.SessionStorage = cloneStringMap(config.SessionStorage)
	out.MatchMedia = cloneBoolMap(config.MatchMedia)
	return out
}

func cloneStringMap(entries map[string]string) map[string]string {
	if len(entries) == 0 {
		return map[string]string{}
	}
	out := make(map[string]string, len(entries))
	for key, value := range entries {
		out[key] = value
	}
	return out
}

func cloneBoolMap(entries map[string]bool) map[string]bool {
	if len(entries) == 0 {
		return map[string]bool{}
	}
	out := make(map[string]bool, len(entries))
	for key, value := range entries {
		out[key] = value
	}
	return out
}
