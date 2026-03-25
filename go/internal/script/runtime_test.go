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

func TestParseHostInvocationNestedArguments(t *testing.T) {
	method, args, err := parseHostInvocation(`setInnerHTML("#out", expr(host:documentCurrentScript()))`)
	if err != nil {
		t.Fatalf("parseHostInvocation() error = %v", err)
	}
	if method != "setInnerHTML" {
		t.Fatalf("method = %q, want setInnerHTML", method)
	}
	if len(args) != 2 {
		t.Fatalf("args len = %d, want 2", len(args))
	}
	if args[1].Kind != ValueKindInvocation {
		t.Fatalf("args[1].Kind = %q, want %q", args[1].Kind, ValueKindInvocation)
	}
	if args[1].Invocation != "host:documentCurrentScript()" {
		t.Fatalf("args[1].Invocation = %q, want host:documentCurrentScript()", args[1].Invocation)
	}
}

func TestDispatchParsesTextContentMutationInvocation(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"setTextContent": UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:setTextContent("#out", "clicked")`})
	if err != nil {
		t.Fatalf("Dispatch(setTextContent) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if call.method != "setTextContent" {
		t.Fatalf("host call method = %q, want setTextContent", call.method)
	}
	if len(call.args) != 2 {
		t.Fatalf("host call args len = %d, want 2", len(call.args))
	}
	if call.args[0].Kind != ValueKindString || call.args[0].String != "#out" {
		t.Fatalf("host call arg[0] = %#v, want selector string", call.args[0])
	}
	if call.args[1].Kind != ValueKindString || call.args[1].String != "clicked" {
		t.Fatalf("host call arg[1] = %#v, want clicked", call.args[1])
	}
}

func TestDispatchParsesTextContentGetterInvocation(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"textContent": StringValue("seed text"),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	result, err := runtime.Dispatch(DispatchRequest{Source: `host:textContent("#src")`})
	if err != nil {
		t.Fatalf("Dispatch(textContent) error = %v", err)
	}
	if result.Value.Kind != ValueKindString || result.Value.String != "seed text" {
		t.Fatalf("Dispatch(textContent) value = %#v, want seed text", result.Value)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	if host.calls[0].method != "textContent" {
		t.Fatalf("host call method = %q, want textContent", host.calls[0].method)
	}
	if len(host.calls[0].args) != 1 || host.calls[0].args[0].Kind != ValueKindString || host.calls[0].args[0].String != "#src" {
		t.Fatalf("host call args = %#v, want selector argument", host.calls[0].args)
	}
}

func TestDispatchParsesTreeMutationInvocations(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"replaceChildren": UndefinedValue(),
			"cloneNode":       UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:replaceChildren("#out", "<span>fresh</span>"); host:cloneNode("#src", true)`})
	if err != nil {
		t.Fatalf("Dispatch(tree mutations) error = %v", err)
	}
	if len(host.calls) != 2 {
		t.Fatalf("host calls = %#v, want two calls", host.calls)
	}
	if host.calls[0].method != "replaceChildren" {
		t.Fatalf("host call[0].method = %q, want replaceChildren", host.calls[0].method)
	}
	if len(host.calls[0].args) != 2 || host.calls[0].args[0].String != "#out" || host.calls[0].args[1].String != "<span>fresh</span>" {
		t.Fatalf("host call[0].args = %#v, want selector + markup", host.calls[0].args)
	}
	if host.calls[1].method != "cloneNode" {
		t.Fatalf("host call[1].method = %q, want cloneNode", host.calls[1].method)
	}
	if len(host.calls[1].args) != 2 || host.calls[1].args[0].String != "#src" || host.calls[1].args[1].Kind != ValueKindBool || !host.calls[1].args[1].Bool {
		t.Fatalf("host call[1].args = %#v, want selector + true", host.calls[1].args)
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

func TestDispatchParsesEventListenerPhaseArgument(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"addEventListener": UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:addEventListener("#btn", "click", 'host:setInnerHTML("#out", "clicked")', "capture")`})
	if err != nil {
		t.Fatalf("Dispatch(addEventListener capture) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if len(call.args) != 4 {
		t.Fatalf("host call args len = %d, want 4", len(call.args))
	}
	if call.args[3].Kind != ValueKindString || call.args[3].String != "capture" {
		t.Fatalf("host call arg[3] = %#v, want capture", call.args[3])
	}
}

func TestDispatchParsesEventListenerOnceArgument(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"addEventListener": UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:addEventListener("#btn", "click", 'host:setInnerHTML("#out", "clicked")', "capture", true)`})
	if err != nil {
		t.Fatalf("Dispatch(addEventListener once) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if len(call.args) != 5 {
		t.Fatalf("host call args len = %d, want 5", len(call.args))
	}
	if call.args[4].Kind != ValueKindBool || !call.args[4].Bool {
		t.Fatalf("host call arg[4] = %#v, want true", call.args[4])
	}
}

