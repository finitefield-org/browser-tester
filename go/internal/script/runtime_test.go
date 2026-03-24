package script

import (
	"errors"
	"testing"
)

type fakeHost struct {
	values map[string]Value
	errs   map[string]error
	calls  []string
}

func (h *fakeHost) Call(method string, _ []Value) (Value, error) {
	h.calls = append(h.calls, method)
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
	if len(host.calls) != 1 || host.calls[0] != "version" {
		t.Fatalf("host calls = %#v, want [\"version\"]", host.calls)
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
