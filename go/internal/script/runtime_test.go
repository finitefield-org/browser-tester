package script

import (
	"errors"
	"fmt"
	"testing"

	"browsertester/internal/dom"
)

type hostCall struct {
	method string
	args   []Value
}

type fakeHost struct {
	values map[string]Value
	errs   map[string]error
	calls  []hostCall
}

func (h *fakeHost) Call(method string, args []Value) (Value, error) {
	copiedArgs := make([]Value, len(args))
	copy(copiedArgs, args)
	h.calls = append(h.calls, hostCall{method: method, args: copiedArgs})
	if err, ok := h.errs[method]; ok {
		return UndefinedValue(), err
	}
	if value, ok := h.values[method]; ok {
		return value, nil
	}
	return UndefinedValue(), errors.New("host method is not configured")
}

func TestNewRuntimeUsesDefaultConfig(t *testing.T) {
	runtime := NewRuntime(nil)
	if runtime == nil {
		t.Fatalf("NewRuntime() = nil")
	}

	got := runtime.Config()
	want := DefaultRuntimeConfig()
	if got.StepLimit != want.StepLimit {
		t.Fatalf("Config().StepLimit = %d, want %d", got.StepLimit, want.StepLimit)
	}
}

func TestNewRuntimeWithConfigNormalizesStepLimit(t *testing.T) {
	runtime := NewRuntimeWithConfig(RuntimeConfig{StepLimit: 0}, nil)
	if runtime == nil {
		t.Fatalf("NewRuntimeWithConfig() = nil")
	}

	if got, want := runtime.Config().StepLimit, DefaultRuntimeConfig().StepLimit; got != want {
		t.Fatalf("Config().StepLimit = %d, want %d", got, want)
	}
}

func TestDispatchSupportsNoopAndHostCall(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"version": StringValue("v1"),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	result, err := runtime.Dispatch(DispatchRequest{Source: "noop"})
	if err != nil {
		t.Fatalf("Dispatch(noop) error = %v", err)
	}
	if result.Value.Kind != ValueKindUndefined {
		t.Fatalf("Dispatch(noop) kind = %q, want %q", result.Value.Kind, ValueKindUndefined)
	}

	result, err = runtime.Dispatch(DispatchRequest{Source: "host:version"})
	if err != nil {
		t.Fatalf("Dispatch(host:version) error = %v", err)
	}
	if result.Value.Kind != ValueKindString || result.Value.String != "v1" {
		t.Fatalf("Dispatch(host:version) value = %#v, want string v1", result.Value)
	}
	if len(host.calls) != 1 || host.calls[0].method != "version" {
		t.Fatalf("host calls = %#v, want [\"version\"]", host.calls)
	}
}

func TestDispatchParsesHostArguments(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"echo": StringValue("ok"),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:echo("div > section > p.primary", true, 2)`})
	if err != nil {
		t.Fatalf("Dispatch(host:echo(...)) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if call.method != "echo" {
		t.Fatalf("host call method = %q, want echo", call.method)
	}
	if len(call.args) != 3 {
		t.Fatalf("host call args len = %d, want 3", len(call.args))
	}
	if call.args[0].Kind != ValueKindString || call.args[0].String != "div > section > p.primary" {
		t.Fatalf("host call arg[0] = %#v, want selector string", call.args[0])
	}
	if call.args[1].Kind != ValueKindBool || !call.args[1].Bool {
		t.Fatalf("host call arg[1] = %#v, want true", call.args[1])
	}
	if call.args[2].Kind != ValueKindNumber || call.args[2].Number != 2 {
		t.Fatalf("host call arg[2] = %#v, want 2", call.args[2])
	}
}

func TestDispatchParsesQuotedEventListenerSource(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"addEventListener": UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:addEventListener("#btn", "click", 'host:setInnerHTML("#out", "clicked")')`})
	if err != nil {
		t.Fatalf("Dispatch(addEventListener) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if call.method != "addEventListener" {
		t.Fatalf("host call method = %q, want addEventListener", call.method)
	}
	if len(call.args) != 3 {
		t.Fatalf("host call args len = %d, want 3", len(call.args))
	}
	if call.args[2].Kind != ValueKindString || call.args[2].String != `host:setInnerHTML("#out", "clicked")` {
		t.Fatalf("host call arg[2] = %#v, want quoted source string", call.args[2])
	}
}

