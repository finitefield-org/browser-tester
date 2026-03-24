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
	config          SessionConfig
	scheduler       Scheduler
	scrollX         int64
	scrollY         int64
	registry        *mocks.Registry
	domStore        *dom.Store
	domReady        bool
	domErr          error
	focusedSelector string
	interactions    []Interaction
	eventListeners  []eventListenerRecord
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
	s.scheduler.Advance(deltaMs)
	return nil
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
	s.Registry().Location().RecordNavigation(normalized)
	s.scrollX = 0
	s.scrollY = 0
	return nil
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
	s.Registry().FileInput().SetFiles(selector, files)
	return nil
}

func (s *Session) MatchMedia(query string) (bool, error) {
	if s == nil {
		return false, fmt.Errorf("session is unavailable")
	}
	return s.Registry().MatchMedia().Resolve(query)
}

func (s *Session) Click(selector string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, normalized, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	s.interactions = append(s.interactions, Interaction{
		Kind:     InteractionKindClick,
		Selector: normalized,
	})
	if err := s.dispatchEventListeners(store, nodeID, "click"); err != nil {
		return err
	}
	if err := s.applyClickDefaultAction(normalized); err != nil {
		return err
	}
	return nil
}

func (s *Session) Focus(selector string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, normalized, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	s.focusedSelector = normalized
	s.interactions = append(s.interactions, Interaction{
		Kind:     InteractionKindFocus,
		Selector: normalized,
	})
	if err := s.dispatchEventListeners(store, nodeID, "focus"); err != nil {
		return err
	}
	return nil
}

func (s *Session) Blur() error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	previous := s.focusedSelector
	s.focusedSelector = ""
	s.interactions = append(s.interactions, Interaction{
		Kind:     InteractionKindBlur,
		Selector: previous,
	})
	if previous == "" {
		return nil
	}
	store, nodeID, _, _, err := s.resolveActionTarget(previous)
	if err != nil {
		return nil
	}
	if err := s.dispatchEventListeners(store, nodeID, "blur"); err != nil {
		return err
	}
	return nil
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
	if err := s.executeInlineScripts(store); err != nil {
		s.domErr = err
		return nil, err
	}
	return s.domStore, nil
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