func TestDispatchParsesEventListenerRemoval(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"removeEventListener": UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:removeEventListener("#btn", "click", 'host:removeNode("#btn")', "capture")`})
	if err != nil {
		t.Fatalf("Dispatch(removeEventListener) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if len(call.args) != 4 {
		t.Fatalf("host call args len = %d, want 4", len(call.args))
	}
	if call.args[2].Kind != ValueKindString || call.args[2].String != `host:removeNode("#btn")` {
		t.Fatalf("host call arg[2] = %#v, want raw source string", call.args[2])
	}
	if call.args[3].Kind != ValueKindString || call.args[3].String != "capture" {
		t.Fatalf("host call arg[3] = %#v, want capture", call.args[3])
	}
}

func TestDispatchParsesQueueMicrotaskInvocation(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"queueMicrotask": UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:queueMicrotask('host:setInnerHTML(#out, micro)')`})
	if err != nil {
		t.Fatalf("Dispatch(queueMicrotask) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if len(call.args) != 1 {
		t.Fatalf("host call args len = %d, want 1", len(call.args))
	}
	if call.args[0].Kind != ValueKindString || call.args[0].String != `host:setInnerHTML(#out, micro)` {
		t.Fatalf("host call arg[0] = %#v, want raw source string", call.args[0])
	}
}

func TestDispatchParsesSetTimeoutInvocation(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"setTimeout": NumberValue(1),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:setTimeout('host:setInnerHTML(#out, later)', 25)`})
	if err != nil {
		t.Fatalf("Dispatch(setTimeout) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if call.method != "setTimeout" {
		t.Fatalf("host call method = %q, want setTimeout", call.method)
	}
	if len(call.args) != 2 {
		t.Fatalf("host call args len = %d, want 2", len(call.args))
	}
	if call.args[0].Kind != ValueKindString || call.args[0].String != `host:setInnerHTML(#out, later)` {
		t.Fatalf("host call arg[0] = %#v, want raw source string", call.args[0])
	}
	if call.args[1].Kind != ValueKindNumber || call.args[1].Number != 25 {
		t.Fatalf("host call arg[1] = %#v, want 25", call.args[1])
	}
}

