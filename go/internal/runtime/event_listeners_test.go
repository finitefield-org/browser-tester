package runtime

import "testing"

func TestSessionEventListenersInspection(t *testing.T) {
	s := NewSession(DefaultSessionConfig())
	if err := s.WriteHTML(`<main><button id="btn">Go</button><div id="out"></div><script>host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#out", "beforeend", "<span>once</span>")', "capture", true); host:addEventListener("#btn", "click", 'host:insertAdjacentHTML("#out", "beforeend", "<span>stay</span>")', "bubble")</script></main>`); err != nil {
		t.Fatalf("WriteHTML() error = %v", err)
	}

	listeners := s.EventListeners()
	if len(listeners) != 2 {
		t.Fatalf("EventListeners() len = %d, want 2", len(listeners))
	}
	if listeners[0].NodeID == 0 || listeners[0].NodeID != listeners[1].NodeID {
		t.Fatalf("EventListeners() node ids = %#v, want same non-zero node id", listeners)
	}
	if listeners[0].Event != "click" || listeners[0].Phase != "capture" || !listeners[0].Once {
		t.Fatalf("EventListeners()[0] = %#v, want capture once click listener", listeners[0])
	}
	if listeners[1].Event != "click" || listeners[1].Phase != "bubble" || listeners[1].Once {
		t.Fatalf("EventListeners()[1] = %#v, want bubble persistent click listener", listeners[1])
	}

	listeners[0].Source = "mutated"
	fresh := s.EventListeners()
	if fresh[0].Source != `host:insertAdjacentHTML("#out", "beforeend", "<span>once</span>")` {
		t.Fatalf("EventListeners() reread source = %q, want original source", fresh[0].Source)
	}

	if err := s.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) error = %v", err)
	}
	after := s.EventListeners()
	if len(after) != 1 {
		t.Fatalf("EventListeners() after click len = %d, want 1", len(after))
	}
	if after[0].Phase != "bubble" || after[0].Once {
		t.Fatalf("EventListeners() after click = %#v, want persistent bubble listener", after[0])
	}

	var nilSession *Session
	if got := nilSession.EventListeners(); got != nil {
		t.Fatalf("nil EventListeners() = %#v, want nil", got)
	}
}
