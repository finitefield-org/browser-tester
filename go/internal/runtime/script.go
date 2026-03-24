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

func (s *Session) executeInlineScripts(store *dom.Store) error {
	if s == nil || store == nil {
		return nil
	}
	nodes := store.Nodes()
	for _, node := range nodes {
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
		if _, err := s.runScriptOnStore(store, source); err != nil {
			return err
		}
	}
	return nil
}

type inlineScriptHost struct {
	session *Session
	store   *dom.Store
}

func (h *inlineScriptHost) Call(method string, args []script.Value) (script.Value, error) {
	if h == nil || h.store == nil {
		return script.UndefinedValue(), fmt.Errorf("inline script host is unavailable")
	}

	switch method {
	case "querySelector":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, ok, err := h.store.QuerySelector(selector)
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
		nodes, err := h.store.QuerySelectorAll(selector)
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
		matched, err := h.store.Matches(nodeID, selector)
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
		closestID, ok, err := h.store.Closest(nodeID, selector)
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
		nodeID, err := inlineScriptResolveElement(h.store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		value, err := h.store.InnerHTMLForNode(nodeID)
		if err != nil {
			return script.UndefinedValue(), err
		}
		return script.StringValue(value), nil

	case "outerHTML":
		selector, err := scriptStringArg(method, args, 0)
		if err != nil {
			return script.UndefinedValue(), err
		}
		nodeID, err := inlineScriptResolveElement(h.store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		value, err := h.store.OuterHTMLForNode(nodeID)
		if err != nil {
			return script.UndefinedValue(), err
		}
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
		nodeID, err := inlineScriptResolveElement(h.store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if err := h.store.SetInnerHTML(nodeID, markup); err != nil {
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
		nodeID, err := inlineScriptResolveElement(h.store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if err := h.store.SetOuterHTML(nodeID, markup); err != nil {
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
		nodeID, err := inlineScriptResolveElement(h.store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if err := h.store.InsertAdjacentHTML(nodeID, position, markup); err != nil {
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
		if h.session == nil {
			return script.UndefinedValue(), fmt.Errorf("inline script session is unavailable")
		}
		nodeID, err := inlineScriptResolveElement(h.store, selector)
		if err != nil {
			return script.UndefinedValue(), err
		}
		if err := h.session.registerEventListener(nodeID, eventType, source); err != nil {
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

	default:
		return script.UndefinedValue(), fmt.Errorf("unsupported host method %q", method)
	}
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
	if index >= len(args) {
		return 0, fmt.Errorf("%s requires argument %d", method, index+1)
	}
	if args[index].Kind != script.ValueKindNumber {
		return 0, fmt.Errorf("%s argument %d must be a number", method, index+1)
	}
	return dom.NodeID(args[index].Number), nil
}