func TestDispatchParsesWriteHTMLInvocation(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"writeHTML": UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:writeHTML('<main><div data-x="a;b">ok</div></main>')`})
	if err != nil {
		t.Fatalf("Dispatch(writeHTML) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if call.method != "writeHTML" {
		t.Fatalf("host call method = %q, want writeHTML", call.method)
	}
	if len(call.args) != 1 {
		t.Fatalf("host call args len = %d, want 1", len(call.args))
	}
	if call.args[0].Kind != ValueKindString || call.args[0].String != `<main><div data-x="a;b">ok</div></main>` {
		t.Fatalf("host call arg[0] = %#v, want raw markup string", call.args[0])
	}
}

func TestDispatchParsesLocationNavigationInvocations(t *testing.T) {
	tests := []struct {
		name       string
		source     string
		method     string
		wantArgs   int
		wantString string
	}{
		{
			name:       "assign",
			source:     `host:locationAssign("https://example.test/assign")`,
			method:     "locationAssign",
			wantArgs:   1,
			wantString: "https://example.test/assign",
		},
		{
			name:       "replace",
			source:     `host:locationReplace("https://example.test/replace")`,
			method:     "locationReplace",
			wantArgs:   1,
			wantString: "https://example.test/replace",
		},
		{
			name:     "reload",
			source:   `host:locationReload()`,
			method:   "locationReload",
			wantArgs: 0,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			host := &fakeHost{
				values: map[string]Value{
					tc.method: UndefinedValue(),
				},
				errs: map[string]error{},
			}
			runtime := NewRuntime(host)

			_, err := runtime.Dispatch(DispatchRequest{Source: tc.source})
			if err != nil {
				t.Fatalf("Dispatch(%s) error = %v", tc.method, err)
			}
			if len(host.calls) != 1 {
				t.Fatalf("host calls = %#v, want one call", host.calls)
			}
			call := host.calls[0]
			if call.method != tc.method {
				t.Fatalf("host call method = %q, want %s", call.method, tc.method)
			}
			if len(call.args) != tc.wantArgs {
				t.Fatalf("host call args len = %d, want %d", len(call.args), tc.wantArgs)
			}
			if tc.wantArgs == 1 {
				if call.args[0].Kind != ValueKindString || call.args[0].String != tc.wantString {
					t.Fatalf("host call arg[0] = %#v, want %q", call.args[0], tc.wantString)
				}
			}
		})
	}
}

func TestDispatchParsesLocationGetterInvocations(t *testing.T) {
	tests := []struct {
		name       string
		source     string
		method     string
		wantString string
	}{
		{
			name:       "href",
			source:     `host:locationHref()`,
			method:     "locationHref",
			wantString: "https://example.test:8443/path/name?mode=full#step-1",
		},
		{
			name:       "origin",
			source:     `host:locationOrigin()`,
			method:     "locationOrigin",
			wantString: "https://example.test:8443",
		},
		{
			name:       "protocol",
			source:     `host:locationProtocol()`,
			method:     "locationProtocol",
			wantString: "https:",
		},
		{
			name:       "host",
			source:     `host:locationHost()`,
			method:     "locationHost",
			wantString: "example.test:8443",
		},
		{
			name:       "hostname",
			source:     `host:locationHostname()`,
			method:     "locationHostname",
			wantString: "example.test",
		},
		{
			name:       "port",
			source:     `host:locationPort()`,
			method:     "locationPort",
			wantString: "8443",
		},
		{
			name:       "pathname",
			source:     `host:locationPathname()`,
			method:     "locationPathname",
			wantString: "/path/name",
		},
		{
			name:       "search",
			source:     `host:locationSearch()`,
			method:     "locationSearch",
			wantString: "?mode=full",
		},
		{
			name:       "hash",
			source:     `host:locationHash()`,
			method:     "locationHash",
			wantString: "#step-1",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			host := &fakeHost{
				values: map[string]Value{
					tc.method: StringValue(tc.wantString),
				},
				errs: map[string]error{},
			}
			runtime := NewRuntime(host)

			result, err := runtime.Dispatch(DispatchRequest{Source: tc.source})
			if err != nil {
				t.Fatalf("Dispatch(%s) error = %v", tc.method, err)
			}
			if len(host.calls) != 1 {
				t.Fatalf("host calls = %#v, want one call", host.calls)
			}
			call := host.calls[0]
			if call.method != tc.method {
				t.Fatalf("host call method = %q, want %s", call.method, tc.method)
			}
			if len(call.args) != 0 {
				t.Fatalf("host call args len = %d, want 0", len(call.args))
			}
			if result.Value.Kind != ValueKindString || result.Value.String != tc.wantString {
				t.Fatalf("Dispatch(%s) value = %#v, want string %q", tc.method, result.Value, tc.wantString)
			}
		})
	}
}

func TestDispatchParsesLocationPropertyInvocation(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"locationSet": UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:locationSet("hash", "#next")`})
	if err != nil {
		t.Fatalf("Dispatch(locationSet) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if call.method != "locationSet" {
		t.Fatalf("host call method = %q, want locationSet", call.method)
	}
	if len(call.args) != 2 {
		t.Fatalf("host call args len = %d, want 2", len(call.args))
	}
	if call.args[0].Kind != ValueKindString || call.args[0].String != "hash" {
		t.Fatalf("host call arg[0] = %#v, want hash", call.args[0])
	}
	if call.args[1].Kind != ValueKindString || call.args[1].String != "#next" {
		t.Fatalf("host call arg[1] = %#v, want #next", call.args[1])
	}
}

