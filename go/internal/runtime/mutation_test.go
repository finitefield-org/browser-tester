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
	text, err := s.TextContent("#target")
	if err != nil {
		t.Fatalf("TextContent(#target) error = %v", err)
	}
	if got, want := text, "Helloworld"; got != want {
		t.Fatalf("TextContent(#target) = %q, want %q", got, want)
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
	if err := s.SetTextContent("#target", `plain <text> & more`); err != nil {
		t.Fatalf("SetTextContent(#target) error = %v", err)
	}
	if got, want := s.DumpDOM(), `<section id="wrap"><div id="target">plain &lt;text&gt; &amp; more</div><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("DumpDOM() after SetTextContent = %q, want %q", got, want)
	}
	text, err = s.TextContent("#target")
	if err != nil {
		t.Fatalf("TextContent(#target) after SetTextContent error = %v", err)
	}
	if got, want := text, `plain <text> & more`; got != want {
		t.Fatalf("TextContent(#target) after SetTextContent = %q, want %q", got, want)
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
	if got, want := s.DumpDOM(), `<section id="wrap"><a id="bb"></a><div id="target"><i id="ab">a</i>plain &lt;text&gt; &amp; more<i id="be">b</i></div><a id="ae"></a><p id="tail">tail</p></section>`; got != want {
		t.Fatalf("DumpDOM() after InsertAdjacentHTML = %q, want %q", got, want)
	}

	if err := s.SetOuterHTML("#tail", `<aside id="tail2">z</aside>`); err != nil {
		t.Fatalf("SetOuterHTML(#tail) error = %v", err)
	}
	if got, want := s.DumpDOM(), `<section id="wrap"><a id="bb"></a><div id="target"><i id="ab">a</i>plain &lt;text&gt; &amp; more<i id="be">b</i></div><a id="ae"></a><aside id="tail2">z</aside></section>`; got != want {
		t.Fatalf("DumpDOM() after SetOuterHTML = %q, want %q", got, want)
	}
	if _, err := s.OuterHTML("#tail"); err == nil {
		t.Fatalf("OuterHTML(#tail) error = nil, want missing target error")
	}
}

func TestSessionSetTextContentPreservesFocusedNodeAndClearsTargetDescendants(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><div id="target"><span id="child">x</span></div><p id="other">y</p></main>`,
	})

	if err := s.Focus("#target"); err != nil {
		t.Fatalf("Focus(#target) error = %v", err)
	}
	if err := s.Navigate("#child"); err != nil {
		t.Fatalf("Navigate(#child) error = %v", err)
	}

	if err := s.SetTextContent("#target", "plain"); err != nil {
		t.Fatalf("SetTextContent(#target) error = %v", err)
	}
	if got, want := s.FocusedSelector(), "#target"; got != want {
		t.Fatalf("FocusedSelector() after SetTextContent = %q, want %q", got, want)
	}
	if got := s.domStore.TargetNodeID(); got != 0 {
		t.Fatalf("TargetNodeID() after SetTextContent = %d, want 0", got)
	}
	if got, want := s.DumpDOM(), `<main><div id="target">plain</div><p id="other">y</p></main>`; got != want {
		t.Fatalf("DumpDOM() after SetTextContent = %q, want %q", got, want)
	}
}

