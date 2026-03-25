package runtime

import (
	"fmt"
	"strings"

	"browsertester/internal/dom"
	"browsertester/internal/script"
)

func (s *Session) runScriptOnStore(store *dom.Store, source string) (script.Value, error) {
	if s == nil {
		return script.UndefinedValue(), fmt.Errorf("session is unavailable")
	}
	runtime := script.NewRuntime(&inlineScriptHost{session: s, store: store})
	result, err := runtime.Dispatch(script.DispatchRequest{Source: source})
	if err != nil {
		return script.UndefinedValue(), err
	}
	return result.Value, nil
}

func (s *Session) executeInlineScripts(store *dom.Store) (err error) {
	if s == nil || store == nil {
		return nil
	}
	defer func() {
		if err != nil {
			s.discardMicrotasks()
		}
	}()
	nodes := store.Nodes()
	for _, node := range nodes {
		if s.domStore != nil && s.domStore != store {
			return nil
		}
		if node == nil || node.Kind != dom.NodeKindElement || node.TagName != "script" {
			continue
		}
		if store.Node(node.ID) == nil {
			continue
		}
		source := store.TextContentForNode(node.ID)
		if strings.TrimSpace(source) == "" {
			continue
		}
		outerHTML, err := store.OuterHTMLForNode(node.ID)
		if err != nil {
			return err
		}
		if _, err := s.runInlineScriptOnStore(store, outerHTML, source); err != nil {
			return err
		}
		if err := s.drainMicrotasks(store); err != nil {
			return err
		}
		if s.domStore != nil && s.domStore != store {
			return nil
		}
	}
	return nil
}

func (s *Session) runInlineScriptOnStore(store *dom.Store, currentScript string, source string) (script.Value, error) {
	if s == nil {
		return script.UndefinedValue(), fmt.Errorf("session is unavailable")
	}
	prev := s.currentScriptHTML
	s.currentScriptHTML = currentScript
	s.lastInlineScriptHTML = currentScript
	defer func() {
		s.currentScriptHTML = prev
	}()
	return s.runScriptOnStore(store, source)
}

type inlineScriptHost struct {
	session *Session
	store   *dom.Store
}

func (h *inlineScriptHost) currentStore() *dom.Store {
	if h == nil {
		return nil
	}
	if h.session != nil && h.session.domStore != nil {
		return h.session.domStore
	}
	return h.store
}