func TestDispatchParsesHistoryNavigationInvocations(t *testing.T) {
	tests := []struct {
		name     string
		source   string
		method   string
		wantArgs int
		check    func(t *testing.T, call hostCall)
	}{
		{
			name:     "pushState",
			source:   `host:historyPushState("step-1", "", "https://example.test/step-1")`,
			method:   "historyPushState",
			wantArgs: 3,
			check: func(t *testing.T, call hostCall) {
				if call.args[0].Kind != ValueKindString || call.args[0].String != "step-1" {
					t.Fatalf("host call arg[0] = %#v, want step-1", call.args[0])
				}
				if call.args[1].Kind != ValueKindString || call.args[1].String != "" {
					t.Fatalf("host call arg[1] = %#v, want empty title", call.args[1])
				}
				if call.args[2].Kind != ValueKindString || call.args[2].String != "https://example.test/step-1" {
					t.Fatalf("host call arg[2] = %#v, want url", call.args[2])
				}
			},
		},
		{
			name:     "replaceState",
			source:   `host:historyReplaceState("step-2", "", "https://example.test/step-2")`,
			method:   "historyReplaceState",
			wantArgs: 3,
			check: func(t *testing.T, call hostCall) {
				if call.args[0].Kind != ValueKindString || call.args[0].String != "step-2" {
					t.Fatalf("host call arg[0] = %#v, want step-2", call.args[0])
				}
				if call.args[2].Kind != ValueKindString || call.args[2].String != "https://example.test/step-2" {
					t.Fatalf("host call arg[2] = %#v, want url", call.args[2])
				}
			},
		},
		{
			name:     "back",
			source:   `host:historyBack()`,
			method:   "historyBack",
			wantArgs: 0,
		},
		{
			name:     "forward",
			source:   `host:historyForward()`,
			method:   "historyForward",
			wantArgs: 0,
		},
		{
			name:     "go",
			source:   `host:historyGo(-1)`,
			method:   "historyGo",
			wantArgs: 1,
			check: func(t *testing.T, call hostCall) {
				if call.args[0].Kind != ValueKindNumber || call.args[0].Number != -1 {
					t.Fatalf("host call arg[0] = %#v, want -1", call.args[0])
				}
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			host := &fakeHost{
				values: map[string]Value{
					tc.method: UndefinedValue(),
				},
				errs: map[string]error{},
			}
			runtime := NewRuntime(host)

			_, err := runtime.Dispatch(DispatchRequest{Source: tc.source})
			if err != nil {
				t.Fatalf("Dispatch(%s) error = %v", tc.method, err)
			}
			if len(host.calls) != 1 {
				t.Fatalf("host calls = %#v, want one call", host.calls)
			}
			call := host.calls[0]
			if call.method != tc.method {
				t.Fatalf("host call method = %q, want %s", call.method, tc.method)
			}
			if len(call.args) != tc.wantArgs {
				t.Fatalf("host call args len = %d, want %d", len(call.args), tc.wantArgs)
			}
			if tc.check != nil {
				tc.check(t, call)
			}
		})
	}
}