func TestSessionSetTextContentUpdatesTextareaDefaultValue(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<form id="profile"><textarea id="bio">Base</textarea><button id="reset" type="reset">Reset</button></form>`,
	})

	if err := s.SetTextContent("#bio", "Draft"); err != nil {
		t.Fatalf("SetTextContent(#bio) error = %v", err)
	}
	if got, want := s.DumpDOM(), `<form id="profile"><textarea id="bio">Draft</textarea><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("DumpDOM() after SetTextContent(#bio) = %q, want %q", got, want)
	}

	if err := s.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) error = %v", err)
	}
	if got, want := s.DumpDOM(), `<form id="profile"><textarea id="bio">Draft</textarea><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("DumpDOM() after reset click = %q, want %q", got, want)
	}

	if err := s.TypeText("#bio", "User"); err != nil {
		t.Fatalf("TypeText(#bio) error = %v", err)
	}
	if got, want := s.DumpDOM(), `<form id="profile"><textarea id="bio">User</textarea><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("DumpDOM() after TypeText(#bio) = %q, want %q", got, want)
	}

	if err := s.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) after TypeText error = %v", err)
	}
	if got, want := s.DumpDOM(), `<form id="profile"><textarea id="bio">Draft</textarea><button id="reset" type="reset">Reset</button></form>`; got != want {
		t.Fatalf("DumpDOM() after reset click following TypeText = %q, want %q", got, want)
	}
}

