package runtime

import "testing"

func TestSessionMutationHelpersReadAndWriteDOM(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<section id="wrap"><div id="target"><p>Hello</p><span>world</span></div><p id="tail">tail</p></section>`,
	})

	inner, err := s.InnerHTML("#target")
	if err != nil {
		t.Fatalf("InnerHTML(#target) error = %v", err)
	}
	if got, want := inner, `<p>Hello</p><span>world</span>`; got != want {
		t.Fatalf("InnerHTML(#target) = %q, want %q", got, want)
	}

	outer, err := s.OuterHTML("#target")
	if err != nil {
		t.Fatalf("OuterHTML(#target) error = %v", err)
	}
	if got, want := outer, `<div id="target"><p>Hello</p><span>world</span></div>`; got != want {
		t.Fatalf("OuterHTML(#target) = %q, want %q", got, want)
	}

	if err := s.SetInnerHTML("#target", `<em id="next">updated</em>tail`); err != nil {
		t.Fatalf("SetInnerHTML(#target) error = %v", err)
	}
	if got, want := s.DumpDOM(), `<section id="wrap"><div id="target"><em id="next">updated</em>tail</div><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("DumpDOM() after SetInnerHTML = %q, want %q", got, want)
	}

	if err := s.InsertAdjacentHTML("#target", "beforebegin", `<a id="bb"></a>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(beforebegin) error = %v", err)
	}
	if err := s.InsertAdjacentHTML("#target", "afterbegin", `<i id="ab">a</i>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(afterbegin) error = %v", err)
	}
	if err := s.InsertAdjacentHTML("#target", "beforeend", `<i id="be">b</i>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(beforeend) error = %v", err)
	}
	if err := s.InsertAdjacentHTML("#target", "afterend", `<a id="ae"></a>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(afterend) error = %v", err)
	}
	if got, want := s.DumpDOM(), `<section id="wrap"><a id="bb"></a><div id="target"><i id="ab">a</i><em id="next">updated</em>tail<i id="be">b</i></div><a id="ae"></a><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("DumpDOM() after InsertAdjacentHTML = %q, want %q", got, want)
	}

	if err := s.SetOuterHTML("#tail", `<aside id="tail2">z</aside>`); err != nil {
		t.Fatalf("SetOuterHTML(#tail) error = %v", err)
	}
	if got, want := s.DumpDOM(), `<section id="wrap"><a id="bb"></a><div id="target"><i id="ab">a</i><em id="next">updated</em>tail<i id="be">b</i></div><a id="ae"></a><aside id="tail2">z</aside></section>`; got != want {
		t.Fatalf("DumpDOM() after SetOuterHTML = %q, want %q", got, want)
	}
	if _, err := s.OuterHTML("#tail"); err == nil {
		t.Fatalf("OuterHTML(#tail) error = nil, want missing target error")
	}
}

func TestSessionRemoveNodeRemovesSubtreeAndClearsFocus(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><div id="remove"><span id="child">x</span></div><input id="name"></main>`,
	})

	if err := s.Focus("#remove"); err != nil {
		t.Fatalf("Focus(#remove) error = %v", err)
	}
	if got, want := s.FocusedSelector(), "#remove"; got != want {
		t.Fatalf("FocusedSelector() before RemoveNode = %q, want %q", got, want)
	}

	if err := s.RemoveNode("#remove"); err != nil {
		t.Fatalf("RemoveNode(#remove) error = %v", err)
	}
	if got, want := s.DumpDOM(), `<main><input id="name"></main>`; got != want {
		t.Fatalf("DumpDOM() after RemoveNode = %q, want %q", got, want)
	}
	if got := s.FocusedSelector(); got != "" {
		t.Fatalf("FocusedSelector() after RemoveNode = %q, want empty", got)
	}
	if _, err := s.OuterHTML("#child"); err == nil {
		t.Fatalf("OuterHTML(#child) error = nil, want missing target error")
	}
}