func TestDispatchParsesHistoryAccessorsAndScrollRestoration(t *testing.T) {
	tests := []struct {
		name       string
		source     string
		method     string
		wantArgs   int
		wantKind   ValueKind
		wantString string
		wantNumber float64
	}{
		{
			name:       "length",
			source:     `host:historyLength()`,
			method:     "historyLength",
			wantArgs:   0,
			wantKind:   ValueKindNumber,
			wantNumber: 2,
		},
		{
			name:       "state",
			source:     `host:historyState()`,
			method:     "historyState",
			wantArgs:   0,
			wantKind:   ValueKindString,
			wantString: "current",
		},
		{
			name:       "scrollRestoration",
			source:     `host:historyScrollRestoration()`,
			method:     "historyScrollRestoration",
			wantArgs:   0,
			wantKind:   ValueKindString,
			wantString: "auto",
		},
		{
			name:     "setScrollRestoration",
			source:   `host:historySetScrollRestoration("manual")`,
			method:   "historySetScrollRestoration",
			wantArgs: 1,
			wantKind: ValueKindUndefined,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			host := &fakeHost{
				values: map[string]Value{
					tc.method: func() Value {
						switch tc.method {
						case "historyLength":
							return NumberValue(2)
						case "historyState":
							return StringValue(tc.wantString)
						case "historyScrollRestoration":
							return StringValue(tc.wantString)
						default:
							return UndefinedValue()
						}
					}(),
				},
				errs: map[string]error{},
			}
			runtime := NewRuntime(host)

			result, err := runtime.Dispatch(DispatchRequest{Source: tc.source})
			if err != nil {
				t.Fatalf("Dispatch(%s) error = %v", tc.method, err)
			}
			if len(host.calls) != 1 {
				t.Fatalf("host calls = %#v, want one call", host.calls)
			}
			call := host.calls[0]
			if call.method != tc.method {
				t.Fatalf("host call method = %q, want %s", call.method, tc.method)
			}
			if len(call.args) != tc.wantArgs {
				t.Fatalf("host call args len = %d, want %d", len(call.args), tc.wantArgs)
			}
			if result.Value.Kind != tc.wantKind {
				t.Fatalf("Dispatch(%s) kind = %q, want %q", tc.method, result.Value.Kind, tc.wantKind)
			}
			if tc.wantKind == ValueKindString && result.Value.String != tc.wantString {
				t.Fatalf("Dispatch(%s) value = %q, want %q", tc.method, result.Value.String, tc.wantString)
			}
			if tc.wantKind == ValueKindNumber && result.Value.Number != tc.wantNumber {
				t.Fatalf("Dispatch(%s) value = %v, want %v", tc.method, result.Value.Number, tc.wantNumber)
			}
		})
	}
}

func TestDispatchParsesDocumentCookieInvocations(t *testing.T) {
	tests := []struct {
		name     string
		source   string
		method   string
		wantArgs int
		check    func(t *testing.T, call hostCall, result Value)
	}{
		{
			name:     "getter",
			source:   `host:documentCookie()`,
			method:   "documentCookie",
			wantArgs: 0,
			check: func(t *testing.T, call hostCall, result Value) {
				if result.Kind != ValueKindString || result.String != "theme=light" {
					t.Fatalf("Dispatch(documentCookie) value = %#v, want string theme=light", result)
				}
			},
		},
		{
			name:     "setter",
			source:   `host:setDocumentCookie("theme=dark")`,
			method:   "setDocumentCookie",
			wantArgs: 1,
			check: func(t *testing.T, call hostCall, result Value) {
				if call.args[0].Kind != ValueKindString || call.args[0].String != "theme=dark" {
					t.Fatalf("host call arg[0] = %#v, want theme=dark", call.args[0])
				}
				if result.Kind != ValueKindUndefined {
					t.Fatalf("Dispatch(setDocumentCookie) value = %#v, want undefined", result)
				}
			},
		},
		{
			name:     "navigatorCookieEnabled",
			source:   `host:navigatorCookieEnabled()`,
			method:   "navigatorCookieEnabled",
			wantArgs: 0,
			check: func(t *testing.T, call hostCall, result Value) {
				if result.Kind != ValueKindBool || !result.Bool {
					t.Fatalf("Dispatch(navigatorCookieEnabled) value = %#v, want true", result)
				}
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			host := &fakeHost{
				values: map[string]Value{
					"documentCookie":         StringValue("theme=light"),
					"setDocumentCookie":      UndefinedValue(),
					"navigatorCookieEnabled": BoolValue(true),
				},
				errs: map[string]error{},
			}
			runtime := NewRuntime(host)

			result, err := runtime.Dispatch(DispatchRequest{Source: tc.source})
			if err != nil {
				t.Fatalf("Dispatch(%s) error = %v", tc.method, err)
			}
			if len(host.calls) != 1 {
				t.Fatalf("host calls = %#v, want one call", host.calls)
			}
			call := host.calls[0]
			if call.method != tc.method {
				t.Fatalf("host call method = %q, want %s", call.method, tc.method)
			}
			if len(call.args) != tc.wantArgs {
				t.Fatalf("host call args len = %d, want %d", len(call.args), tc.wantArgs)
			}
			if tc.check != nil {
				tc.check(t, call, result.Value)
			}
		})
	}
}