func TestSessionTextareaChildMutationsUpdateResetDefaultValue(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<form id="profile"><textarea id="bio">Base</textarea><button id="reset" type="reset">Reset</button></form>`,
	})

	if err := s.ReplaceChildren("#bio", "Draft"); err != nil {
		t.Fatalf("ReplaceChildren(#bio) error = %v", err)
	}
	if err := s.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) after ReplaceChildren error = %v", err)
	}
	if got, err := s.TextContent("#bio"); err != nil {
		t.Fatalf("TextContent(#bio) after reset click error = %v", err)
	} else if got != "Draft" {
		t.Fatalf("TextContent(#bio) after reset click = %q, want %q", got, "Draft")
	}

	if err := s.SetInnerHTML("#bio", "Fresh"); err != nil {
		t.Fatalf("SetInnerHTML(#bio) second update error = %v", err)
	}
	if err := s.InsertAdjacentHTML("#bio", "beforeend", `<span id="bang">!</span>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(#bio,beforeend) error = %v", err)
	}
	if err := s.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) after InsertAdjacentHTML error = %v", err)
	}
	if got, err := s.TextContent("#bio"); err != nil {
		t.Fatalf("TextContent(#bio) after InsertAdjacentHTML reset error = %v", err)
	} else if got != "Fresh!" {
		t.Fatalf("TextContent(#bio) after InsertAdjacentHTML reset = %q, want %q", got, "Fresh!")
	}

	if err := s.SetInnerHTML("#bio", "Fresh"); err != nil {
		t.Fatalf("SetInnerHTML(#bio) third update error = %v", err)
	}
	if err := s.InsertAdjacentHTML("#bio", "beforeend", `<span id="bang">!</span>`); err != nil {
		t.Fatalf("InsertAdjacentHTML(#bio,beforeend) second error = %v", err)
	}
	if err := s.RemoveNode("#bang"); err != nil {
		t.Fatalf("RemoveNode(#bang) error = %v", err)
	}
	if err := s.Click("#reset"); err != nil {
		t.Fatalf("Click(#reset) after RemoveNode error = %v", err)
	}
	if got, err := s.TextContent("#bio"); err != nil {
		t.Fatalf("TextContent(#bio) after RemoveNode reset error = %v", err)
	} else if got != "Fresh" {
		t.Fatalf("TextContent(#bio) after RemoveNode reset = %q, want %q", got, "Fresh")
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
	if _, err := nilSession.TextContent("#target"); err == nil {
		t.Fatalf("nil TextContent() error = nil, want session unavailable error")
	}
	if _, err := nilSession.OuterHTML("#target"); err == nil {
		t.Fatalf("nil OuterHTML() error = nil, want session unavailable error")
	}
	if err := nilSession.SetInnerHTML("#target", "<p>x</p>"); err == nil {
		t.Fatalf("nil SetInnerHTML() error = nil, want session unavailable error")
	}
	if err := nilSession.ReplaceChildren("#target", "<p>x</p>"); err == nil {
		t.Fatalf("nil ReplaceChildren() error = nil, want session unavailable error")
	}
	if err := nilSession.SetTextContent("#target", "x"); err == nil {
		t.Fatalf("nil SetTextContent() error = nil, want session unavailable error")
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
	if err := nilSession.CloneNode("#target", true); err == nil {
		t.Fatalf("nil CloneNode() error = nil, want session unavailable error")
	}

	s := NewSession(SessionConfig{
		HTML: `<div id="top">plain</div><section id="root"><span id="inner">x</span></section>`,
	})

	if _, err := s.InnerHTML("#missing"); err == nil {
		t.Fatalf("InnerHTML(#missing) error = nil, want missing target error")
	}
	if _, err := s.TextContent("#missing"); err == nil {
		t.Fatalf("TextContent(#missing) error = nil, want missing target error")
	}
	if _, err := s.OuterHTML("#missing"); err == nil {
		t.Fatalf("OuterHTML(#missing) error = nil, want missing target error")
	}
	if err := s.SetInnerHTML("#missing", "<p>x</p>"); err == nil {
		t.Fatalf("SetInnerHTML(#missing) error = nil, want missing target error")
	}
	if err := s.ReplaceChildren("#missing", "<p>x</p>"); err == nil {
		t.Fatalf("ReplaceChildren(#missing) error = nil, want missing target error")
	}
	if err := s.SetTextContent("#missing", "x"); err == nil {
		t.Fatalf("SetTextContent(#missing) error = nil, want missing target error")
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
	if err := s.CloneNode("#missing", true); err == nil {
		t.Fatalf("CloneNode(#missing) error = nil, want missing target error")
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
	if err := s.CloneNode("#top", true); err != nil {
		t.Fatalf("CloneNode(#top) error = %v, want clone to succeed for top-level nodes", err)
	}
	if got, want := s.DumpDOM(), `<div id="top">plain</div><div id="top">plain</div><section id="root"><span id="inner">x</span></section>`; got != want {
		t.Fatalf("DumpDOM() after CloneNode(#top) = %q, want %q", got, want)
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

func TestSessionWriteHTMLRestoresSessionStateOnHostFailure(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/app",
		HTML: `<main><script>host:setWindowName("alpha"); host:setDocumentCookie("theme=dark"); host:historySetScrollRestoration("manual"); host:historyPushState("step-1", "", "/step-1")</script></main>`,
	})

	if _, err := s.ensureDOM(); err != nil {
		t.Fatalf("ensureDOM() error = %v", err)
	}
	if got := s.WindowName(); got != "alpha" {
		t.Fatalf("WindowName() before failed WriteHTML = %q, want alpha", got)
	}
	if got := s.documentCookie(); got != "theme=dark" {
		t.Fatalf("documentCookie() before failed WriteHTML = %q, want theme=dark", got)
	}
	if got, want := s.URL(), "https://example.test/step-1"; got != want {
		t.Fatalf("URL() before failed WriteHTML = %q, want %q", got, want)
	}
	if got, want := s.windowHistoryLength(), 2; got != want {
		t.Fatalf("windowHistoryLength() before failed WriteHTML = %d, want %d", got, want)
	}
	if got, ok := s.windowHistoryState(); !ok || got != "step-1" {
		t.Fatalf("windowHistoryState() before failed WriteHTML = (%q, %v), want (\"step-1\", true)", got, ok)
	}
	if got := s.windowHistoryScrollRestoration(); got != "manual" {
		t.Fatalf("windowHistoryScrollRestoration() before failed WriteHTML = %q, want manual", got)
	}
	if got := s.NavigationLog(); len(got) != 1 || got[0] != "https://example.test/step-1" {
		t.Fatalf("NavigationLog() before failed WriteHTML = %#v, want one navigation to step-1", got)
	}

	if err := s.WriteHTML(`<main><script>host:setWindowName("beta"); host:setDocumentCookie("lang=en"); host:historySetScrollRestoration("auto"); host:historyPushState("step-2", "", "/step-2"); host:locationAssign("/next"); host:doesNotExist()</script></main>`); err == nil {
		t.Fatalf("WriteHTML() error = nil, want host failure")
	}

	if got := s.WindowName(); got != "alpha" {
		t.Fatalf("WindowName() after failed WriteHTML = %q, want alpha", got)
	}
	if got := s.documentCookie(); got != "theme=dark" {
		t.Fatalf("documentCookie() after failed WriteHTML = %q, want theme=dark", got)
	}
	if got, want := s.URL(), "https://example.test/step-1"; got != want {
		t.Fatalf("URL() after failed WriteHTML = %q, want %q", got, want)
	}
	if got, want := s.windowHistoryLength(), 2; got != want {
		t.Fatalf("windowHistoryLength() after failed WriteHTML = %d, want %d", got, want)
	}
	if got, ok := s.windowHistoryState(); !ok || got != "step-1" {
		t.Fatalf("windowHistoryState() after failed WriteHTML = (%q, %v), want (\"step-1\", true)", got, ok)
	}
	if got := s.windowHistoryScrollRestoration(); got != "manual" {
		t.Fatalf("windowHistoryScrollRestoration() after failed WriteHTML = %q, want manual", got)
	}
	if got := s.NavigationLog(); len(got) != 1 || got[0] != "https://example.test/step-1" {
		t.Fatalf("NavigationLog() after failed WriteHTML = %#v, want one navigation to step-1", got)
	}
	if got, want := s.LastInlineScriptHTML(), `<script>host:setWindowName("alpha"); host:setDocumentCookie("theme=dark"); host:historySetScrollRestoration("manual"); host:historyPushState("step-1", "", "/step-1")</script>`; got != want {
		t.Fatalf("LastInlineScriptHTML() after failed WriteHTML = %q, want %q", got, want)
	}
}

func TestSessionWriteHTMLRejectsInvalidHistoryScrollRestorationWithoutMutatingSessionState(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/app",
		HTML: `<main><script>host:setWindowName("alpha"); host:setDocumentCookie("theme=dark"); host:historySetScrollRestoration("manual"); host:historyPushState("step-1", "", "/step-1")</script></main>`,
	})

	if _, err := s.ensureDOM(); err != nil {
		t.Fatalf("ensureDOM() error = %v", err)
	}
	if err := s.WriteHTML(`<main><script>host:historySetScrollRestoration("sideways")</script></main>`); err == nil {
		t.Fatalf("WriteHTML() error = nil, want invalid history scroll restoration failure")
	}

	if got := s.WindowName(); got != "alpha" {
		t.Fatalf("WindowName() after rejected WriteHTML = %q, want alpha", got)
	}
	if got := s.documentCookie(); got != "theme=dark" {
		t.Fatalf("documentCookie() after rejected WriteHTML = %q, want theme=dark", got)
	}
	if got, want := s.URL(), "https://example.test/step-1"; got != want {
		t.Fatalf("URL() after rejected WriteHTML = %q, want %q", got, want)
	}
	if got, want := s.windowHistoryLength(), 2; got != want {
		t.Fatalf("windowHistoryLength() after rejected WriteHTML = %d, want %d", got, want)
	}
	if got, ok := s.windowHistoryState(); !ok || got != "step-1" {
		t.Fatalf("windowHistoryState() after rejected WriteHTML = (%q, %v), want (\"step-1\", true)", got, ok)
	}
	if got := s.windowHistoryScrollRestoration(); got != "manual" {
		t.Fatalf("windowHistoryScrollRestoration() after rejected WriteHTML = %q, want manual", got)
	}
	if got := s.NavigationLog(); len(got) != 1 || got[0] != "https://example.test/step-1" {
		t.Fatalf("NavigationLog() after rejected WriteHTML = %#v, want one navigation to step-1", got)
	}
}