func TestDispatchSupportsMultipleHostStatements(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"setInnerHTML": UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:setInnerHTML("#out", "first"); host:setInnerHTML("#out", "second")`})
	if err != nil {
		t.Fatalf("Dispatch(multiple host statements) error = %v", err)
	}
	if len(host.calls) != 2 {
		t.Fatalf("host calls = %#v, want two calls", host.calls)
	}
	if host.calls[0].method != "setInnerHTML" || host.calls[1].method != "setInnerHTML" {
		t.Fatalf("host call methods = %#v, want setInnerHTML twice", host.calls)
	}
	if host.calls[0].args[1].Kind != ValueKindString || host.calls[0].args[1].String != "first" {
		t.Fatalf("host call[0] arg[1] = %#v, want first", host.calls[0].args[1])
	}
	if host.calls[1].args[1].Kind != ValueKindString || host.calls[1].args[1].String != "second" {
		t.Fatalf("host call[1] arg[1] = %#v, want second", host.calls[1].args[1])
	}
}

type domQueryHost struct {
	store *dom.Store
}

func (h *domQueryHost) Call(method string, args []Value) (Value, error) {
	if h == nil || h.store == nil {
		return UndefinedValue(), fmt.Errorf("dom query host is unavailable")
	}
	switch method {
	case "querySelector":
		if len(args) != 1 || args[0].Kind != ValueKindString {
			return UndefinedValue(), fmt.Errorf("querySelector requires one selector string")
		}
		nodeID, ok, err := h.store.QuerySelector(args[0].String)
		if err != nil {
			return UndefinedValue(), err
		}
		if !ok {
			return UndefinedValue(), nil
		}
		return StringValue(fmt.Sprintf("%d", nodeID)), nil
	case "querySelectorAll":
		if len(args) != 1 || args[0].Kind != ValueKindString {
			return UndefinedValue(), fmt.Errorf("querySelectorAll requires one selector string")
		}
		nodes, err := h.store.QuerySelectorAll(args[0].String)
		if err != nil {
			return UndefinedValue(), err
		}
		return NumberValue(float64(nodes.Length())), nil
	case "matches":
		if len(args) != 2 || args[0].Kind != ValueKindNumber || args[1].Kind != ValueKindString {
			return UndefinedValue(), fmt.Errorf("matches requires a node id and selector string")
		}
		matched, err := h.store.Matches(dom.NodeID(args[0].Number), args[1].String)
		if err != nil {
			return UndefinedValue(), err
		}
		return BoolValue(matched), nil
	case "closest":
		if len(args) != 2 || args[0].Kind != ValueKindNumber || args[1].Kind != ValueKindString {
			return UndefinedValue(), fmt.Errorf("closest requires a node id and selector string")
		}
		nodeID, ok, err := h.store.Closest(dom.NodeID(args[0].Number), args[1].String)
		if err != nil {
			return UndefinedValue(), err
		}
		if !ok {
			return UndefinedValue(), nil
		}
		return StringValue(fmt.Sprintf("%d", nodeID)), nil
	default:
		return UndefinedValue(), fmt.Errorf("host method is not configured")
	}
}

func TestDispatchSupportsDOMQueryHostCalls(t *testing.T) {
	store := dom.NewStore()
	if err := store.BootstrapHTML(`<main><section><p id="first">one</p></section><p id="second">two</p></main>`); err != nil {
		t.Fatalf("BootstrapHTML() error = %v", err)
	}

	firstID, ok, err := store.QuerySelector("#first")
	if err != nil {
		t.Fatalf("QuerySelector(#first) error = %v", err)
	}
	if !ok {
		t.Fatalf("QuerySelector(#first) ok = false, want true")
	}
	sectionID, ok, err := store.QuerySelector("section")
	if err != nil {
		t.Fatalf("QuerySelector(section) error = %v", err)
	}
	if !ok {
		t.Fatalf("QuerySelector(section) ok = false, want true")
	}

	runtime := NewRuntime(&domQueryHost{store: store})

	result, err := runtime.Dispatch(DispatchRequest{Source: `host:querySelector("#first")`})
	if err != nil {
		t.Fatalf("Dispatch(querySelector) error = %v", err)
	}
	if result.Value.Kind != ValueKindString || result.Value.String != fmt.Sprintf("%d", firstID) {
		t.Fatalf("Dispatch(querySelector) value = %#v, want node id string", result.Value)
	}

	result, err = runtime.Dispatch(DispatchRequest{Source: `host:querySelectorAll("main p")`})
	if err != nil {
		t.Fatalf("Dispatch(querySelectorAll) error = %v", err)
	}
	if result.Value.Kind != ValueKindNumber || result.Value.Number != 2 {
		t.Fatalf("Dispatch(querySelectorAll) value = %#v, want count 2", result.Value)
	}

	result, err = runtime.Dispatch(DispatchRequest{Source: fmt.Sprintf(`host:matches(%d, "main > section > p")`, firstID)})
	if err != nil {
		t.Fatalf("Dispatch(matches) error = %v", err)
	}
	if result.Value.Kind != ValueKindBool || !result.Value.Bool {
		t.Fatalf("Dispatch(matches) value = %#v, want true", result.Value)
	}

	result, err = runtime.Dispatch(DispatchRequest{Source: fmt.Sprintf(`host:closest(%d, "main > section")`, firstID)})
	if err != nil {
		t.Fatalf("Dispatch(closest) error = %v", err)
	}
	if result.Value.Kind != ValueKindString || result.Value.String != fmt.Sprintf("%d", sectionID) {
		t.Fatalf("Dispatch(closest) value = %#v, want section node id string", result.Value)
	}
}

func TestDispatchReturnsUnsupportedForUnknownSource(t *testing.T) {
	runtime := NewRuntime(nil)
	_, err := runtime.Dispatch(DispatchRequest{Source: "let a = 1"})
	if err == nil {
		t.Fatalf("Dispatch() error = nil, want unsupported error")
	}

	scriptErr, ok := err.(Error)
	if !ok {
		t.Fatalf("Dispatch() error type = %T, want script.Error", err)
	}
	if scriptErr.Kind != ErrorKindUnsupported {
		t.Fatalf("Dispatch() error kind = %q, want %q", scriptErr.Kind, ErrorKindUnsupported)
	}
}

func TestDispatchReturnsHostErrorsExplicitly(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{},
		errs: map[string]error{
			"boom": errors.New("host failed"),
		},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: "host:boom"})
	if err == nil {
		t.Fatalf("Dispatch() error = nil, want host error")
	}

	scriptErr, ok := err.(Error)
	if !ok {
		t.Fatalf("Dispatch() error type = %T, want script.Error", err)
	}
	if scriptErr.Kind != ErrorKindHost {
		t.Fatalf("Dispatch() error kind = %q, want %q", scriptErr.Kind, ErrorKindHost)
	}
}

func TestDispatchIsNilSafe(t *testing.T) {
	var runtime *Runtime

	if got, want := runtime.Config().StepLimit, DefaultRuntimeConfig().StepLimit; got != want {
		t.Fatalf("nil Config().StepLimit = %d, want %d", got, want)
	}

	_, err := runtime.Dispatch(DispatchRequest{Source: "noop"})
	if err == nil {
		t.Fatalf("nil Dispatch() error = nil, want runtime unavailable error")
	}

	scriptErr, ok := err.(Error)
	if !ok {
		t.Fatalf("nil Dispatch() error type = %T, want script.Error", err)
	}
	if scriptErr.Kind != ErrorKindRuntime {
		t.Fatalf("nil Dispatch() error kind = %q, want %q", scriptErr.Kind, ErrorKindRuntime)
	}
}

func TestDispatchValidatesHostMethodName(t *testing.T) {
	runtime := NewRuntime(&fakeHost{
		values: map[string]Value{},
		errs:   map[string]error{},
	})

	_, err := runtime.Dispatch(DispatchRequest{Source: "host:   "})
	if err == nil {
		t.Fatalf("Dispatch(host:<blank>) error = nil, want parse error")
	}

	scriptErr, ok := err.(Error)
	if !ok {
		t.Fatalf("Dispatch(host:<blank>) error type = %T, want script.Error", err)
	}
	if scriptErr.Kind != ErrorKindParse {
		t.Fatalf("Dispatch(host:<blank>) error kind = %q, want %q", scriptErr.Kind, ErrorKindParse)
	}
}