func TestDispatchParsesDocumentCurrentScriptInvocation(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"documentCurrentScript": StringValue(`<script id="boot">host:documentCurrentScript()</script>`),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	result, err := runtime.Dispatch(DispatchRequest{Source: `host:documentCurrentScript()`})
	if err != nil {
		t.Fatalf("Dispatch(documentCurrentScript) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if call.method != "documentCurrentScript" {
		t.Fatalf("host call method = %q, want documentCurrentScript", call.method)
	}
	if len(call.args) != 0 {
		t.Fatalf("host call args len = %d, want 0", len(call.args))
	}
	if result.Value.Kind != ValueKindString {
		t.Fatalf("Dispatch(documentCurrentScript) kind = %q, want string", result.Value.Kind)
	}
}

func TestDispatchParsesWindowNameInvocations(t *testing.T) {
	tests := []struct {
		name     string
		source   string
		method   string
		wantArgs int
		check    func(t *testing.T, call hostCall, result Value)
	}{
		{
			name:     "getter",
			source:   `host:windowName()`,
			method:   "windowName",
			wantArgs: 0,
			check: func(t *testing.T, call hostCall, result Value) {
				if result.Kind != ValueKindString || result.String != "alpha" {
					t.Fatalf("Dispatch(windowName) value = %#v, want string alpha", result)
				}
			},
		},
		{
			name:     "setter",
			source:   `host:setWindowName("alpha")`,
			method:   "setWindowName",
			wantArgs: 1,
			check: func(t *testing.T, call hostCall, result Value) {
				if call.args[0].Kind != ValueKindString || call.args[0].String != "alpha" {
					t.Fatalf("host call arg[0] = %#v, want alpha", call.args[0])
				}
				if result.Kind != ValueKindUndefined {
					t.Fatalf("Dispatch(setWindowName) value = %#v, want undefined", result)
				}
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			host := &fakeHost{
				values: map[string]Value{
					"windowName":    StringValue("alpha"),
					"setWindowName": UndefinedValue(),
				},
				errs: map[string]error{},
			}
			runtime := NewRuntime(host)

			result, err := runtime.Dispatch(DispatchRequest{Source: tc.source})
			if err != nil {
				t.Fatalf("Dispatch(%s) error = %v", tc.method, err)
			}
			if len(host.calls) != 1 {
				t.Fatalf("host calls = %#v, want one call", host.calls)
			}
			call := host.calls[0]
			if call.method != tc.method {
				t.Fatalf("host call method = %q, want %s", call.method, tc.method)
			}
			if len(call.args) != tc.wantArgs {
				t.Fatalf("host call args len = %d, want %d", len(call.args), tc.wantArgs)
			}
			if tc.check != nil {
				tc.check(t, call, result.Value)
			}
		})
	}
}

func TestDispatchParsesWebStorageInvocations(t *testing.T) {
	tests := []struct {
		name       string
		source     string
		method     string
		wantArgs   int
		wantKind   ValueKind
		wantString string
		wantNumber float64
	}{
		{
			name:       "localStorageGetItem",
			source:     `host:localStorageGetItem("theme")`,
			method:     "localStorageGetItem",
			wantArgs:   1,
			wantKind:   ValueKindString,
			wantString: "dark",
		},
		{
			name:     "localStorageSetItem",
			source:   `host:localStorageSetItem("theme", "light")`,
			method:   "localStorageSetItem",
			wantArgs: 2,
			wantKind: ValueKindUndefined,
		},
		{
			name:       "localStorageLength",
			source:     `host:localStorageLength()`,
			method:     "localStorageLength",
			wantArgs:   0,
			wantKind:   ValueKindNumber,
			wantNumber: 2,
		},
		{
			name:       "localStorageKey",
			source:     `host:localStorageKey(0)`,
			method:     "localStorageKey",
			wantArgs:   1,
			wantKind:   ValueKindString,
			wantString: "theme",
		},
		{
			name:       "sessionStorageGetItem",
			source:     `host:sessionStorageGetItem("tab")`,
			method:     "sessionStorageGetItem",
			wantArgs:   1,
			wantKind:   ValueKindString,
			wantString: "main",
		},
		{
			name:     "sessionStorageSetItem",
			source:   `host:sessionStorageSetItem("tab", "main")`,
			method:   "sessionStorageSetItem",
			wantArgs: 2,
			wantKind: ValueKindUndefined,
		},
		{
			name:     "sessionStorageRemoveItem",
			source:   `host:sessionStorageRemoveItem("tab")`,
			method:   "sessionStorageRemoveItem",
			wantArgs: 1,
			wantKind: ValueKindUndefined,
		},
		{
			name:     "sessionStorageClear",
			source:   `host:sessionStorageClear()`,
			method:   "sessionStorageClear",
			wantArgs: 0,
			wantKind: ValueKindUndefined,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			host := &fakeHost{
				values: map[string]Value{
					tc.method: func() Value {
						switch tc.wantKind {
						case ValueKindString:
							return StringValue(tc.wantString)
						case ValueKindNumber:
							return NumberValue(tc.wantNumber)
						default:
							return UndefinedValue()
						}
					}(),
				},
				errs: map[string]error{},
			}
			runtime := NewRuntime(host)

			result, err := runtime.Dispatch(DispatchRequest{Source: tc.source})
			if err != nil {
				t.Fatalf("Dispatch(%s) error = %v", tc.method, err)
			}
			if len(host.calls) != 1 {
				t.Fatalf("host calls = %#v, want one call", host.calls)
			}
			call := host.calls[0]
			if call.method != tc.method {
				t.Fatalf("host call method = %q, want %s", call.method, tc.method)
			}
			if len(call.args) != tc.wantArgs {
				t.Fatalf("host call args len = %d, want %d", len(call.args), tc.wantArgs)
			}
			if result.Value.Kind != tc.wantKind {
				t.Fatalf("Dispatch(%s) kind = %q, want %q", tc.method, result.Value.Kind, tc.wantKind)
			}
			if tc.wantKind == ValueKindString && result.Value.String != tc.wantString {
				t.Fatalf("Dispatch(%s) value = %q, want %q", tc.method, result.Value.String, tc.wantString)
			}
			if tc.wantKind == ValueKindNumber && result.Value.Number != tc.wantNumber {
				t.Fatalf("Dispatch(%s) value = %v, want %v", tc.method, result.Value.Number, tc.wantNumber)
			}
		})
	}
}