func (h *inlineScriptHost) Call(method string, args []script.Value) (script.Value, error) {
	store := h.currentStore()
	if h == nil || store == nil {
		return script.UndefinedValue(), fmt.Errorf("inline script host is unavailable")
	}

	switch method {
	case "querySelector":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, ok, err := store.QuerySelector(selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if !ok {
			return script.UndefinedValue(), nil
		}
		return script.StringValue(fmt.Sprintf("%d", nodeID)), nil

	case "querySelectorAll":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodes, err := store.QuerySelectorAll(selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		return script.NumberValue(float64(nodes.Length())), nil

	case "matches":
		nodeID, err := scriptNodeIDArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		selector, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		matched, err := store.Matches(nodeID, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		return script.BoolValue(matched), nil

	case "closest":
		nodeID, err := scriptNodeIDArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		selector, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		closestID, ok, err := store.Closest(nodeID, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if !ok {
			return script.UndefinedValue(), nil
		}
		return script.StringValue(fmt.Sprintf("%d", closestID)), nil

	case "innerHTML":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, err := inlineScriptResolveElement(store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		value, err := store.InnerHTMLForNode(nodeID)
		if err != nil {
			return script.UndefinedValue(), err
		}
		return script.StringValue(value), nil

	case "outerHTML":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, err := inlineScriptResolveElement(store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		value, err := store.OuterHTMLForNode(nodeID)
		if err != nil {
			return script.UndefinedValue(), err
		}
		return script.StringValue(value), nil

	case "textContent":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, err := inlineScriptResolveElement(store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		value := store.TextContentForNode(nodeID)
		return script.StringValue(value), nil

	case "setInnerHTML":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		markup, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, err := inlineScriptResolveElement(store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if err := store.SetInnerHTML(nodeID, markup); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "setOuterHTML":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		markup, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, err := inlineScriptResolveElement(store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if err := store.SetOuterHTML(nodeID, markup); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "setTextContent":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		text, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, err := inlineScriptResolveElement(store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if err := store.SetTextContent(nodeID, text); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "replaceChildren":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		markup, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, err := inlineScriptResolveElement(store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if err := store.ReplaceChildren(nodeID, markup); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "insertAdjacentHTML":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		position, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		markup, err := scriptStringArg(method, args, 2)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, err := inlineScriptResolveElement(store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if err := store.InsertAdjacentHTML(nodeID, position, markup); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "writeHTML":
		markup, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.WriteHTML(markup); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "locationAssign":
		url, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 1 {
			return script.UndefinedValue(), fmt.Errorf("locationAssign accepts at most 1 argument")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.AssignLocation(url); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "locationReplace":
		url, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 1 {
			return script.UndefinedValue(), fmt.Errorf("locationReplace accepts at most 1 argument")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.ReplaceLocation(url); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "locationReload":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("locationReload accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.ReloadLocation(); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "locationSet":
		property, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		value, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 2 {
			return script.UndefinedValue(), fmt.Errorf("locationSet accepts at most 2 arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.SetLocationProperty(property, value); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "locationHref":
		return h.locationString(method, args, (*Session).LocationHref)

	case "locationOrigin":
		return h.locationString(method, args, (*Session).LocationOrigin)

	case "locationProtocol":
		return h.locationString(method, args, (*Session).LocationProtocol)

	case "locationHost":
		return h.locationString(method, args, (*Session).LocationHost)

	case "locationHostname":
		return h.locationString(method, args, (*Session).LocationHostname)

	case "locationPort":
		return h.locationString(method, args, (*Session).LocationPort)

	case "locationPathname":
		return h.locationString(method, args, (*Session).LocationPathname)

	case "locationSearch":
		return h.locationString(method, args, (*Session).LocationSearch)

	case "locationHash":
		return h.locationString(method, args, (*Session).LocationHash)

	case "historyLength":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("history.length accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		return script.NumberValue(float64(h.session.windowHistoryLength())), nil

	case "historyState":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("history.state accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		state, ok := h.session.windowHistoryState()
		if !ok {
			return script.StringValue("null"), nil
		}
		return script.StringValue(state), nil

	case "historyScrollRestoration":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("history.scrollRestoration accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		return script.StringValue(h.session.windowHistoryScrollRestoration()), nil

	case "historySetScrollRestoration":
		value, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 1 {
			return script.UndefinedValue(), fmt.Errorf("history.scrollRestoration accepts at most 1 argument")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.setWindowHistoryScrollRestoration(value); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "historyPushState":
		if len(args) < 2 || len(args) > 3 {
			return script.UndefinedValue(), fmt.Errorf("history.pushState() expects 2 or 3 arguments")
		}
		state, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		title, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		url := ""
		if len(args) == 3 {
			url, err = scriptStringArg(method, args, 2)
			if err != nil {
				return script.UndefinedValue(), err
			}
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.windowHistoryPushState(state, title, url); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "historyReplaceState":
		if len(args) < 2 || len(args) > 3 {
			return script.UndefinedValue(), fmt.Errorf("history.replaceState() expects 2 or 3 arguments")
		}
		state, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		title, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		url := ""
		if len(args) == 3 {
			url, err = scriptStringArg(method, args, 2)
			if err != nil {
				return script.UndefinedValue(), err
			}
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.windowHistoryReplaceState(state, title, url); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "historyBack":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("history.back() expects no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.windowHistoryBack(); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "historyForward":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("history.forward() expects no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.windowHistoryForward(); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "historyGo":
		delta := int64(0)
		if len(args) > 0 {
			if len(args) > 1 {
				return script.UndefinedValue(), fmt.Errorf("history.go() accepts at most 1 argument")
			}
			var err error
			delta, err = scriptInt64Arg(method, args, 0)
			if err != nil {
				return script.UndefinedValue(), err
			}
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.windowHistoryGo(delta); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "documentCookie":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("document.cookie accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		return script.StringValue(h.session.documentCookie()), nil

	case "documentCurrentScript":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("document.currentScript accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		return script.StringValue(h.session.documentCurrentScript()), nil

	case "setDocumentCookie":
		value, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 1 {
			return script.UndefinedValue(), fmt.Errorf("document.cookie accepts at most 1 argument")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.setDocumentCookie(value); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "navigatorCookieEnabled":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("navigator.cookieEnabled accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		return script.BoolValue(h.session.navigatorCookieEnabled()), nil

	case "localStorageGetItem":
		key, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 1 {
			return script.UndefinedValue(), fmt.Errorf("localStorageGetItem accepts at most 1 argument")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		value, ok := h.session.localStorageGetItem(key)
		if !ok {
			return script.StringValue("null"), nil
		}
		return script.StringValue(value), nil

	case "localStorageSetItem":
		key, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		value, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 2 {
			return script.UndefinedValue(), fmt.Errorf("localStorageSetItem accepts at most 2 arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.localStorageSetItem(key, value); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "localStorageRemoveItem":
		key, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 1 {
			return script.UndefinedValue(), fmt.Errorf("localStorageRemoveItem accepts at most 1 argument")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.localStorageRemoveItem(key); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "localStorageClear":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("localStorageClear accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.localStorageClear(); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "localStorageLength":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("localStorageLength accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		return script.NumberValue(float64(h.session.localStorageLength())), nil

	case "localStorageKey":
		index, err := scriptInt64Arg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 1 {
			return script.UndefinedValue(), fmt.Errorf("localStorageKey accepts at most 1 argument")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		key, ok := h.session.localStorageKey(int(index))
		if !ok {
			return script.StringValue("null"), nil
		}
		return script.StringValue(key), nil

	case "sessionStorageGetItem":
		key, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 1 {
			return script.UndefinedValue(), fmt.Errorf("sessionStorageGetItem accepts at most 1 argument")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		value, ok := h.session.sessionStorageGetItem(key)
		if !ok {
			return script.StringValue("null"), nil
		}
		return script.StringValue(value), nil

	case "sessionStorageSetItem":
		key, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		value, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 2 {
			return script.UndefinedValue(), fmt.Errorf("sessionStorageSetItem accepts at most 2 arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.sessionStorageSetItem(key, value); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "sessionStorageRemoveItem":
		key, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 1 {
			return script.UndefinedValue(), fmt.Errorf("sessionStorageRemoveItem accepts at most 1 argument")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.sessionStorageRemoveItem(key); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "sessionStorageClear":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("sessionStorageClear accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.sessionStorageClear(); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "sessionStorageLength":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("sessionStorageLength accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		return script.NumberValue(float64(h.session.sessionStorageLength())), nil

	case "sessionStorageKey":
		index, err := scriptInt64Arg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 1 {
			return script.UndefinedValue(), fmt.Errorf("sessionStorageKey accepts at most 1 argument")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		key, ok := h.session.sessionStorageKey(int(index))
		if !ok {
			return script.StringValue("null"), nil
		}
		return script.StringValue(key), nil

	case "windowName":
		if len(args) > 0 {
			return script.UndefinedValue(), fmt.Errorf("window.name accepts no arguments")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		return script.StringValue(h.session.WindowName()), nil

	case "setWindowName":
		value, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if len(args) > 1 {
			return script.UndefinedValue(), fmt.Errorf("window.name accepts at most 1 argument")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.setWindowName(value); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "addEventListener":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		eventType, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		source, err := scriptStringArg(method, args, 2)
		if err != nil {
			return script.UndefinedValue(), err
		}
		phase := string(eventPhaseTarget)
		once := false
		if len(args) > 3 {
			if len(args) > 5 {
				return script.UndefinedValue(), fmt.Errorf("addEventListener accepts at most 5 arguments")
			}
			phase, err = scriptStringArg(method, args, 3)
			if err != nil {
				return script.UndefinedValue(), err
			}
		}
		if len(args) > 4 {
			once, err = scriptBoolArg(method, args, 4)
			if err != nil {
				return script.UndefinedValue(), err
			}
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		nodeID, err := inlineScriptResolveElement(store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if err := h.session.registerEventListener(nodeID, eventType, source, phase, once); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "removeEventListener":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		eventType, err := scriptStringArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		source, err := scriptStringArg(method, args, 2)
		if err != nil {
			return script.UndefinedValue(), err
		}
		phase := string(eventPhaseTarget)
		if len(args) > 3 {
			if len(args) > 4 {
				return script.UndefinedValue(), fmt.Errorf("removeEventListener accepts at most 4 arguments")
			}
			phase, err = scriptStringArg(method, args, 3)
			if err != nil {
				return script.UndefinedValue(), err
			}
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		nodeID, err := inlineScriptResolveElement(store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if _, err := h.session.removeEventListener(nodeID, eventType, source, phase); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "queueMicrotask":
		source, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if strings.TrimSpace(source) == "" {
			return script.UndefinedValue(), fmt.Errorf("microtask source must not be empty")
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		h.session.enqueueMicrotask(source)
		return script.UndefinedValue(), nil

	case "setTimeout":
		source, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		timeoutMs := int64(0)
		if len(args) > 1 {
			if len(args) > 2 {
				return script.UndefinedValue(), fmt.Errorf("setTimeout accepts at most 2 arguments")
			}
			timeoutMs, err = scriptInt64Arg(method, args, 1)
			if err != nil {
				return script.UndefinedValue(), err
			}
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		id, err := h.session.scheduleTimeout(source, timeoutMs)
		if err != nil {
			return script.UndefinedValue(), err
		}
		return script.NumberValue(float64(id)), nil

	case "setInterval":
		source, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		timeoutMs := int64(0)
		if len(args) > 1 {
			if len(args) > 2 {
				return script.UndefinedValue(), fmt.Errorf("setInterval accepts at most 2 arguments")
			}
			timeoutMs, err = scriptInt64Arg(method, args, 1)
			if err != nil {
				return script.UndefinedValue(), err
			}
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		id, err := h.session.scheduleInterval(source, timeoutMs)
		if err != nil {
			return script.UndefinedValue(), err
		}
		return script.NumberValue(float64(id)), nil

	case "requestAnimationFrame":
		source, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		id, err := h.session.requestAnimationFrame(source)
		if err != nil {
			return script.UndefinedValue(), err
		}
		return script.NumberValue(float64(id)), nil

	case "clearTimeout":
		timerID := int64(0)
		if len(args) > 0 {
			if len(args) > 1 {
				return script.UndefinedValue(), fmt.Errorf("clearTimeout accepts at most 1 argument")
			}
			var err error
			timerID, err = scriptInt64Arg(method, args, 0)
			if err != nil {
				return script.UndefinedValue(), err
			}
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		h.session.clearTimeout(timerID)
		return script.UndefinedValue(), nil

	case "clearInterval":
		timerID := int64(0)
		if len(args) > 0 {
			if len(args) > 1 {
				return script.UndefinedValue(), fmt.Errorf("clearInterval accepts at most 1 argument")
			}
			var err error
			timerID, err = scriptInt64Arg(method, args, 0)
			if err != nil {
				return script.UndefinedValue(), err
			}
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		h.session.clearInterval(timerID)
		return script.UndefinedValue(), nil

	case "cancelAnimationFrame":
		frameID := int64(0)
		if len(args) > 0 {
			if len(args) > 1 {
				return script.UndefinedValue(), fmt.Errorf("cancelAnimationFrame accepts at most 1 argument")
			}
			var err error
			frameID, err = scriptInt64Arg(method, args, 0)
			if err != nil {
				return script.UndefinedValue(), err
			}
		}
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		h.session.cancelAnimationFrame(frameID)
		return script.UndefinedValue(), nil

	case "preventDefault":
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.preventDefault(); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "stopPropagation":
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		if err := h.session.stopPropagation(); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "removeNode":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, err := inlineScriptResolveElement(h.store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if err := h.store.RemoveNode(nodeID); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	case "cloneNode":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		deep, err := scriptBoolArg(method, args, 1)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, err := inlineScriptResolveElement(h.store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if _, err := h.store.CloneNodeAfter(nodeID, deep); err != nil {
			return script.UndefinedValue(), err
		}
		return script.UndefinedValue(), nil

	default:
		return script.UndefinedValue(), fmt.Errorf("unsupported host method %q", method)
	}
}

func (h *inlineScriptHost) locationString(method string, args []script.Value, getter func(*Session) (string, error)) (script.Value, error) {
	if len(args) > 0 {
		return script.UndefinedValue(), fmt.Errorf("%s accepts no arguments", method)
	}
	if h.session == nil {
		return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
	}
	value, err := getter(h.session)
	if err != nil {
		return script.UndefinedValue(), err
	}
	return script.StringValue(value), nil
}

func inlineScriptResolveElement(store *dom.Store, selector string) (dom.NodeID, error) {
	if store == nil {
		return 0, fmt.Errorf("inline script DOM store is unavailable")
	}
	normalized := strings.TrimSpace(selector)
	if normalized == "" {
		return 0, fmt.Errorf("selector must not be empty")
	}
	nodeID, ok, err := store.QuerySelector(normalized)
	if err != nil {
		return 0, err
	}
	if !ok {
		return 0, fmt.Errorf("selector `%s` did not match any element", normalized)
	}
	return nodeID, nil
}

func scriptStringArg(method string, args []script.Value, index int) (string, error) {
	if index >= len(args) {
		return "", fmt.Errorf("%s requires argument %d", method, index+1)
	}
	if args[index].Kind != script.ValueKindString {
		return "", fmt.Errorf("%s argument %d must be a string", method, index+1)
	}
	return args[index].String, nil
}

func scriptNodeIDArg(method string, args []script.Value, index int) (dom.NodeID, error) {
	value, err := scriptInt64Arg(method, args, index)
	return dom.NodeID(value), err
}

func scriptBoolArg(method string, args []script.Value, index int) (bool, error) {
	if index >= len(args) {
		return false, fmt.Errorf("%s requires argument %d", method, index+1)
	}
	if args[index].Kind != script.ValueKindBool {
		return false, fmt.Errorf("%s argument %d must be a boolean", method, index+1)
	}
	return args[index].Bool, nil
}

func scriptInt64Arg(method string, args []script.Value, index int) (int64, error) {
	if index >= len(args) {
		return 0, fmt.Errorf("%s requires argument %d", method, index+1)
	}
	if args[index].Kind != script.ValueKindNumber {
		return 0, fmt.Errorf("%s argument %d must be a number", method, index+1)
	}
	return int64(args[index].Number), nil
}
