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

func (v DebugView) DumpDOM() string {
	if v.session == nil {
		return ""
	}
	return v.session.DumpDOM()
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