func TestDispatchParsesSetIntervalInvocation(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"setInterval": NumberValue(2),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:setInterval('host:insertAdjacentHTML("#log", "beforeend", "<span>tick</span>")', 50)`})
	if err != nil {
		t.Fatalf("Dispatch(setInterval) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if call.method != "setInterval" {
		t.Fatalf("host call method = %q, want setInterval", call.method)
	}
	if len(call.args) != 2 {
		t.Fatalf("host call args len = %d, want 2", len(call.args))
	}
	if call.args[0].Kind != ValueKindString || call.args[0].String != `host:insertAdjacentHTML("#log", "beforeend", "<span>tick</span>")` {
		t.Fatalf("host call arg[0] = %#v, want raw source string", call.args[0])
	}
	if call.args[1].Kind != ValueKindNumber || call.args[1].Number != 50 {
		t.Fatalf("host call arg[1] = %#v, want 50", call.args[1])
	}
}

func TestDispatchParsesTimerCancellationInvocations(t *testing.T) {
	tests := []struct {
		name   string
		source string
		method string
	}{
		{
			name:   "clearTimeout",
			source: `host:clearTimeout(1)`,
			method: "clearTimeout",
		},
		{
			name:   "clearTimeout interval alias",
			source: `host:clearInterval(2)`,
			method: "clearInterval",
		},
		{
			name:   "clearInterval alias only",
			source: `host:clearInterval(3)`,
			method: "clearInterval",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			host := &fakeHost{
				values: map[string]Value{
					tc.method: UndefinedValue(),
				},
				errs: map[string]error{},
			}
			runtime := NewRuntime(host)

			_, err := runtime.Dispatch(DispatchRequest{Source: tc.source})
			if err != nil {
				t.Fatalf("Dispatch(%s) error = %v", tc.method, err)
			}
			if len(host.calls) != 1 {
				t.Fatalf("host calls = %#v, want one call", host.calls)
			}
			call := host.calls[0]
			if call.method != tc.method {
				t.Fatalf("host call method = %q, want %s", call.method, tc.method)
			}
			if len(call.args) != 1 {
				t.Fatalf("host call args len = %d, want 1", len(call.args))
			}
			if call.args[0].Kind != ValueKindNumber {
				t.Fatalf("host call arg[0] = %#v, want number", call.args[0])
			}
		})
	}
}

func TestDispatchParsesAnimationFrameInvocations(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"requestAnimationFrame": NumberValue(3),
			"cancelAnimationFrame":  UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:requestAnimationFrame('host:insertAdjacentHTML("#log", "beforeend", "<span>frame</span>")')`})
	if err != nil {
		t.Fatalf("Dispatch(requestAnimationFrame) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if call.method != "requestAnimationFrame" {
		t.Fatalf("host call method = %q, want requestAnimationFrame", call.method)
	}
	if len(call.args) != 1 {
		t.Fatalf("host call args len = %d, want 1", len(call.args))
	}
	if call.args[0].Kind != ValueKindString || call.args[0].String != `host:insertAdjacentHTML("#log", "beforeend", "<span>frame</span>")` {
		t.Fatalf("host call arg[0] = %#v, want raw source string", call.args[0])
	}

	host.calls = nil
	_, err = runtime.Dispatch(DispatchRequest{Source: `host:cancelAnimationFrame(3)`})
	if err != nil {
		t.Fatalf("Dispatch(cancelAnimationFrame) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls after cancel = %#v, want one call", host.calls)
	}
	call = host.calls[0]
	if call.method != "cancelAnimationFrame" {
		t.Fatalf("host call method = %q, want cancelAnimationFrame", call.method)
	}
	if len(call.args) != 1 || call.args[0].Kind != ValueKindNumber || call.args[0].Number != 3 {
		t.Fatalf("host call args = %#v, want number 3", call.args)
	}
}

