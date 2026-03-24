package browsertester

import rt "browsertester/internal/runtime"

type HarnessBuilder struct {
	config rt.SessionConfig
}

func NewHarnessBuilder() *HarnessBuilder {
	return &HarnessBuilder{config: rt.DefaultSessionConfig()}
}

func (b *HarnessBuilder) URL(url string) *HarnessBuilder {
	b.config.URL = url
	return b
}

func (b *HarnessBuilder) HTML(html string) *HarnessBuilder {
	b.config.HTML = html
	return b
}

func (b *HarnessBuilder) LocalStorage(entries map[string]string) *HarnessBuilder {
	b.config.LocalStorage = cloneStringMap(entries)
	return b
}

func (b *HarnessBuilder) SessionStorage(entries map[string]string) *HarnessBuilder {
	b.config.SessionStorage = cloneStringMap(entries)
	return b
}

func (b *HarnessBuilder) RandomSeed(seed int64) *HarnessBuilder {
	b.config.RandomSeed = seed
	b.config.HasRandomSeed = true
	return b
}

func (b *HarnessBuilder) MatchMedia(entries map[string]bool) *HarnessBuilder {
	b.config.MatchMedia = cloneBoolMap(entries)
	return b
}

func (b *HarnessBuilder) OpenFailure(message string) *HarnessBuilder {
	b.config.OpenFailure = message
	return b
}

func (b *HarnessBuilder) CloseFailure(message string) *HarnessBuilder {
	b.config.CloseFailure = message
	return b
}

func (b *HarnessBuilder) PrintFailure(message string) *HarnessBuilder {
	b.config.PrintFailure = message
	return b
}

func (b *HarnessBuilder) ScrollFailure(message string) *HarnessBuilder {
	b.config.ScrollFailure = message
	return b
}

func (b *HarnessBuilder) Build() (*Harness, error) {
	session := rt.NewSession(b.config)
	return &Harness{session: session}, nil
}

func FromHTML(html string) (*Harness, error) {
	return NewHarnessBuilder().HTML(html).Build()
}

func FromHTMLWithURL(url, html string) (*Harness, error) {
	return NewHarnessBuilder().URL(url).HTML(html).Build()
}

func FromHTMLWithLocalStorage(html string, entries map[string]string) (*Harness, error) {
	return NewHarnessBuilder().HTML(html).LocalStorage(entries).Build()
}

func FromHTMLWithURLAndLocalStorage(url, html string, entries map[string]string) (*Harness, error) {
	return NewHarnessBuilder().URL(url).HTML(html).LocalStorage(entries).Build()
}

func FromHTMLWithSessionStorage(html string, entries map[string]string) (*Harness, error) {
	return NewHarnessBuilder().HTML(html).SessionStorage(entries).Build()
}

func FromHTMLWithURLAndSessionStorage(url, html string, entries map[string]string) (*Harness, error) {
	return NewHarnessBuilder().URL(url).HTML(html).SessionStorage(entries).Build()
}

type Harness struct {
	session *rt.Session
}

func (h *Harness) URL() string {
	if h == nil || h.session == nil {
		return ""
	}
	return h.session.URL()
}

func (h *Harness) HTML() string {
	if h == nil || h.session == nil {
		return ""
	}
	return h.session.HTML()
}

func (h *Harness) NowMs() int64 {
	if h == nil || h.session == nil {
		return 0
	}
	return h.session.NowMs()
}

func (h *Harness) Debug() DebugView {
	if h == nil || h.session == nil {
		return DebugView{}
	}
	return DebugView{session: h.session}
}

func (h *Harness) Mocks() MockRegistryView {
	if h == nil || h.session == nil {
		return MockRegistryView{}
	}
	return MockRegistryView{registry: h.session.Registry()}
}

func (h *Harness) ReadClipboard() (string, error) {
	if h == nil || h.session == nil {
		return "", NewError(ErrorKindMock, "clipboard is unavailable")
	}
	text, err := h.session.ReadClipboard()
	if err != nil {
		return "", NewError(ErrorKindMock, err.Error())
	}
	return text, nil
}

func (h *Harness) WriteClipboard(text string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindMock, "clipboard is unavailable")
	}
	if err := h.session.WriteClipboard(text); err != nil {
		return NewError(ErrorKindMock, err.Error())
	}
	return nil
}

func (h *Harness) Fetch(url string) (FetchResponse, error) {
	if h == nil || h.session == nil {
		return FetchResponse{}, NewError(ErrorKindMock, "fetch is unavailable")
	}
	normalized, status, body, err := h.session.Fetch(url)
	if err != nil {
		return FetchResponse{}, NewError(ErrorKindMock, err.Error())
	}
	return FetchResponse{
		URL:    normalized,
		Status: status,
		Body:   body,
	}, nil
}