func TestSessionMutationHelpersRejectInvalidInputs(t *testing.T) {
	var nilSession *Session
	if _, err := nilSession.InnerHTML("#target"); err == nil {
		t.Fatalf("nil InnerHTML() error = nil, want session unavailable error")
	}
	if _, err := nilSession.OuterHTML("#target"); err == nil {
		t.Fatalf("nil OuterHTML() error = nil, want session unavailable error")
	}
	if err := nilSession.SetInnerHTML("#target", "<p>x</p>"); err == nil {
		t.Fatalf("nil SetInnerHTML() error = nil, want session unavailable error")
	}
	if err := nilSession.SetOuterHTML("#target", "<p>x</p>"); err == nil {
		t.Fatalf("nil SetOuterHTML() error = nil, want session unavailable error")
	}
	if err := nilSession.InsertAdjacentHTML("#target", "beforeend", "<p>x</p>"); err == nil {
		t.Fatalf("nil InsertAdjacentHTML() error = nil, want session unavailable error")
	}
	if err := nilSession.RemoveNode("#target"); err == nil {
		t.Fatalf("nil RemoveNode() error = nil, want session unavailable error")
	}

	s := NewSession(SessionConfig{
		HTML: `<div id="top">plain</div><section id="root"><span id="inner">x</span></section>`,
	})

	if _, err := s.InnerHTML("#missing"); err == nil {
		t.Fatalf("InnerHTML(#missing) error = nil, want missing target error")
	}
	if _, err := s.OuterHTML("#missing"); err == nil {
		t.Fatalf("OuterHTML(#missing) error = nil, want missing target error")
	}
	if err := s.SetInnerHTML("#missing", "<p>x</p>"); err == nil {
		t.Fatalf("SetInnerHTML(#missing) error = nil, want missing target error")
	}
	if err := s.SetOuterHTML("#missing", "<p>x</p>"); err == nil {
		t.Fatalf("SetOuterHTML(#missing) error = nil, want missing target error")
	}
	if err := s.InsertAdjacentHTML("#missing", "beforeend", "<p>x</p>"); err == nil {
		t.Fatalf("InsertAdjacentHTML(#missing) error = nil, want missing target error")
	}
	if err := s.RemoveNode("#missing"); err == nil {
		t.Fatalf("RemoveNode(#missing) error = nil, want missing target error")
	}

	if _, err := s.InnerHTML("div[item="); err == nil {
		t.Fatalf("InnerHTML(div[item=) error = nil, want selector syntax error")
	}
	if err := s.InsertAdjacentHTML("#inner", "sideways", "<p>x</p>"); err == nil {
		t.Fatalf("InsertAdjacentHTML(invalid position) error = nil, want invalid position error")
	}

	if err := s.SetOuterHTML("#top", `<article id="new"></article>`); err == nil {
		t.Fatalf("SetOuterHTML(#top) error = nil, want document-parent restriction")
	}
	if err := s.InsertAdjacentHTML("#top", "beforebegin", `<a id="bb"></a>`); err == nil {
		t.Fatalf("InsertAdjacentHTML(#top,beforebegin) error = nil, want document-parent restriction")
	}
	if err := s.InsertAdjacentHTML("#top", "afterend", `<a id="ae"></a>`); err == nil {
		t.Fatalf("InsertAdjacentHTML(#top,afterend) error = nil, want document-parent restriction")
	}
}

func TestSessionWriteHTMLReplacesDocumentAndResetsTransientState(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><button id="btn">old</button><div id="out">before</div><script>host:addEventListener("#btn", "click", 'host:setInnerHTML("#out", "old-listener")')</script></main>`,
	})

	if err := s.Focus("#btn"); err != nil {
		t.Fatalf("Focus(#btn) error = %v", err)
	}
	if err := s.ScrollTo(4, 5); err != nil {
		t.Fatalf("ScrollTo(4, 5) error = %v", err)
	}

	markup := `<main><button id="btn">new</button><div id="out">fresh</div><script>host:setInnerHTML("#out", "written")</script></main>`
	if err := s.WriteHTML(markup); err != nil {
		t.Fatalf("WriteHTML() error = %v", err)
	}

	if got, want := s.DumpDOM(), `<main><button id="btn">new</button><div id="out">written</div><script>host:setInnerHTML("#out", "written")</script></main>`; got != want {
		t.Fatalf("DumpDOM() after WriteHTML = %q, want %q", got, want)
	}
	if got, want := s.HTML(), markup; got != want {
		t.Fatalf("HTML() after WriteHTML = %q, want %q", got, want)
	}
	if got := s.FocusedSelector(); got != "" {
		t.Fatalf("FocusedSelector() after WriteHTML = %q, want empty", got)
	}
	if gotX, gotY := s.ScrollPosition(); gotX != 0 || gotY != 0 {
		t.Fatalf("ScrollPosition() after WriteHTML = (%d, %d), want (0, 0)", gotX, gotY)
	}

	if err := s.Click("#btn"); err != nil {
		t.Fatalf("Click(#btn) after WriteHTML error = %v", err)
	}
	if got, want := s.DumpDOM(), `<main><button id="btn">new</button><div id="out">written</div><script>host:setInnerHTML("#out", "written")</script></main>`; got != want {
		t.Fatalf("DumpDOM() after Click on rewritten document = %q, want %q", got, want)
	}
}

func TestSessionWriteHTMLRejectsInvalidMarkupWithoutMutatingDocument(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><div id="out">old</div></main>`,
	})

	if err := s.WriteHTML(`<main><div id="broken"></main>`); err == nil {
		t.Fatalf("WriteHTML(invalid) error = nil, want parse error")
	}
	if got, want := s.DumpDOM(), `<main><div id="out">old</div></main>`; got != want {
		t.Fatalf("DumpDOM() after failed WriteHTML = %q, want %q", got, want)
	}
	if got, want := s.HTML(), `<main><div id="out">old</div></main>`; got != want {
		t.Fatalf("HTML() after failed WriteHTML = %q, want %q", got, want)
	}
}