func TestDispatchParsesPreventDefaultInvocation(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"preventDefault": UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:preventDefault()`})
	if err != nil {
		t.Fatalf("Dispatch(preventDefault) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if call.method != "preventDefault" {
		t.Fatalf("host call method = %q, want preventDefault", call.method)
	}
	if len(call.args) != 0 {
		t.Fatalf("host call args len = %d, want 0", len(call.args))
	}
}

func TestDispatchParsesStopPropagationInvocation(t *testing.T) {
	host := &fakeHost{
		values: map[string]Value{
			"stopPropagation": UndefinedValue(),
		},
		errs: map[string]error{},
	}
	runtime := NewRuntime(host)

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:stopPropagation()`})
	if err != nil {
		t.Fatalf("Dispatch(stopPropagation) error = %v", err)
	}
	if len(host.calls) != 1 {
		t.Fatalf("host calls = %#v, want one call", host.calls)
	}
	call := host.calls[0]
	if call.method != "stopPropagation" {
		t.Fatalf("host call method = %q, want stopPropagation", call.method)
	}
	if len(call.args) != 0 {
		t.Fatalf("host call args len = %d, want 0", len(call.args))
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

	_, err := runtime.Dispatch(DispatchRequest{Source: `host:setInnerHTML("#out", "first;part"); host:setInnerHTML("#out", "second")`})
	if err != nil {
		t.Fatalf("Dispatch(multiple host statements) error = %v", err)
	}
	if len(host.calls) != 2 {
		t.Fatalf("host calls = %#v, want two calls", host.calls)
	}
	if host.calls[0].method != "setInnerHTML" || host.calls[1].method != "setInnerHTML" {
		t.Fatalf("host call methods = %#v, want setInnerHTML twice", host.calls)
	}
	if host.calls[0].args[1].Kind != ValueKindString || host.calls[0].args[1].String != "first;part" {
		t.Fatalf("host call[0] arg[1] = %#v, want first;part", host.calls[0].args[1])
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
