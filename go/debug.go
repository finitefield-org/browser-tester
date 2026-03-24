package browsertester

import rt "browsertester/internal/runtime"

type DebugView struct {
	session *rt.Session
}

func (v DebugView) URL() string {
	if v.session == nil {
		return ""
	}
	return v.session.URL()
}

func (v DebugView) HTML() string {
	if v.session == nil {
		return ""
	}
	return v.session.HTML()
}

func (v DebugView) NowMs() int64 {
	if v.session == nil {
		return 0
	}
	return v.session.NowMs()
}