func (h *Harness) Alert(message string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindMock, "alert is unavailable")
	}
	if err := h.session.Alert(message); err != nil {
		return NewError(ErrorKindMock, err.Error())
	}
	return nil
}

func (h *Harness) Confirm(message string) (bool, error) {
	if h == nil || h.session == nil {
		return false, NewError(ErrorKindMock, "confirm is unavailable")
	}
	value, err := h.session.Confirm(message)
	if err != nil {
		return false, NewError(ErrorKindMock, err.Error())
	}
	return value, nil
}

func (h *Harness) Prompt(message string) (string, bool, error) {
	if h == nil || h.session == nil {
		return "", false, NewError(ErrorKindMock, "prompt is unavailable")
	}
	value, submitted, err := h.session.Prompt(message)
	if err != nil {
		return "", false, NewError(ErrorKindMock, err.Error())
	}
	return value, submitted, nil
}

func (h *Harness) Open(url string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindMock, "open is unavailable")
	}
	if err := h.session.Open(url); err != nil {
		return NewError(ErrorKindMock, err.Error())
	}
	return nil
}

func (h *Harness) Close() error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindMock, "close is unavailable")
	}
	if err := h.session.Close(); err != nil {
		return NewError(ErrorKindMock, err.Error())
	}
	return nil
}

func (h *Harness) Print() error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindMock, "print is unavailable")
	}
	if err := h.session.Print(); err != nil {
		return NewError(ErrorKindMock, err.Error())
	}
	return nil
}

func (h *Harness) ScrollTo(x, y int64) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindMock, "scroll is unavailable")
	}
	if err := h.session.ScrollTo(x, y); err != nil {
		return NewError(ErrorKindMock, err.Error())
	}
	return nil
}

func (h *Harness) ScrollBy(x, y int64) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindMock, "scroll is unavailable")
	}
	if err := h.session.ScrollBy(x, y); err != nil {
		return NewError(ErrorKindMock, err.Error())
	}
	return nil
}

func (h *Harness) Navigate(url string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindMock, "navigate is unavailable")
	}
	if err := h.session.Navigate(url); err != nil {
		return NewError(ErrorKindMock, err.Error())
	}
	return nil
}

func (h *Harness) Click(selector string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindEvent, "click is unavailable")
	}
	if err := h.session.Click(selector); err != nil {
		return NewError(ErrorKindEvent, err.Error())
	}
	return nil
}

func (h *Harness) Focus(selector string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindEvent, "focus is unavailable")
	}
	if err := h.session.Focus(selector); err != nil {
		return NewError(ErrorKindEvent, err.Error())
	}
	return nil
}

func (h *Harness) Blur() error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindEvent, "blur is unavailable")
	}
	if err := h.session.Blur(); err != nil {
		return NewError(ErrorKindEvent, err.Error())
	}
	return nil
}

func (h *Harness) GetAttribute(selector, name string) (string, bool, error) {
	if h == nil || h.session == nil {
		return "", false, NewError(ErrorKindDOM, "get attribute is unavailable")
	}
	value, ok, err := h.session.GetAttribute(selector, name)
	if err != nil {
		return "", false, NewError(ErrorKindDOM, err.Error())
	}
	return value, ok, nil
}

func (h *Harness) HasAttribute(selector, name string) (bool, error) {
	if h == nil || h.session == nil {
		return false, NewError(ErrorKindDOM, "has attribute is unavailable")
	}
	ok, err := h.session.HasAttribute(selector, name)
	if err != nil {
		return false, NewError(ErrorKindDOM, err.Error())
	}
	return ok, nil
}

func (h *Harness) SetAttribute(selector, name, value string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindDOM, "set attribute is unavailable")
	}
	if err := h.session.SetAttribute(selector, name, value); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

func (h *Harness) RemoveAttribute(selector, name string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindDOM, "remove attribute is unavailable")
	}
	if err := h.session.RemoveAttribute(selector, name); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

func (h *Harness) CaptureDownload(fileName string, bytes []byte) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindMock, "capture download is unavailable")
	}
	if err := h.session.CaptureDownload(fileName, bytes); err != nil {
		return NewError(ErrorKindMock, err.Error())
	}
	return nil
}

func (h *Harness) SetFiles(selector string, files []string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindMock, "set files is unavailable")
	}
	if err := h.session.SetFiles(selector, files); err != nil {
		return NewError(ErrorKindMock, err.Error())
	}
	return nil
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
